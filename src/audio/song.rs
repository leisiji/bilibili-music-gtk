use gtk::{glib, prelude::*, subclass::prelude::*};

pub struct SongData {
    artist: Option<String>,
    title: Option<String>,
    uuid: Option<String>,
    duration: u64,
    url: String,
    album: Option<String>,
}

impl Default for SongData {
    fn default() -> Self {
        SongData {
            artist: Some("Invalid Artist".to_string()),
            title: Some("Invalid Title".to_string()),
            uuid: None,
            duration: 0,
            url: "Invalid url".to_string(),
            album: Some("Invalid Album".to_string()),
        }
    }
}

impl SongData {
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }

    pub fn url(&self) -> String {
        String::clone(&self.url)
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecString, ParamSpecUInt};

    use super::*;

    #[derive(Default)]
    pub struct Song {
        pub data: RefCell<SongData>,
        pub playing: Cell<bool>,
        pub selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Song {
        const NAME: &'static str = "BiliBiliSong";
        type Type = super::Song;
    }

    impl ObjectImpl for Song {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        "url",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new("artist", "", "", None, ParamFlags::READABLE),
                    ParamSpecUInt::new("duration", "", "", 0, u32::MAX, 0, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "url" => {
                    if let Ok(_p) = value.get::<&str>() {
                        self.data.replace(SongData::default());
                    }
                }
                "playing" => {
                    let p = value.get::<bool>().expect("Value must be a boolean");
                    self.playing.set(p);
                }
                "selected" => {
                    let p = value.get::<bool>().expect("Value must be a boolean");
                    self.selected.set(p);
                }
                _ => unimplemented!(),
            }
        }
        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "artist" => obj.artist().to_value(),
                "album" => obj.album().to_value(),
                "title" => obj.title().to_value(),
                "duration" => obj.duration().to_value(),
                "url" => obj.url().to_value(),
                "playing" => self.playing.get().to_value(),
                "selected" => self.selected.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Song(ObjectSubclass<imp::Song>);
}

impl Default for Song {
    fn default() -> Self {
        Self::empty()
    }
}

impl Song {
    pub fn new(url: &str) -> Self {
        glib::Object::new::<Self>(&[("url", &url)]).expect("Failed to create song object")
    }

    pub fn empty() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to create an empty Song object")
    }

    pub fn selected(&self) -> bool {
        self.imp().selected.get()
    }

    pub fn set_selected(&self, selected: bool) {
        let was_selected = self.imp().selected.replace(selected);
        if was_selected != selected {
            self.notify("selected");
        }
    }

    pub fn artist(&self) -> String {
        match self.imp().data.borrow().artist() {
            Some(artist) => artist.to_string(),
            None => String::from("Unknown artist"),
        }
    }

    pub fn title(&self) -> String {
        match self.imp().data.borrow().title() {
            Some(title) => title.to_string(),
            None => String::from("Unknown title"),
        }
    }

    pub fn album(&self) -> String {
        match self.imp().data.borrow().album() {
            Some(album) => album.to_string(),
            None => String::from("Unknown album"),
        }
    }

    pub fn duration(&self) -> u64 {
        self.imp().data.borrow().duration
    }

    pub fn url(&self) -> String {
        self.imp().data.borrow().url()
    }
}
