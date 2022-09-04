use std::{rc::Rc, sync::Arc};

use gstreamer_player::prelude::Cast;
use gtk::glib::{self, clone, MainContext, Sender};

use super::{queue::Queue, song::SongData, state::PlayerState, Song};
use log::debug;

#[derive(Clone, Copy, glib::Enum, PartialEq)]
#[enum_type(name = "PlayerRepeatMode")]
pub enum RepeatMode {
    Consecutive,
    RepeatAll,
    RepeatOne,
}

impl Default for RepeatMode {
    fn default() -> Self {
        RepeatMode::Consecutive
    }
}

pub enum PlayerAction {
    PlaySong(String),
    AddSong(SongData),
    UpdatePosition(u64),
}

#[derive(PartialEq, Copy, Clone)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState::Stopped
    }
}

pub struct AudioPlayer {
    state: PlayerState,
    backend: gstreamer_player::Player,
    pub queue: Queue,
    pub tx: Arc<Sender<PlayerAction>>,
}

impl AudioPlayer {
    fn play_song(&self, song: Song) {
        song.set_playing(true);

        if let Some(uri) = song.uri() {
            self.backend.set_uri(Some(uri.as_str()));
        } else {
            let song_data = song.song_data();
            let tx = self.tx.clone();
            MainContext::default().spawn(async move {
                if let Ok(uri) = song_data.download() {
                    debug!("Download success {}.", uri);
                    tx.send(PlayerAction::PlaySong(uri)).unwrap();
                }
            });
        }

        self.state.set_current_song(Some(song));
    }

    fn set_playback_state(&self, state: PlaybackState) {
        if self.state.current_song() == None {
            if let Some(next_song) = self.queue.next_song() {
                self.play_song(next_song);
                self.state.set_playback_state(&state);

                match state {
                    PlaybackState::Playing => self.backend.play(),
                    PlaybackState::Paused => self.backend.pause(),
                    PlaybackState::Stopped => self.backend.stop(),
                }
            } else {
                self.backend.set_uri(None);
                self.state.set_current_song(None);
                self.state.set_playback_state(&PlaybackState::Stopped);
            }
        }

        self.state.set_playback_state(&state);

        match state {
            PlaybackState::Playing => self.backend.play(),
            PlaybackState::Paused => self.backend.pause(),
            PlaybackState::Stopped => self.backend.stop(),
        }
    }

    pub fn skip_to(&self, pos: u32) {
        if Some(pos) == self.queue.current_song_index() {
            return;
        }

        if let Some(current_song) = self.state.current_song() {
            current_song.set_playing(false);
        }

        if let Some(song) = self.queue.skip_song(pos) {
            let was_playing = self.state.playing();
            if was_playing {
                self.set_playback_state(PlaybackState::Paused);
            }
            self.play_song(song);
            if was_playing {
                self.set_playback_state(PlaybackState::Playing);
            }
        } else {
            self.backend.set_uri(None);
            self.state.set_current_song(None);
            self.set_playback_state(PlaybackState::Stopped);
        }
    }

    fn process_action(&self, action: PlayerAction) -> glib::Continue {
        match action {
            PlayerAction::AddSong(data) => {
                let song = Song::new(data);
                self.queue.add_song(&song);
            }
            PlayerAction::PlaySong(uri) => {
                self.backend.set_uri(Some(uri.as_str()));
                self.backend.play();
            }
            PlayerAction::UpdatePosition(pos) => {
                self.state.set_position(pos);
            }
        }
        glib::Continue(true)
    }

    fn init_backend(&self) {
        let tx = self.tx.clone();
        self.backend
            .connect_position_updated(move |_, clock| {
                if let Some(clock) = clock {
                    tx.send(PlayerAction::UpdatePosition(clock.seconds())).unwrap();
                }
            });
    }

    pub fn new() -> Rc<Self> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let dispatcher = gstreamer_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gstreamer_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gstreamer_player::PlayerSignalDispatcher>()),
        );

        let audio_player = Rc::new(Self {
            backend: player,
            state: PlayerState::default(),
            queue: Queue::default(),
            tx: Arc::new(tx),
        });

        rx.attach(
            None,
            clone!(@strong audio_player as this => move |action| this.clone().process_action(action))
        );

        audio_player.init_backend();

        audio_player
    }

    pub fn queue(&self) -> &Queue {
        return &self.queue;
    }

    pub fn play(&self) {
        if !self.state.playing() {
            self.set_playback_state(PlaybackState::Playing)
        }
    }

    pub fn state(&self) -> &PlayerState {
        &self.state
    }
}
