use anyhow::Ok;
use gtk::{prelude::*, Application, ApplicationWindow, TreeView};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::music::{collectionlist::CollectionList, model::PlayListModel, utils::Player};

pub(crate) struct App {
    rt: Arc<Runtime>,
    collection_list: CollectionList,
}

impl App {
    pub(crate) fn new() -> Arc<Self> {
        let rt = Arc::new(Runtime::new().unwrap());
        let collection_list = CollectionList::new();
        let app = App {
            rt,
            collection_list,
        };
        Arc::new(app)
    }

    pub(crate) fn init(app: Arc<Self>, application: &gtk::Application) {
        let glade_src = include_str!("../ui/window.ui");
        let builder = gtk::Builder::from_string(glade_src);
        let tree: TreeView = builder.object("music_list").unwrap();

        let window: ApplicationWindow = builder.object("app_win").unwrap();
        window.set_application(Some(application));

        let first_playlist = app.collection_list.get(None).unwrap();

        let player = Player::new(&app.rt, &builder, first_playlist);
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

        let playlist_model = PlayListModel::new(&tree, first_playlist);
        PlayListModel::init(&playlist_model, &app.rt);
    }

    pub(crate) fn run() {
        let application = Application::builder()
            .application_id("com.github.leisiji.bilibili-music-gtk4")
            .build();

        let weak_application = application.downgrade();
        application.connect_activate(move |_| {
            if let Some(application) = weak_application.upgrade() {
                let app = App::new();
                Self::init(app, &application);
            }
        });

        application.run();
    }
}
