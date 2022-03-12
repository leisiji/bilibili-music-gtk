use super::data::Song;
use super::utils::Player;
use duration_string::DurationString;
use glib::{StaticType, MainContext};
use gtk::prelude::*;
use gtk::{prelude::CellLayoutExt, CellRendererText, ListStore, TreeView, TreeViewColumn};
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub(crate) struct PlayList {
    // store: ListStore,
    // songlist: RefCell<Vec<Song>>,
    tx: std::sync::Mutex<Sender<Song>>
}

impl PlayList {
    pub fn new(tree: &TreeView) -> Self {
        // let player = Player::new();
        // let songlist = Mutex::new(Vec::new());

        let store = ListStore::new(&[String::static_type()]);
        tree.set_model(Some(&store));

        let song_name = CellRendererText::new();
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&song_name, true);
        col.add_attribute(&song_name, "text", 0);
        tree.append_column(&col);

        /*
        let song = CellRendererText::new();
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&song, true);
        col.add_attribute(&song, "text", 0);
        tree.append_column(&col);
        */

        let (tx, rx) = std::sync::mpsc::channel::<Song>();

        MainContext::default().spawn_local(async move {
            while let Ok(song) = rx.recv() {
                println!("{:?}", song);
                let iter = store.append();
                store.set(&iter, &[(0, &song.name)]);
            }
        });

        PlayList { tx: Mutex::new(tx) }
    }

    pub fn add(&self, song: Song) {
        let tx = self.tx.lock().unwrap();
        tx.send(song);
    }
}
