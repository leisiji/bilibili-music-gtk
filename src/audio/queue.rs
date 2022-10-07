use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::bilibili::data::write_config;

use super::{song::Song, RepeatMode, SongData};

mod imp {
    use std::cell::Cell;

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject, ParamSpecUInt};

    use crate::{
        audio::{shuffle::ShuffleListModel, song::Song},
        bilibili::data::parse_config,
    };

    use super::*;

    #[derive(Default)]
    pub struct Queue {
        pub store: gio::ListStore,
        pub repeat_mode: Cell<RepeatMode>,
        pub current_pos: Cell<Option<u32>>,
        pub model: ShuffleListModel,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Queue {
        const NAME: &'static str = "PlayerQueue";
        type Type = super::Queue;

        fn new() -> Self {
            let store = gio::ListStore::new(Song::static_type());
            let model = ShuffleListModel::new(Some(&store));

            if let Ok(data) = parse_config() {
                for i in data {
                    let song = Song::new(i);
                    store.append(&song);
                }
            }

            Self {
                store,
                repeat_mode: Cell::new(RepeatMode::default()),
                current_pos: Cell::new(None),
                model,
            }
        }
    }

    impl ObjectImpl for Queue {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "current",
                        "",
                        "",
                        Song::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecEnum::new(
                        "repeat-mode",
                        "",
                        "",
                        RepeatMode::static_type(),
                        0,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecUInt::new("n-songs", "", "", 0, u32::MAX, 0, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "current" => obj.current_song().to_value(),
                "repeat-mode" => self.repeat_mode.get().to_value(),
                "n-songs" => self.store.n_items().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Queue(ObjectSubclass<imp::Queue>);
}

impl Default for Queue {
    fn default() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to create Queue object")
    }
}

impl Queue {
    pub fn select_song_at(&self, index: u32) {
        if let Some(song) = self.imp().model.item(index) {
            let song = song.downcast_ref::<Song>().unwrap();
            let is_selected = !song.selected();
            song.set_selected(is_selected);
        }
    }

    pub fn song_at(&self, pos: u32) -> Option<Song> {
        if let Some(song) = self.imp().model.item(pos) {
            return Some(song.downcast::<Song>().unwrap());
        }
        None
    }

    pub fn n_songs(&self) -> u32 {
        self.imp().model.n_items()
    }

    pub fn is_empty(&self) -> bool {
        self.imp().model.n_items() == 0
    }

    pub fn is_first_song(&self) -> bool {
        if let Some(current_pos) = self.imp().current_pos.get() {
            return current_pos == 0;
        }

        false
    }

    pub fn current_song(&self) -> Option<Song> {
        if let Some(pos) = self.imp().current_pos.get() {
            return self.song_at(pos);
        }
        None
    }

    pub fn current_song_index(&self) -> Option<u32> {
        self.imp().current_pos.get()
    }

    pub fn model(&self) -> &gio::ListModel {
        self.imp().model.as_ref()
    }

    pub fn add_song(&self, song: &Song) {
        let n = self.model().n_items();
        for i in 1..n {
            let iter = self.model().item(i).unwrap().downcast::<Song>().unwrap();
            if song.equals(&iter) {
                return;
            }
        }

        self.imp().store.append(song);
        self.notify("n-songs");
        self.sync_config();
    }

    pub fn add_songs(&self, songs: &[impl IsA<glib::Object>]) {
        let is_shuffled = self.imp().model.shuffled();
        self.imp().model.unshuffle();

        self.imp()
            .store
            .splice(self.imp().model.n_items(), 0, songs);

        if is_shuffled {
            self.imp().model.reshuffle();
        }
        self.sync_config();
    }

    pub fn skip_song(&self, pos: u32) -> Option<Song> {
        self.imp().current_pos.replace(Some(pos));
        self.notify("current");
        self.song_at(pos)
    }

    fn to_vec(&self) -> Vec<SongData> {
        let mut v: Vec<SongData> = Vec::new();
        let n = self.model().n_items();
        for i in 0..n {
            let obj = self.model().item(i).unwrap();
            let song = obj.downcast_ref::<Song>().unwrap();
            v.push(song.imp().data.borrow().clone());
        }
        v
    }

    fn sync_config(&self) {
        let data: Vec<SongData> = self.to_vec();
        write_config(data).unwrap();
    }

    pub fn previous_song(&self) -> Option<Song> {
        if let Some(current_pos) = self.imp().current_pos.get() {
            if current_pos > 0 {
                let prev = current_pos - 1;
                self.imp().current_pos.replace(Some(prev));
                self.notify("current");
                return self.song_at(current_pos - 1);
            }
        }

        None
    }

    pub fn next_song(&self) -> Option<Song> {
        let store = &self.imp().model;

        let n_songs = store.n_items();
        if n_songs == 0 {
            return None;
        }

        let repeat_mode = self.imp().repeat_mode.get();
        if let Some(current) = self.current_song_index() {
            let next: Option<u32> = match repeat_mode {
                RepeatMode::Consecutive if current < n_songs => Some(current + 1),
                RepeatMode::RepeatOne => Some(current),
                RepeatMode::RepeatAll if current < n_songs - 1 => Some(current + 1),
                RepeatMode::RepeatAll if current == n_songs - 1 => Some(0),
                _ => None,
            };

            if let Some(next) = next {
                self.imp().current_pos.replace(Some(next));
                self.notify("current");
                self.song_at(next)
            } else {
                self.imp().current_pos.replace(None);
                self.notify("current");
                None
            }
        } else {
            // Return the first song
            self.imp().current_pos.replace(Some(0));
            self.notify("current");
            self.song_at(0)
        }
    }

    pub fn unselect_all_songs(&self) {
        for i in 0..self.imp().store.n_items() {
            let song = self.imp().store.item(i).unwrap();
            song.downcast_ref::<Song>().unwrap().set_selected(false);
        }
    }

    pub fn n_selected_songs(&self) -> u32 {
        let mut count = 0;
        for i in 0..self.imp().store.n_items() {
            let song = self.imp().store.item(i).unwrap();
            if song.downcast_ref::<Song>().unwrap().selected() {
                count += 1;
            }
        }

        count
    }

    pub fn remove_song(&self, song: &Song) {
        let was_shuffled = self.imp().model.shuffled();
        let n_songs = self.n_songs();
        for pos in 0..n_songs {
            let s = self
                .imp()
                .store
                .item(pos)
                .unwrap()
                .downcast::<Song>()
                .unwrap();
            if s.equals(song) {
                self.imp().store.remove(pos);
                break;
            }
        }

        if n_songs != self.n_songs() {
            if was_shuffled {
                self.imp().model.reshuffle();
            }
            self.notify("n-songs");
        }
        self.sync_config();
    }
}
