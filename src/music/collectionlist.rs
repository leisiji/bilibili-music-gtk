use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use super::data::Song;

pub(crate) struct CollectionList {
    playlist_map: BTreeMap<String, Vec<Song>>,
    pub cur_bvid: RefCell<String>,
}
const FIRST_KEY: &str = "all";

impl CollectionList {
    pub fn new() -> Arc<Mutex<Self>> {
        let mut map: BTreeMap<String, Vec<Song>> = BTreeMap::new();
        map.insert(String::from(FIRST_KEY), Vec::new());

        let collectionlist = CollectionList {
            playlist_map: map,
            cur_bvid: RefCell::new(String::from(FIRST_KEY)),
        };
        let collectionlist = Arc::new(Mutex::new(collectionlist));

        collectionlist
    }

    pub fn get_song(&self, index: usize) -> &Song {
        let bvid = self.cur_bvid.borrow();
        let playlist = self.playlist_map.get(&*bvid).unwrap();
        return playlist.get(index).unwrap();
    }

    pub fn get_collection(&self, bvid: &String) -> Option<&Vec<Song>> {
        return self.playlist_map.get(bvid);
    }

    pub fn get_collection_size(&self) -> usize {
        let bvid = self.cur_bvid.borrow();
        let playlist = self.playlist_map.get(&*bvid).unwrap();
        return playlist.len();
    }

    pub fn add_song(&mut self, bvid: &String, song: &Song) {
        let playlist = self.playlist_map.get_mut(bvid).unwrap();
        playlist.push(song.clone());
        let playlist = self.playlist_map.get_mut(FIRST_KEY).unwrap();
        playlist.push(song.clone());
    }

    pub fn add_collection(&mut self, bvid: &String) {
        if !self.playlist_map.contains_key(bvid) {
            self.playlist_map.insert(bvid.clone(), Vec::new());
        }
    }
}
