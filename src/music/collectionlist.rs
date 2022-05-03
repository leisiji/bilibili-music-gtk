use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::music::config::PlayList;

use super::{config::parse_config, data::SongCollection, model::PlayListModel};

pub(crate) struct CollectionList {
    playlist_map: BTreeMap<String, Arc<Mutex<PlayList>>>,
    cur_bvid: String,
}
const FIRST_KEY: &str = "all";

impl CollectionList {
    pub fn new() -> Self {
        let mut map: BTreeMap<String, Arc<Mutex<PlayList>>> = BTreeMap::new();
        map.insert(
            String::from(FIRST_KEY),
            Arc::new(Mutex::new(PlayList::new())),
        );

        CollectionList {
            playlist_map: map,
            cur_bvid: String::from(FIRST_KEY),
        }
    }

    pub fn get(&self, bvid: Option<&String>) -> Option<&Arc<Mutex<PlayList>>> {
        match bvid {
            None => self.playlist_map.get(FIRST_KEY),
            Some(key) => self.playlist_map.get(key),
        }
    }
}
