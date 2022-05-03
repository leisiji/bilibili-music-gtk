use std::sync::{Arc, Mutex};

use super::config::{parse_config, PlayList};
use super::data::{Song, SongCollection};
use anyhow::Ok;
use glib::StaticType;
use gtk::prelude::*;
use gtk::{prelude::CellLayoutExt, CellRendererText, ListStore, TreeView, TreeViewColumn};
use tokio::runtime::Runtime;

pub(crate) struct PlayListModel {
    tx: glib::Sender<TreeViewCtrl>,
    playlist: Arc<Mutex<PlayList>>,
}

pub(crate) enum TreeViewCtrl {
    Add(Song),
}

impl PlayListModel {
    pub fn init(playlist_model: &Arc<Self>, rt: &Runtime) {
        let config = parse_config().unwrap();
        for bv in config.bv_list {
            let playlist_model = playlist_model.clone();
            rt.spawn(async move {
                let collection = SongCollection::new(bv.bvid.as_str());
                collection.get_songs(&playlist_model).await?;
                Ok(())
            });
        }
    }

    pub fn new(tree: &TreeView, playlist: &Arc<Mutex<PlayList>>) -> Arc<Self> {
        // name, duration, cur list index
        let store = ListStore::new(&[
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

        let playlist = playlist.clone();
        let playlistmodel = Arc::new(PlayListModel { tx, playlist });

        let model = playlistmodel.clone();
        rx.attach(None, move |ctrl| {
            match ctrl {
                TreeViewCtrl::Add(song) => model.add_song_(song, &store),
            };
            glib::Continue(true)
        });

        playlistmodel
    }

    pub fn add(&self, song: Song) {
        self.tx
            .send(TreeViewCtrl::Add(song))
            .expect("Failed to add song");
    }

    fn add_song_(&self, song: Song, store: &ListStore) {
        let iter = store.append();
        let duration = format!("{:0>2}:{:0>2}", song.duration / 60, song.duration % 60);
        let index: u32;

        {
            let mut playlist = self.playlist.lock().unwrap();
            index = playlist.list.len().try_into().unwrap();
            playlist.list.push(song.clone());
        }

        store.set(&iter, &[(0, &song.name), (1, &duration), (2, &index)]);
    }
}
