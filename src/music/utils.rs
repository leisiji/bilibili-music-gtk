use std::sync::Arc;

use anyhow::Ok;
use glib::{MainContext, Sender};
use gstreamer_player::prelude::Cast;
use tokio::runtime::Runtime;

use super::{config::PLAYLIST, data::download_song};

extern crate gstreamer_player;

enum PlayerAction {
    DownPlay(u32),
    Play(String),
    Next,
    Pause,
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
            PlayerAction::Pause => println!("todo"),
            PlayerAction::Next => Self::play_next_(player),
        };
    }

    pub fn new(rt: &Arc<Runtime>) -> Arc<Self> {
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

        player
    }

    fn play_(&self, path: String) {
        let uri = format!("file://{}", path);
        self.internal_player.set_uri(Some(uri.as_str()));
        self.internal_player.play();
    }

    fn download_(&self, index: u32) {
        let playlist = PLAYLIST.lock().unwrap();
        let index: usize = index.try_into().unwrap();
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

    fn play_next_(player: &Arc<Self>) {
        let index: usize;
        {
            let mut playlist = PLAYLIST.lock().unwrap();
            playlist.cur = playlist.cur + 1;
            index = playlist.cur.try_into().unwrap();
            if index >= playlist.list.len() {
                return;
            }
        }
        player.tx.send(PlayerAction::DownPlay(index.try_into().unwrap())).unwrap();
    }

    pub fn down_play(&self, index: u32) {
       self.tx.send(PlayerAction::DownPlay(index)).unwrap();
    }

    pub fn play(&self, path: String) {
        self.tx.send(PlayerAction::Play(path)).unwrap();
    }

    pub fn play_next(&self) {
       self.tx.send(PlayerAction::Next).unwrap();
    }
}
