use std::sync::Arc;

use anyhow::Ok;
use glib::{MainContext, Sender};
use gstreamer_player::prelude::Cast;
use gtk::{Builder, Button, prelude::*};
use tokio::runtime::Runtime;

use super::{config::PLAYLIST, data::download_song};

enum PlayerAction {
    DownPlay(usize),
    Play(String),
    Next,
    Prev,
    PauseOrStart,
}

pub(crate) struct Player {
    internal_player: gstreamer_player::Player,
    tx: Arc<Sender<PlayerAction>>,
    rt: Arc<Runtime>,
}

impl Player {

    fn handle(player: &Arc<Self>, action: PlayerAction) {
        match action {
            PlayerAction::DownPlay(index) => player.download_(index),
            PlayerAction::Play(song) => player.play_(song),
            PlayerAction::PauseOrStart => player.internal_player.pause(),
            PlayerAction::Next => Self::play_index_(player, 1),
            PlayerAction::Prev => Self::play_index_(player, -1),
        };
    }

    fn init_player_widget(player: &Arc<Self>, builder: &Builder) {
        let backward: Button = builder.object("backward_button").unwrap();
        let forward: Button = builder.object("forward_button").unwrap();
        let pause_button: Button = builder.object("pause_button").unwrap();

        let p = player.clone();
        forward.connect_clicked(move |_| {
            p.play_next();
        });

        let p = player.clone();
        pause_button.connect_clicked(move |_| {
            p.pause();
        });

        let p = player.clone();
        backward.connect_clicked(move |_| {
            p.play_prev();
        });
    }

    pub fn new(rt: &Arc<Runtime>, builder: &Builder) -> Arc<Self> {
        gstreamer::init().expect("failed to init gstreamer");

        let dispatcher = gstreamer_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gstreamer_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gstreamer_player::PlayerSignalDispatcher>()),
        );

        let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);
        let tx = Arc::new(tx);

        let player = Arc::new(Player {
            internal_player: player,
            tx: tx.clone(),
            rt: rt.clone(),
        });

        let rx_player = player.clone();
        rx.attach(None, move |action| {
            Self::handle(&rx_player, action);
            glib::Continue(true)
        });

        player.internal_player.connect_end_of_stream(move |_player| {
            tx.send(PlayerAction::Next).unwrap();
        });

        Self::init_player_widget(&player, builder);

        player
    }

    fn play_(&self, path: String) {
        let uri = format!("file://{}", path);
        self.internal_player.set_uri(Some(uri.as_str()));
        self.internal_player.play();
    }

    fn download_(&self, index: usize) {
        let playlist = PLAYLIST.lock().unwrap();
        if let Some(song) = playlist.list.get(index) {
            let url = song.play_url.clone();
            let name = song.name.clone();
            let tx = self.tx.clone();
            self.rt.spawn(async move {
                let s = download_song(url.as_str(), name.as_str()).await?;
                tx.send(PlayerAction::Play(s)).unwrap();
                Ok(())
            });
        }
    }

    fn play_index_(player: &Arc<Self>, inc: i32) {
        let mut playlist = PLAYLIST.lock().unwrap();
        let new: i32 = i32::try_from(playlist.cur).unwrap() + inc;
        if let std::result::Result::Ok(new) = usize::try_from(new) {
            if new < playlist.list.len().try_into().unwrap() {
                playlist.cur = new;
                player.tx.send(PlayerAction::DownPlay(new)).unwrap();
            }
        }
    }

    pub fn down_play(&self, index: usize) {
        PLAYLIST.lock().unwrap().cur = index;
        self.tx.send(PlayerAction::DownPlay(index)).unwrap();
    }

    pub fn play(&self, path: String) {
        self.tx.send(PlayerAction::Play(path)).unwrap();
    }

    pub fn play_next(&self) {
       self.tx.send(PlayerAction::Next).unwrap();
    }

    pub fn play_prev(&self) {
       self.tx.send(PlayerAction::Prev).unwrap();
    }

    pub fn pause(&self) {
        self.tx.send(PlayerAction::PauseOrStart).unwrap();
    }
}
