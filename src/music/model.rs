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
    AddCollection(String, String),
}

struct TreeStore {
    store: ListStore,
    tree: TreeView,
}

struct TreeViewCtrlModel {
    collectionlist: TreeStore,
    playlist: TreeStore,
}

impl PlayListModel {
    fn handle_ctrl(&self, ctrl: &TreeViewCtrl, model: &Arc<TreeViewCtrlModel>) {
        match ctrl {
            TreeViewCtrl::AddSong((index, song)) => {
                self.add_song_(index, song, &model.playlist.store)
            }
            TreeViewCtrl::AddCollection(title, bvid) => {
                self.add_collection_(title, bvid, &model.collectionlist.store)
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

    fn connect_update_playlist(tree_model: &Arc<TreeViewCtrlModel>, playlist_model: &Arc<Self>) {
        let playlist_model = playlist_model.clone();
        let tree_model_strong = tree_model.clone();
        tree_model.collectionlist.tree.connect_row_activated(move |tree, _path, _col| {
            if let Some((model, iter)) = tree.selection().selected() {
                let bvid: String = model.get(&iter, 1).get::<String>().unwrap();
                let collectionlist = playlist_model.collectionlist.lock().unwrap();
                let collection = collectionlist.get_collection(&bvid).unwrap();
                let store = &tree_model_strong.playlist.store;
                store.clear();
                let mut index: u32 = 0;
                for song in collection {
                    playlist_model.add_song_(&index, song, store);
                    index = index + 1;
                }
            }
        });
    }

    fn init_collection_tree(builder: &Builder) -> TreeStore {
        let tree: TreeView = builder.object("collectionlist").unwrap();
        let store = ListStore::new(&[String::static_type(), String::static_type()]);

        tree.set_model(Some(&store));

        let title = CellRendererText::new();
        title.set_width(500);
        let col = TreeViewColumn::builder()
            .sizing(gtk::TreeViewColumnSizing::Fixed)
            .build();
        col.pack_start(&title, true);
        col.add_attribute(&title, "text", 0);
        tree.append_column(&col);

        /* init the first collection that contians all songs */
        let collection = String::from("所有歌曲");
        let bvid = String::from("all");
        let iter = store.append();
        store.set(&iter, &[(0, &collection), (1, &bvid)]);

        TreeStore { store, tree }
    }

    fn init_playlist_tree(
        builder: &Builder,
        rt: &Arc<Runtime>,
        collectionlist: &Arc<Mutex<CollectionList>>,
    ) -> TreeStore {
        let tree: TreeView = builder.object("playlist").unwrap();

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

        let player = Player::new(rt, builder, collectionlist);
        let p = player.clone();
        tree.connect_row_activated(move |tree, _path, _col| {
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

        TreeStore { store, tree }
    }

    pub fn new(builder: &Builder) -> Arc<Self> {
        let collectionlist = CollectionList::new();
        let rt = Arc::new(Runtime::new().unwrap());
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let playlist_model = Arc::new(PlayListModel {
            tx,
            collectionlist: collectionlist.clone(),
            rt: rt.clone(),
        });

        let playlist = Self::init_playlist_tree(builder, &rt, &collectionlist);
        let collectionlist = Self::init_collection_tree(builder);

        let treeview_ctl_model = Arc::new(TreeViewCtrlModel {
            collectionlist,
            playlist,
        });

        Self::connect_update_playlist(&treeview_ctl_model, &playlist_model);

        let model = playlist_model.clone();
        rx.attach(None, move |ctrl| {
            model.handle_ctrl(&ctrl, &treeview_ctl_model);
            glib::Continue(true)
        });

        playlist_model
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
            .send(TreeViewCtrl::AddCollection(title, bvid.clone()))
            .expect("Failed to add collection");
    }

    fn add_collection_(&self, title: &String, bvid: &String, store: &ListStore) {
        let iter = store.append();
        store.set(&iter, &[(0, &title), (1, &bvid)])
    }

    fn add_song_(&self, index: &u32, song: &Song, store: &ListStore) {
        let iter = store.append();
        let duration = format!("{:0>2}:{:0>2}", song.duration / 60, song.duration % 60);
        store.set(&iter, &[(0, &song.name), (1, &duration), (2, index)]);
    }
}
