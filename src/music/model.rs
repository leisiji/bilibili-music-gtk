use super::utils::Player;
use glib::StaticType;
use gtk::prelude::*;
use gtk::{prelude::CellLayoutExt, CellRendererText, ListStore, TreeView, TreeViewColumn};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub(crate) struct PlayList {
    pub tree: TreeView,
    store: ListStore,
    player: Arc<Player>,
    songlist: Arc<Mutex<RefCell<Vec<String>>>>,
}

impl PlayList {
    pub fn new() -> Self {
        let player = Player::new();
        let store = ListStore::new(&[String::static_type()]);
        let tree = TreeView::with_model(&store);
        let songlist: Arc<Mutex<RefCell<Vec<String>>>> =
            Arc::new(Mutex::new(RefCell::new(Vec::new())));

        let song = CellRendererText::new();
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&song, true);
        col.add_attribute(&song, "text", 0);
        tree.append_column(&col);

        let p = player.clone();
        let list = songlist.clone();
        tree.connect_row_activated(move |_, path, _| {
            let index: Result<usize, _> = path.indices()[0].try_into();
            if let Ok(index) = index {
                let list = list.lock().unwrap();
                let song = &list.borrow()[index];
                p.play(song.as_str());
            }
        });

        PlayList {
            tree,
            store,
            player,
            songlist,
        }
    }

    pub fn add(&self, file: String) {
        let iter = self.store.append();
        self.store.set(&iter, &[(0, &file)]);

        let songlist = self.songlist.lock().unwrap();
        let mut songlist = songlist.borrow_mut();
        songlist.push(file);
    }
}
