use std::sync::Arc;

use anyhow::Ok;
use glib::{MainContext, Sender};
use gstreamer_player::prelude::Cast;
use tokio::runtime::Runtime;

use super::{config::PLAYLIST, data::download_song};

extern crate gstreamer_player;

enum PlayerAction {
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
            PlayerAction::Play(uri) => player.play_(&uri),
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

    fn play_(&self, path: &str) {
        let uri = format!("file://{}", path);
        self.internal_player.set_uri(Some(uri.as_str()));
        self.internal_player.play();
    }

    fn play_next_(player: &Arc<Self>) {
        let playlist = PLAYLIST.lock().unwrap();
        let index: usize = playlist.cur.try_into().unwrap();
        if let Some(song) = playlist.list.get(index) {
            let url = song.play_url.clone();
            let name = song.name.clone();
            let next_player = player.clone();
            player.rt.spawn(async move {
                let s = download_song(url.as_str(), name.as_str()).await?;
                next_player.play(s.as_str());
                Ok(())
            });
        }
    }

    pub fn play(&self, path: &str) {
       self.tx.send(PlayerAction::Play(path.to_string())).unwrap();
    }

    pub fn play_next(&self) {
       self.tx.send(PlayerAction::Next).unwrap();
    }
}
