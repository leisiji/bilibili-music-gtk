use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use super::song::Song;

mod imp {
    use std::cell::Cell;

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject, ParamSpecUInt};

    use crate::audio::{player::RepeatMode, shuffle::ShuffleListModel, song::Song};

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
    pub fn song_at(&self, pos: u32) -> Option<Song> {
        if let Some(song) = self.imp().model.item(pos) {
            return Some(song.downcast::<Song>().unwrap());
        }
        None
    }

    pub fn current_song(&self) -> Option<Song> {
        if let Some(pos) = self.imp().current_pos.get() {
            return self.song_at(pos);
        }
        None
    }

    pub fn model(&self) -> &gio::ListModel {
        self.imp().model.as_ref()
    }

    pub fn add_song(&self, song: &Song) {
        self.imp().store.append(song);
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
    }
}
