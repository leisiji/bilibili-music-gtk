use std::io::{BufReader, prelude::*, BufWriter};
use std::path::PathBuf;
use std::fs::File;

use anyhow::{Ok, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

pub(crate) static BILIBILI_REFERER: &str = "https://www.bilibili.com/";
pub(crate) static BILIBILI_UA: &str = "User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 Edg/98.0.1108.56";
const APP_DIR: &str = "bilibili-music-gtk4";

#[derive(Debug, Deserialize, Serialize)]
pub struct Bv {
    pub bvid: String,
    pub blacklist: Vec<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub(crate) bv_list: Vec<Bv>,
}

pub fn parse_config() -> Result<Config> {
    let file = File::open(&*CONFIG_FILE)?;
    let buf_reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(buf_reader)?;
    Ok(config)
}

pub fn write_config(config: &Config) -> Result<()> {
    let file = File::create(&*CONFIG_FILE)?;
    let mut buf_writer = BufWriter::new(file);
    let s = serde_json::to_vec(config)?;
    buf_writer.write(&s)?;
    buf_writer.flush()?;
    Ok(())
}

impl Bv {
    pub fn new(bv: &String) -> Self {
        Bv {
            bvid: bv.clone(),
            blacklist: Vec::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            bv_list: Vec::new()
        }
    }
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
        let dir = BILIBILI_XDG.get_config_home();
        let file = dir.join("config.json");
        if !dir.exists() {
            BILIBILI_XDG.create_cache_directory(dir).unwrap();
            if !file.exists() {
                File::create(file.clone()).unwrap();
            }
        }
        file
    };
}
