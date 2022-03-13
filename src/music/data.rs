use std::cell::{Ref, RefCell};

use anyhow::{Ok, Result};
use serde::Deserialize;

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
    bvid: String,
    pages: Vec<PageInfo>,
}

#[derive(Deserialize)]
struct Dash {
    duration: u32,
    audio: Vec<SongInfo>,
}

#[derive(Deserialize)]
struct PlayerData {
    dash: Dash,
}

#[derive(Deserialize)]
struct PlayerInfo {
    code: u32,
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

    pub async fn get_songs<F: Fn(Song)>(&self, consume: F) -> Result<()> {
        const URL_COLLECTION_INFO: &str = "http://api.bilibili.com/x/web-interface/view?bvid=";
        let videoinfo = reqwest::get(format!("{}{}", URL_COLLECTION_INFO, self.bvid))
            .await?
            .json::<VideoInfo>()
            .await?;
        for page in &videoinfo.data.pages[0..10] {
            let player_info = reqwest::get(format!(
                "https://api.bilibili.com/x/player/playurl?cid={}&bvid={}&qn=64&fnval=16",
                page.cid, self.bvid
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
                consume(song);
            }
        }
        Ok(())
    }
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
