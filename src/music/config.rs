use std::path::PathBuf;

use lazy_static::lazy_static;
use xdg::BaseDirectories;

pub(crate) static BILIBILI_REFERER: &str = "https://www.bilibili.com/";
pub(crate) static BILIBILI_UA: &str = "User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 Edg/98.0.1108.56";

lazy_static! {
    pub(crate) static ref BILIBILI_XDG: BaseDirectories =
        BaseDirectories::with_prefix("bilibili-music-gtk4").unwrap();
    pub(crate) static ref CACHE_DIR: PathBuf = {
        BILIBILI_XDG
            .create_cache_directory(BILIBILI_XDG.get_cache_home())
            .unwrap()
    };
}
