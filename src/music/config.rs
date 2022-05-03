use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Ok, Result};
use lazy_static::lazy_static;
use serde::Deserialize;
use xdg::BaseDirectories;

use crate::music::data::Song;

pub(crate) static BILIBILI_REFERER: &str = "https://www.bilibili.com/";
pub(crate) static BILIBILI_UA: &str = "User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 Edg/98.0.1108.56";
const APP_DIR: &str = "bilibili-music-gtk4";

pub(crate) struct PlayList {
    pub(crate) list: Vec<Song>,
    pub(crate) cur: usize,
}

impl PlayList {
    pub fn new() -> Self {
        PlayList {
            list: Vec::new(),
            cur: 0,
        }
    }
}

#[derive(Deserialize)]
pub struct Bv {
    pub bvid: String,
    pub blacklist: Vec<u32>,
}

#[derive(Deserialize)]
pub struct Config {
    pub bv_list: Vec<Bv>,
}

pub fn parse_config() -> Result<Config> {
    let file = File::open(&*CONFIG_FILE)?;
    let buf_reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(buf_reader)?;
    Ok(config)
}

lazy_static! {
    pub(crate) static ref BILIBILI_XDG: BaseDirectories =
        BaseDirectories::with_prefix(APP_DIR).unwrap();
    pub(crate) static ref CACHE_DIR: PathBuf = {
        BILIBILI_XDG
            .create_cache_directory(BILIBILI_XDG.get_cache_home())
            .unwrap()
    };
    pub(crate) static ref CONFIG_FILE: PathBuf = {
        let file = BILIBILI_XDG.get_config_home().join("config.json");
        if !file.exists() {
            BILIBILI_XDG.create_cache_directory(file).unwrap()
        } else {
            file
        }
    };
}
