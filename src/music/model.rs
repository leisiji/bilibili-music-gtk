use super::data::Song;
use super::utils::Player;
use glib::{MainContext, StaticType};
use gtk::prelude::*;
use gtk::{prelude::CellLayoutExt, CellRendererText, ListStore, TreeView, TreeViewColumn};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(crate) struct PlayList {
    // store: ListStore,
    // songlist: RefCell<Vec<Song>>,
    tx: std::sync::Mutex<Sender<Song>>,
}

impl PlayList {
    pub fn new(tree: &TreeView) -> Self {
        // let songlist = Mutex::new(Vec::new());
        let store = ListStore::new(&[
            String::static_type(),
            String::static_type(),
            String::static_type(),
        ]);

        tree.set_model(Some(&store));

        let song_name = CellRendererText::new();
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&song_name, true);
        col.add_attribute(&song_name, "text", 0);
        tree.append_column(&col);

        let duration = CellRendererText::new();
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&duration, true);
        col.add_attribute(&duration, "text", 1);
        tree.append_column(&col);

        let (tx, rx) = std::sync::mpsc::channel::<Song>();

        MainContext::default().spawn_local(async move {
            while let Ok(song) = rx.recv() {
                let iter = store.append();
                let duration_string = format!("{}", song.duration);
                store.set(
                    &iter,
                    &[(0, &song.name), (1, &duration_string), (2, &song.play_url)],
                );
            }
        });

        PlayList { tx: Mutex::new(tx) }
    }

    pub fn add(&self, song: Song) {
        let tx = self.tx.lock().unwrap();
        tx.send(song);
    }
}
