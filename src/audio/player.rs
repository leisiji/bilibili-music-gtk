use std::{rc::Rc, sync::Arc};

use gtk::glib::{self, clone, Sender};

use super::{queue::Queue, song::SongData, Song};

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
    AddSong(SongData),
}

pub struct AudioPlayer {
    queue: Queue,
    pub tx: Arc<Sender<PlayerAction>>,
}

impl AudioPlayer {
    fn process_action(&self, action: PlayerAction) -> glib::Continue {
        match action {
            PlayerAction::AddSong(data) => {
                let song = Song::new(data);
                self.queue.add_song(&song);
            }
        }
        glib::Continue(true)
    }

    pub fn new() -> Rc<Self> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let audio_player = Rc::new(Self {
            queue: Queue::default(),
            tx: Arc::new(tx),
        });

        rx.attach(
            None,
            clone!(@strong audio_player as this => move |action| this.clone().process_action(action))
        );

        audio_player
    }

    pub fn queue(&self) -> &Queue {
        return &self.queue;
    }
}
