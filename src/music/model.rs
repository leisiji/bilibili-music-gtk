use super::data::Song;
use glib::StaticType;
use gtk::prelude::*;
use gtk::{prelude::CellLayoutExt, CellRendererText, ListStore, TreeView, TreeViewColumn};
use std::sync::Mutex;

pub(crate) struct PlayList {
    tx: Mutex<glib::Sender<TreeViewCtrl>>,
}

pub(crate) enum TreeViewCtrl {
    Start,
    Add(Song),
    End,
}

impl PlayList {
    pub fn new(tree: TreeView) -> Self {
        // name, duration, play_url
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

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        rx.attach(None, move |ctrl| {
            match ctrl {
                TreeViewCtrl::Add(song) => {
                    let iter = store.append();
                    let duration_string = format!("{}", song.duration);
                    store.set(
                        &iter,
                        &[(0, &song.name), (1, &duration_string), (2, &song.play_url)],
                    );
                }
                TreeViewCtrl::Start => todo!(),
                TreeViewCtrl::End => todo!(),
            };
            glib::Continue(true)
        });

        PlayList { tx: Mutex::new(tx) }
    }

    pub fn add(&self, song: Song) {
        let tx = self.tx.lock().unwrap();
        tx.send(TreeViewCtrl::Add(song))
            .expect("Failed to add song");
    }
    pub fn start(&self) {
        let tx = self.tx.lock().unwrap();
        tx.send(TreeViewCtrl::Start)
            .expect("Failed to start");
    }
    pub fn end(&self) {
        let tx = self.tx.lock().unwrap();
        tx.send(TreeViewCtrl::End)
            .expect("Failed to end");
    }
}
