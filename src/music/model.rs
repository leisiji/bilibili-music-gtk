use std::sync::{Arc, Mutex};

use super::collectionlist::CollectionList;
use super::config::parse_config;
use super::data::{Song, SongCollection};
use super::utils::Player;
use anyhow::Ok;
use glib::StaticType;
use gtk::{prelude::CellLayoutExt, CellRendererText, ListStore, TreeView, TreeViewColumn};
use gtk::{prelude::*, Builder};
use tokio::runtime::Runtime;

pub(crate) struct PlayListModel {
    tx: glib::Sender<TreeViewCtrl>,
    collectionlist: Arc<Mutex<CollectionList>>,
    rt: Arc<Runtime>,
}

pub(crate) enum TreeViewCtrl {
    AddSong((u32, Song)),
    AddCollection(String),
}

struct TreeViewCtrlStore {
    collectionlist_store: ListStore,
    playlist_store: ListStore,
}

impl PlayListModel {
    fn handle_ctrl(&self, ctrl: &TreeViewCtrl, store: &TreeViewCtrlStore) {
        match ctrl {
            TreeViewCtrl::AddSong((index, song)) => {
                self.add_song_(index, song, &store.playlist_store)
            }
            TreeViewCtrl::AddCollection(title) => {
                self.add_collection_(title, &store.collectionlist_store)
            }
        };
    }

    pub fn init(playlist_model: &Arc<Self>) {
        let config = parse_config().unwrap();
        for bv in config.bv_list {
            let model = playlist_model.clone();
            playlist_model.rt.spawn(async move {
                let collection = SongCollection::new(bv.bvid.as_str());
                collection.get_songs(&model).await?;
                Ok(())
            });
        }
    }

    fn init_collection_tree(_playlist_model: &Arc<Self>, builder: &Builder) -> ListStore {
        let collectionlist_tree: TreeView = builder.object("collectionlist").unwrap();

        let store = ListStore::new(&[String::static_type()]);

        collectionlist_tree.set_model(Some(&store));

        let title = CellRendererText::new();
        title.set_width(500);
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&title, true);
        col.add_attribute(&title, "text", 0);
        collectionlist_tree.append_column(&col);

        /* init the first collection that contians all songs */
        let collection = String::from("所有歌曲");
        let iter = store.append();
        store.set(&iter, &[(0, &collection)]);

        store
    }

    fn init_playlist_tree(
        builder: &Builder,
        rt: &Arc<Runtime>,
        collectionlist: &Arc<Mutex<CollectionList>>,
    ) -> ListStore {
        let playlist_tree: TreeView = builder.object("playlist").unwrap();

        // name, duration, cur list index
        let playlist_store = ListStore::new(&[
            String::static_type(),
            String::static_type(),
            u32::static_type(),
        ]);

        playlist_tree.set_model(Some(&playlist_store));

        let song_name = CellRendererText::new();
        song_name.set_width(500);
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&song_name, true);
        col.add_attribute(&song_name, "text", 0);
        playlist_tree.append_column(&col);

        let duration = CellRendererText::new();
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&duration, true);
        col.add_attribute(&duration, "text", 1);
        playlist_tree.append_column(&col);

        let player = Player::new(rt, builder, collectionlist);
        let p = player.clone();
        playlist_tree.connect_row_activated(move |tree, _path, _col| {
            if let Some((model, iter)) = tree.selection().selected() {
                let cur: usize = model
                    .get(&iter, 2)
                    .get::<u32>()
                    .unwrap()
                    .try_into()
                    .unwrap();
                p.down_play(cur);
            }
        });

        playlist_store
    }

    pub fn new(builder: &Builder) -> Arc<Self> {
        let collectionlist = CollectionList::new();
        let rt = Arc::new(Runtime::new().unwrap());
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let playlistmodel = Arc::new(PlayListModel {
            tx,
            collectionlist: collectionlist.clone(),
            rt: rt.clone(),
        });

        let playlist_store = Self::init_playlist_tree(builder, &rt, &collectionlist);
        let collectionlist_store = Self::init_collection_tree(&playlistmodel, builder);

        let store = TreeViewCtrlStore {
            collectionlist_store,
            playlist_store,
        };

        let model = playlistmodel.clone();
        rx.attach(None, move |ctrl| {
            model.handle_ctrl(&ctrl, &store);
            glib::Continue(true)
        });

        playlistmodel
    }

    pub fn add_song(&self, bvid: String, song: Song) {
        let index: u32;
        {
            let mut collectionlist = self.collectionlist.lock().unwrap();
            index = collectionlist.get_collection_size() as u32;
            collectionlist.add_song(&bvid, &song);
        }
        self.tx
            .send(TreeViewCtrl::AddSong((index, song)))
            .expect("Failed to add song");
    }

    pub fn add_collection(&self, bvid: &String, title: String) {
        {
            let mut collectionlist = self.collectionlist.lock().unwrap();
            collectionlist.add_collection(&bvid);
        }
        self.tx
            .send(TreeViewCtrl::AddCollection(title))
            .expect("Failed to add collection");
    }

    fn add_collection_(&self, title: &String, store: &ListStore) {
        let iter = store.append();
        store.set(&iter, &[(0, &title)])
    }

    fn add_song_(&self, index: &u32, song: &Song, store: &ListStore) {
        let iter = store.append();
        let duration = format!("{:0>2}:{:0>2}", song.duration / 60, song.duration % 60);
        store.set(&iter, &[(0, &song.name), (1, &duration), (2, index)]);
    }
}
