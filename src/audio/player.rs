use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use gtk::glib;

use super::queue::Queue;

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

pub struct AudioPlayer {
    queue: Queue,
}

impl AudioPlayer {
    pub fn new() -> Rc<Self> {
        Rc::new(Self { queue: Queue::default() })
    }

    pub fn queue(&self) -> &Queue {
        return &self.queue;
    }
}
