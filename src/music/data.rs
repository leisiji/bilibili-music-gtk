use crate::music::config::{self, CACHE_DIR};
use anyhow::{Ok, Result};
use reqwest::header;
use serde::Deserialize;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::ops::Add;
use std::sync::Arc;
use tokio::runtime::Runtime;

use super::model::PlayList;

pub(crate) struct SongCollection {
    bvid: String,
}

#[derive(Debug)]
pub struct Song {
    pub name: String,
    pub duration: u32,
    pub play_url: String,
}

#[derive(Deserialize)]
pub struct SongInfo {
    baseUrl: String,
}

#[derive(Deserialize)]
pub struct PageInfo {
    cid: u32,
    part: String,
    duration: u32,
}

#[derive(Deserialize)]
struct SongCollectionData {
    pages: Vec<PageInfo>,
}

#[derive(Deserialize)]
struct Dash {
    audio: Vec<SongInfo>,
}

#[derive(Deserialize)]
struct PlayerData {
    dash: Dash,
}

#[derive(Deserialize)]
struct PlayerInfo {
    data: PlayerData,
}

#[derive(Deserialize)]
struct VideoInfo {
    data: SongCollectionData,
}

impl SongCollection {
    pub fn new(bvid: &str) -> Self {
        let bvid = String::from(bvid);
        SongCollection { bvid }
    }

    pub async fn get_songs(&self, list: Arc<PlayList>) -> Result<()> {
        const URL_COLLECTION_INFO: &str = "http://api.bilibili.com/x/web-interface/view?bvid=";
        let videoinfo = reqwest::get(format!("{}{}", URL_COLLECTION_INFO, self.bvid))
            .await?
            .json::<VideoInfo>()
            .await?;
        let mut handles = Vec::new();

        for page in videoinfo.data.pages {
            let list = list.clone();
            let bvid = self.bvid.clone();
            let h = tokio::spawn(async move {
                let player_info = reqwest::get(format!(
                    "https://api.bilibili.com/x/player/playurl?cid={}&bvid={}&qn=64&fnval=16",
                    page.cid, bvid
                ))
                .await?
                .json::<PlayerInfo>()
                .await?;

                if let Some(audio) = player_info.data.dash.audio.get(0) {
                    let song: Song = Song {
                        name: page.part.clone(),
                        play_url: audio.baseUrl.clone(),
                        duration: page.duration,
                    };
                    list.add(song);
                }
                Ok(())
            });
            handles.push(h);
        }

        for h in handles {
            h.await?;
        }

        Ok(())
    }
}

pub async fn download_song(url: &str, name: &str) -> Result<String> {
    let mut headers = header::HeaderMap::default();
    headers.insert(
        header::REFERER,
        header::HeaderValue::from_static(config::BILIBILI_REFERER),
    );
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(config::BILIBILI_UA),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let response = client.get(url).send().await?;

    let path = CACHE_DIR.join(name);
    let s = path.clone().into_os_string().into_string().unwrap();

    let mut dest = { File::create(path)? };
    let buf = response.bytes().await?;
    dest.write(buf.borrow())?;

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::SongCollection;
    use tokio::runtime::Runtime;

    #[test]
    fn it_works() {
        let rt = Runtime::new().unwrap();
        let s = SongCollection::new("BV135411V7A5");
        rt.block_on(s.get_songs(|song| {
            println!("{:?}", song);
        }));
    }
}
