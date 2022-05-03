use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use anyhow::Ok;
use glib::{MainContext, Sender};
use gstreamer_player::prelude::Cast;
use gtk::{prelude::*, Builder, Button};
use tokio::runtime::Runtime;

use super::{data::download_song, collectionlist::CollectionList};

enum PlayerAction {
    DownPlay(usize),
    Play(String),
    Next,
    Prev,
    Pause,
    PlayContinue,
}

pub(crate) struct Player {
    internal_player: gstreamer_player::Player,
    tx: Arc<Sender<PlayerAction>>,
    rt: Arc<Runtime>,
    playing: RefCell<bool>,
    pause_button: Button,
    collectionlist: Arc<Mutex<CollectionList>>,
    index: RefCell<usize>,
}

impl Player {
    fn set_playing(player: &Arc<Self>, playing: bool) {
        *player.playing.borrow_mut() = playing;
        if playing {
            player
                .pause_button
                .set_icon_name("media-playback-pause-symbolic");
        } else {
            player
                .pause_button
                .set_icon_name("media-playback-start-symbolic");
        }
    }

    fn handle(player: &Arc<Self>, action: PlayerAction) {
        match action {
            PlayerAction::DownPlay(index) => player.download_(index),
            PlayerAction::Next => player.play_index_(1),
            PlayerAction::Prev => player.play_index_(-1),
            PlayerAction::Play(song) => {
                player.play_(song);
                Self::set_playing(player, true);
            }
            PlayerAction::Pause => {
                player.internal_player.pause();
                Self::set_playing(player, false);
            }
            PlayerAction::PlayContinue => {
                player.internal_player.play();
                Self::set_playing(player, true);
            }
        };
    }

    pub fn new(rt: &Arc<Runtime>, builder: &Builder, collectionlist: &Arc<Mutex<CollectionList>>) -> Arc<Self> {
        gstreamer::init().expect("failed to init gstreamer");

        let dispatcher = gstreamer_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gstreamer_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gstreamer_player::PlayerSignalDispatcher>()),
        );

        let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);
        let tx = Arc::new(tx);

        let backward: Button = builder.object("backward_button").unwrap();
        let forward: Button = builder.object("forward_button").unwrap();
        let pause_button: Button = builder.object("pause_button").unwrap();
        let collectionlist = collectionlist.clone();

        let player = Arc::new(Player {
            internal_player: player,
            tx: tx.clone(),
            rt: rt.clone(),
            playing: RefCell::new(false),
            pause_button: pause_button.clone(),
            collectionlist,
            index: RefCell::new(0),
        });

        let rx_player = player.clone();
        rx.attach(None, move |action| {
            Self::handle(&rx_player, action);
            glib::Continue(true)
        });

        /* init backward, forward, pause button click callback */
        let p = player.clone();
        forward.connect_clicked(move |_| {
            p.play_next();
        });
        let p = player.clone();
        pause_button.connect_clicked(move |_| {
            if *p.playing.borrow() {
                p.pause();
            } else {
                p.play_continue();
            }
        });
        let p = player.clone();
        backward.connect_clicked(move |_| {
            p.play_prev();
        });

        player
            .internal_player
            .connect_end_of_stream(move |_player| {
                tx.send(PlayerAction::Next).unwrap();
            });

        player
    }

    fn play_(&self, path: String) {
        let uri = format!("file://{}", path);
        self.internal_player.set_uri(Some(uri.as_str()));
        self.internal_player.play();
    }

    fn download_(&self, index: usize) {
        let collectionlist = self.collectionlist.lock().unwrap();
        let song = collectionlist.get_song(index);
        let url = song.play_url.clone();
        let name = song.name.clone();
        let tx = self.tx.clone();
        self.rt.spawn(async move {
            let s = download_song(url.as_str(), name.as_str()).await?;
            tx.send(PlayerAction::Play(s)).unwrap();
            Ok(())
        });
    }

    pub(crate) fn play_index_(&self, inc: i32) {
        let mut index = self.index.borrow_mut();
        let new_index: i32 = *index as i32 + inc;
        if new_index >= 0 {
            *index = new_index as usize;
            self.tx.send(PlayerAction::DownPlay(*index)).unwrap();
        }
    }

    pub fn down_play(&self, i: usize) {
        let mut index = self.index.borrow_mut();
        *index = i;
        self.tx.send(PlayerAction::DownPlay(i)).unwrap();
    }

    pub fn play_next(&self) {
        self.tx.send(PlayerAction::Next).unwrap();
    }

    pub fn play_prev(&self) {
        self.tx.send(PlayerAction::Prev).unwrap();
    }

    pub fn pause(&self) {
        self.tx.send(PlayerAction::Pause).unwrap();
    }

    pub fn play_continue(&self) {
        self.tx.send(PlayerAction::PlayContinue).unwrap();
    }
}
