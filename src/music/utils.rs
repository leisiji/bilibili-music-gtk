use std::sync::Arc;

use glib::{MainContext, Sender};
use gstreamer_player::prelude::Cast;

extern crate gstreamer_player;

enum PlayerAction {
    Play(String),
    Pause,
}

pub(crate) struct Player {
    internal_player: gstreamer_player::Player,
    tx: Sender<PlayerAction>,
}

impl Player {
    fn play_handle(&self, action: PlayerAction) {
        match action {
            PlayerAction::Play(uri) => self.play_(&uri),
            PlayerAction::Pause => println!("todo"),
        };
    }

    pub fn new() -> Arc<Self> {
        gstreamer::init().expect("failed to init gstreamer");

        let dispatcher = gstreamer_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gstreamer_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gstreamer_player::PlayerSignalDispatcher>()),
        );

        let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);

        let player = Arc::new(Player {
            internal_player: player,
            tx,
        });

        let rx_player = player.clone();
        rx.attach(None, move |action| {
            rx_player.play_handle(action);
            glib::Continue(true)
        });

        player
    }

    fn play_(&self, path: &str) {
        println!("{}", path);
        self.internal_player.set_uri(Some(path));
        self.internal_player.play();
    }
    pub fn play(&self, uri: &str) {
        self.tx.send(PlayerAction::Play(uri.to_string())).unwrap();
    }
}
