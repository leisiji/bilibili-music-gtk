use gtk::{gdk, gio, glib, subclass::prelude::*};

pub struct SongData {
    artist: Option<String>,
    title: Option<String>,
    uuid: Option<String>,
    duration: u64,
    file: gio::File,
    url: Option<String>,
}

impl SongData {
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

impl Default for SongData {
    fn default() -> Self {
        SongData {
            artist: Some("Invalid Artist".to_string()),
            title: Some("Invalid Title".to_string()),
            uuid: None,
            duration: 0,
            file: gio::File::for_path("/does-not-exist"),
            url: Some("Invalid url".to_string()),
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecString, ParamSpecUInt};

    use super::*;

    #[derive(Default)]
    pub struct Song {
        pub data: RefCell<SongData>,
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
}
