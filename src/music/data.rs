use crate::music::config::{self, CACHE_DIR};
use anyhow::{Ok, Result};
use reqwest::header;
use serde::Deserialize;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use super::model::PlayListModel;

pub(crate) struct SongCollection {
    bvid: String,
}

#[derive(Clone)]
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
    title: String,
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

    pub async fn get_songs(&self, playlist_model: &Arc<PlayListModel>) -> Result<()> {
        const URL_COLLECTION_INFO: &str = "http://api.bilibili.com/x/web-interface/view?bvid=";
        let videoinfo = reqwest::get(format!("{}{}", URL_COLLECTION_INFO, self.bvid))
            .await?
            .json::<VideoInfo>()
            .await?;
        let mut handles = Vec::new();

        playlist_model.add_collection(&self.bvid, videoinfo.data.title);

        for page in videoinfo.data.pages {
            let playlist_model = playlist_model.clone();
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
                    playlist_model.add_song(bvid, song);
                }
                Ok(())
            });
            handles.push(h);
        }

        Ok(())
    }
}

pub async fn download_song(url: &str, name: &str) -> Result<String> {
    let path = CACHE_DIR.join(name);
    let s = path.clone().into_os_string().into_string().unwrap();
    if path.exists() {
        return Ok(s);
    }

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

    let mut dest = { File::create(path)? };
    let buf = response.bytes().await?;
    dest.write(buf.borrow())?;

    Ok(s)
}

/*
pub async fn write_config(path: &str) -> Result<()> {
}
*/
