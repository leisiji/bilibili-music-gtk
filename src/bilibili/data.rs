use anyhow::{Ok, Result};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

use serde::{Deserialize, Serialize};

use crate::{audio::SongData, config::CONFIG_FILE};

/// response from bvid:
/// {
///     "code": 0,
///     "message": "0",
///     "ttl": 1,
///     "data": {
///         "bvid": "BV16f4y1o7Q5",
///         "aid": 300448445,
///         "videos": 1,
///         "tid": 31,
///         "tname": "\u7ffb\u5531",
///         "pic": "http://i1.hdslb.com/bfs/archive/813b0c3e783b9fa9960c1a1a2ea6bb93055f44e7.jpg",
///         "title": "\u3010\u7ffb\u5531\u3011Welcome to Wonderland - Anson Seabra",
///         "duration": 64,
///         "owner": {
///             "name": "\u5ed6\u6cfd\u84dd_",
///         },
///         "stat": {
///             "aid": 300448445,
///             "view": 509678,
///             "danmaku": 409,
///             "reply": 1092,
///             "favorite": 19239,
///             "coin": 8303,
///             "share": 2500,
///             "now_rank": 0,
///             "his_rank": 0,
///             "like": 104382,
///             "dislike": 0,
///             "evaluation": "",
///             "argue_msg": ""
///         },
///         "cid": 759175760,
///         "dimension": {
///             "width": 1920,
///             "height": 1080,
///             "rotate": 0
///         },
///         "pages": [
///             {
///                 "cid": 759175760,
///                 "page": 1,
///                 "from": "vupload",
///                 "part": "\u3010\u7ffb\u5531\u3011Welcome to Wonderland - Anson Seabra",
///                 "duration": 64,
///                 "vid": "",
///                 "weblink": "",
///                 "dimension": {
///                     "width": 1920,
///                     "height": 1080,
///                     "rotate": 0
///                 },
///             }
///         ]
///     }
/// }
#[derive(Deserialize)]
pub struct BvidInfo {
    data: BiliBiliVideoData,
}

#[derive(Deserialize)]
struct BiliBiliVideoData {
    title: String,
    owner: Owner,
    pages: Vec<BiliBiliPageInfo>,
}

#[derive(Deserialize)]
struct Owner {
    name: String,
}

#[derive(Deserialize)]
struct BiliBiliPageInfo {
    cid: u32,
    // part: String,
    duration: u64,
}

impl BvidInfo {
    /*
    pub fn get_pages_num(&self) -> usize {
        return self.data.pages.len();
    }
    */

    pub fn get_page_cid(&self, index: usize) -> u32 {
        if let Some(page) = self.data.pages.get(index) {
            page.cid
        } else {
            0
        }
    }

    pub fn get_page_duration(&self, index: usize) -> u64 {
        if let Some(page) = self.data.pages.get(index) {
            page.duration
        } else {
            0
        }
    }

    pub fn get_titile(&self) -> &String {
        &self.data.title
    }

    pub fn get_author(&self) -> &String {
        &self.data.owner.name
    }
}

#[derive(Deserialize, Serialize)]
struct PlayListData {
    pub data: Vec<SongData>,
}

pub fn parse_config() -> Result<Vec<SongData>> {
    let file = File::open(&*CONFIG_FILE)?;
    let buf_reader = BufReader::new(file);
    let config: PlayListData = serde_json::from_reader(buf_reader)?;
    Ok(config.data)
}

pub fn write_config(data: Vec<SongData>) -> Result<()> {
    let file = File::create(&*CONFIG_FILE)?;
    let mut buf_writer = BufWriter::new(file);
    let list = PlayListData { data };
    let s = serde_json::to_vec(&list)?;
    buf_writer.write(&s)?;
    buf_writer.flush()?;
    Ok(())
}

#[derive(Deserialize)]
pub struct BiliBiliSong {
    pub(super) baseUrl: String,
}

#[derive(Deserialize)]
pub(crate) struct Dash {
    pub(crate) audio: Vec<BiliBiliSong>,
}

#[derive(Deserialize)]
pub(crate) struct PlayUrlData {
    pub(crate) dash: Dash,
}

#[derive(Deserialize)]
pub(crate) struct PlayUrl {
    pub(crate) data: PlayUrlData,
}
