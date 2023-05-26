use anyhow::Result;
use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Serialize};
use lofty::{read_from_path, ParseOptions, TaggedFileExt};

use crate::{
    bilibili::{data::BvidInfo, download_song, get_url},
    config::CACHE_DIR,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct SongData {
    artist: Option<String>,
    title: String,
    duration: u64,
    bvid: String,
    cid: u32,
    album: Option<String>,
}

impl Default for SongData {
    fn default() -> Self {
        SongData {
            artist: Some("Invalid Artist".to_string()),
            title: "Invalid Title".to_string(),
            duration: 0,
            bvid: "Invalid bvid".to_string(),
            cid: 0,
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

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    fn escape(s: &str) -> String {
        s.replace("/", ",")
    }

    pub fn file_name(&self) -> String {
        let suffix: &str = ".m4a";
        if let Some(s) = self.album() {
            let mut s = Self::escape(s);
            s.push('-');
            s.push_str(&Self::escape(self.title()));
            s.push_str(suffix);
            s
        } else {
            let mut s = Self::escape(self.title());
            s.push_str(suffix);
            s
        }
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }

    pub fn bvid(&self) -> String {
        String::clone(&self.bvid)
    }

    pub fn from_bvid(bvid: &str) -> Result<Vec<SongData>> {
        let mut songs = Vec::new();
        let bvid_info: BvidInfo = BvidInfo::from_bvid(bvid)?;

        let pages = bvid_info.get_pages();
        if pages.len() == 1 {
            if let Some(episodes) = bvid_info.get_episodes() {
                for i in episodes {
                    let song_data = Self {
                        artist: Some(bvid_info.get_author().clone()),
                        title: i.page.part,
                        album: None,
                        duration: i.page.duration,
                        bvid: i.bvid,
                        cid: i.page.cid,
                    };
                    songs.push(song_data);
                }
            } else {
                let page = pages.get(0).unwrap();
                let song_data = Self {
                    artist: Some(bvid_info.get_author().clone()),
                    title: bvid_info.get_titile().clone(),
                    album: None,
                    duration: page.duration,
                    bvid: bvid.to_string(),
                    cid: page.cid,
                };
                songs.push(song_data);
            }
        } else {
            for i in pages {
                let song_data = Self {
                    artist: Some(bvid_info.get_author().clone()),
                    title: i.part.clone(),
                    album: Some(bvid_info.get_titile().clone()),
                    duration: i.duration,
                    bvid: bvid.to_string(),
                    cid: i.cid,
                };
                songs.push(song_data);
            }
        }

        Ok(songs)
    }

    pub fn download(&self) -> Result<String> {
        let song_path = CACHE_DIR.join(self.file_name());
        let url = get_url(self.bvid.as_str(), self.cid)?;
        download_song(url.as_str(), song_path.to_str().unwrap())?;
        let uri = format!("file://{}", song_path.display());
        Ok(uri)
    }
}

impl PartialEq for SongData {
    fn eq(&self, other: &Self) -> bool {
        return (self.bvid == other.bvid) && (self.cid == other.cid);
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, ParamSpecUInt};

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
                        "bvid",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE | ParamFlags::CONSTRUCT_ONLY,
                    ),
                    ParamSpecString::new("artist", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("album", "", "", None, ParamFlags::READABLE),
                    ParamSpecUInt::new("duration", "", "", 0, u32::MAX, 0, ParamFlags::READABLE),
                    ParamSpecString::new("title", "", "", None, ParamFlags::READABLE),
                    ParamSpecBoolean::new("playing", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("selected", "", "", false, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        /* set_property 无需实现 properties 中的所有属性，因为某些属性是不可变的 */
        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "bvid" => {
                    if let Ok(bvid) = value.get::<&str>() {
                        if let Ok(song_data) = SongData::from_bvid(bvid) {
                            self.data.replace(song_data.get(0).unwrap().clone());
                            obj.notify("artist");
                            obj.notify("title");
                            obj.notify("album");
                            obj.notify("duration");
                        }
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
                "bvid" => obj.bvid().to_value(),
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
    pub fn new(data: SongData) -> Self {
        let obj = glib::Object::new::<Self>(&[]).expect("Failed to create an empty Song object");
        obj.imp().data.replace(data);
        obj.notify("artist");
        obj.notify("title");
        obj.notify("album");
        obj.notify("duration");
        return obj;
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

    pub fn set_playing(&self, playing: bool) {
        let was_playing = self.imp().playing.replace(playing);
        if was_playing != playing {
            self.notify("playing");
        }
    }

    pub fn artist(&self) -> String {
        match self.imp().data.borrow().artist() {
            Some(artist) => artist.to_string(),
            None => String::from("Unknown artist"),
        }
    }

    pub fn title(&self) -> String {
        self.imp().data.borrow().title().to_string()
    }

    pub fn file_name(&self) -> String {
        self.imp().data.borrow().file_name()
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

    pub fn bvid(&self) -> String {
        self.imp().data.borrow().bvid()
    }

    pub fn cid(&self) -> u32 {
        self.imp().data.borrow().cid
    }

    pub fn uri(&self) -> Option<String> {
        let song_path = CACHE_DIR.join(self.file_name());
        if song_path.exists() {
            let s = glib::filename_to_uri(song_path, None).unwrap().into();
            Some(s)
        } else {
            None
        }
    }

    pub fn song_data(&self) -> SongData {
        self.imp().data.borrow().clone()
    }

    pub fn equals(&self, other: &Self) -> bool {
        *self.imp().data.borrow() == *other.imp().data.borrow()
    }

    pub fn playing(&self) -> bool {
        self.imp().playing.get()
    }
}

#[cfg(test)]
mod test {
    use lofty::{read_from_path, ParseOptions, TaggedFileExt};

    #[test]
    fn test_song() {
        if let Ok(tagged_file) = read_from_path("/home/ye/.cache/bilibili-music-gtk4/我在游戏里和BUG谈恋爱？！.m4a") {
            let m4a = tagged_file.primary_tag().unwrap();
        } else {
            println!("read_from_path failed");
        }
        // let uri = glib::filename_to_uri(song_path, None);
        // let s: String = uri.unwrap().into();
    }
}
