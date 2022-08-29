use std::{fs::File, path::PathBuf};

use lazy_static::lazy_static;
use xdg::BaseDirectories;

const APP_DIR: &str = "bilibili-music-gtk4";
pub static APPLICATION_ID: &str = "org.bilibili.music";

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
