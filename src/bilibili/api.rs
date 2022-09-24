use std::fs::File;
use std::io::Write;
use log::debug;

use anyhow::{Ok, Result};

use super::data::{BvidInfo, PlayUrl};

impl BvidInfo {
    pub fn from_bvid(bvid: &str) -> Result<BvidInfo> {
        const URL_BVID_INFO: &str = "http://api.bilibili.com/x/web-interface/view?bvid=";
        let req = format!("{}{}", URL_BVID_INFO, bvid).to_string();
        let resp = ureq::get(&req).call()?.into_string()?;
        let info: BvidInfo = serde_json::from_str(resp.as_str())?;
        Ok(info)
    }
}

pub fn get_url(bvid: &str, cid: u32) -> Result<String> {
    let req = format!(
        "https://api.bilibili.com/x/player/playurl?cid={}&bvid={}&qn=64&fnval=16",
        cid, bvid
    )
    .to_string();
    let resp = ureq::get(&req).call()?.into_string()?;
    let play_url: PlayUrl = serde_json::from_str(resp.as_str())?;
    let url = play_url.data.dash.audio[0].baseUrl.clone();
    debug!("bvid: {}, cid: {}, url: {}", bvid, cid, url);
    Ok(url)
}

pub fn download_url(url: &str, path: &str) -> Result<()> {
    static BILIBILI_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36 Edg/98.0.1108.56";
    static BILIBILI_REFERER: &str = "https://www.bilibili.com/";

    let resp = ureq::get(url)
        .set("User-Agent", BILIBILI_UA)
        .set("Referer", BILIBILI_REFERER)
        .call()?;

    let mut dest = File::create(path)?;
    let mut bytes: Vec<u8> = Vec::new();
    resp.into_reader().read_to_end(&mut bytes)?;
    dest.write(bytes.as_slice())?;
    Ok(())
}
