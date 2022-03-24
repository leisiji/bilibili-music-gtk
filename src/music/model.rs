use super::config::{self, PLAYLIST};
use super::data::Song;
use glib::StaticType;
use gtk::prelude::*;
use gtk::{prelude::CellLayoutExt, CellRendererText, ListStore, TreeView, TreeViewColumn};

pub(crate) struct PlayListModel {
    tx: glib::Sender<TreeViewCtrl>,
}

pub(crate) enum TreeViewCtrl {
    StartRefresh,
    Add(Song),
    EndRefresh,
}

impl PlayListModel {
    fn add_song(song: &Song, store: &ListStore) {
        let iter = store.append();
        let duration = format!(
            "{:0>2}:{:0>2}",
            song.duration / 60,
            song.duration % 60
        );
        let index: u32 = PLAYLIST.lock().unwrap().list.len().try_into().unwrap();
        store.set(
            &iter,
            &[(0, &song.name), (1, &duration), (2, &song.play_url), (3, &index)],
        );
    }

    pub fn new(tree: &TreeView) -> Self {
        // name, duration, play_url, cur list index
        let store = ListStore::new(&[
            String::static_type(),
            String::static_type(),
            String::static_type(),
            u32::static_type(),
        ]);

        tree.set_model(Some(&store));

        let song_name = CellRendererText::new();
        song_name.set_width(500);
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
                TreeViewCtrl::Add(song) => Self::add_song(&song, &store),
                TreeViewCtrl::StartRefresh => todo!(),
                TreeViewCtrl::EndRefresh => todo!(),
            };
            glib::Continue(true)
        });

        PlayListModel { tx }
    }

    pub fn add(&self, song: Song) {
        self.tx.send(TreeViewCtrl::Add(song.clone())).expect("Failed to add song");
        let mut playlist = PLAYLIST.lock().unwrap();
        playlist.list.push(song);
    }

    pub fn start_refresh(&self) {
        self.tx.send(TreeViewCtrl::StartRefresh).expect("Failed to start");
    }

    pub fn end_refresh(&self) {
        self.tx.send(TreeViewCtrl::EndRefresh).expect("Failed to end");
    }
}
