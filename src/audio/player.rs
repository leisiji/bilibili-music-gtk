use std::{rc::Rc, sync::Arc};

use gstreamer_player::{gst::ClockTime, prelude::Cast};
use gtk::glib::{self, clone, Sender};

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
    PlayNext,
    AddSong(SongData),
    UpdatePosition(u64),
    VolumeChanged(f64),
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
    fn download_song(&self, song: Song) {
        let song_data = song.song_data();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            if let Ok(uri) = song_data.download() {
                debug!("Download success {}.", uri);
                tx.send(PlayerAction::PlaySong(uri)).unwrap();
            }
        });
    }

    fn set_playback_state(&self, state: PlaybackState) {
        match state {
            PlaybackState::Playing => {
                if let Some(song) = self.state.current_song() {
                    if let Some(uri) = song.uri() {
                        self.backend.set_uri(Some(uri.as_str()));
                    } else {
                        self.state.set_playback_state(&PlaybackState::Stopped);
                        self.download_song(song);
                    }
                }
                self.backend.play();
            }
            PlaybackState::Paused => self.backend.pause(),
            PlaybackState::Stopped => self.backend.stop(),
        }
        self.state.set_playback_state(&state);
    }

    pub fn skip_to(&self, pos: u32) {
        if Some(pos) == self.queue.current_song_index() {
            return;
        }

        if let Some(current_song) = self.state.current_song() {
            current_song.set_playing(false);
        }

        if let Some(song) = self.queue.skip_song(pos) {
            self.state.set_current_song(Some(song));
            self.set_playback_state(PlaybackState::Playing);
        } else {
            self.backend.set_uri(None);
            self.state.set_current_song(None);
            self.set_playback_state(PlaybackState::Stopped);
        }
    }

    pub fn skip_next(&self) {
        if let Some(current_song) = self.state.current_song() {
            current_song.set_playing(false);
        }

        if let Some(next_song) = self.queue.next_song() {
            self.state.set_current_song(Some(next_song));
            self.set_playback_state(PlaybackState::Playing);
        } else {
            self.state.set_current_song(None);
            self.set_playback_state(PlaybackState::Stopped);
        }
    }

    pub fn skip_previous(&self) {
        if let Some(_) = self.state.current_song() {
            if self.queue.is_first_song() {
                return;
            }
        }

        if let Some(prev_song) = self.queue.previous_song() {
            prev_song.set_playing(true);
            self.state.set_current_song(Some(prev_song));
            self.backend.seek(ClockTime::from_seconds(0));
            self.set_playback_state(PlaybackState::Playing);
        }
    }

    fn process_action(&self, action: PlayerAction) -> glib::Continue {
        match action {
            PlayerAction::AddSong(data) => {
                let song = Song::new(data);
                self.queue.add_song(&song);
            }
            PlayerAction::PlaySong(uri) => {
                let was_playing = self.state.playing();
                if was_playing {
                    self.backend.stop();
                }
                self.backend.set_uri(Some(uri.as_str()));
                self.backend.play();
            }
            PlayerAction::UpdatePosition(pos) => {
                self.state.set_position(pos);
            }
            PlayerAction::PlayNext => {
                self.skip_next();
            }
            PlayerAction::VolumeChanged(volume) => {
                self.state.set_volume(volume);
            }
        }
        glib::Continue(true)
    }

    fn setup_signal(&self) {
        let tx = self.tx.clone();
        self.backend.connect_position_updated(move |_, clock| {
            if let Some(clock) = clock {
                tx.send(PlayerAction::UpdatePosition(clock.seconds()))
                    .unwrap();
            }
        });

        let tx = self.tx.clone();
        self.backend.connect_end_of_stream(move |_| {
            tx.send(PlayerAction::PlayNext).unwrap();
        });

        let tx = self.tx.clone();
        self.backend.connect_volume_changed(move |player| {
            let volume = player.volume();
            tx.send(PlayerAction::VolumeChanged(volume)).unwrap();
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

        audio_player.setup_signal();

        audio_player
    }

    pub fn queue(&self) -> &Queue {
        return &self.queue;
    }

    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    pub fn toggle_play(&self) {
        if self.state.playing() {
            self.backend.pause();
            self.state.set_playback_state(&PlaybackState::Paused);
        } else {
            self.backend.play();
            self.state.set_playback_state(&PlaybackState::Playing);
        }
    }

    pub fn set_volume(&self, volume: f64) {
        self.backend.set_volume(volume);
    }

    pub fn seek(&self, percent: f64) {
        if let Some(song) = self.state.current_song() {
            let seek_time = percent * (song.duration() as f64);
            self.backend.seek(ClockTime::from_seconds(seek_time as u64));
        }
    }
}
