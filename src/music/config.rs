use std::{
    path::PathBuf,
    sync::Mutex,
};

use lazy_static::lazy_static;
use xdg::BaseDirectories;

use crate::music::data::Song;

pub(crate) static BILIBILI_REFERER: &str = "https://www.bilibili.com/";
pub(crate) static BILIBILI_UA: &str = "User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 Edg/98.0.1108.56";

pub(crate) struct PlayList {
    pub(crate) list: Vec<Song>,
    pub(crate) cur: usize,
}

impl PlayList {
    fn new() -> Self {
        PlayList {
            list: Vec::new(),
            cur: 0,
        }
    }
}

lazy_static! {
    pub(crate) static ref BILIBILI_XDG: BaseDirectories =
        BaseDirectories::with_prefix("bilibili-music-gtk4").unwrap();
    pub(crate) static ref CACHE_DIR: PathBuf = {
        BILIBILI_XDG
            .create_cache_directory(BILIBILI_XDG.get_cache_home())
            .unwrap()
    };
    pub(crate) static ref PLAYLIST: Mutex<PlayList> = Mutex::new(PlayList::new());
}
