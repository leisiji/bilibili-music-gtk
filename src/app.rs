use anyhow::Ok;
use gtk::{prelude::*, Application, ApplicationWindow, TreeView};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::music::{
    data::SongCollection,
    model::PlayListModel,
    utils::Player
};

pub(crate) struct App {
    rt: Arc<Runtime>,
}

impl App {
    pub(crate) fn new() -> Arc<Self> {
        let rt = Arc::new(Runtime::new().unwrap());
        let app = App { rt };
        Arc::new(app)
    }

    pub(crate) fn init(app: Arc<Self>, application: &gtk::Application) {
        let glade_src = include_str!("../ui/window.ui");
        let builder = gtk::Builder::from_string(glade_src);
        let tree: TreeView = builder.object("music_list").unwrap();
        let playlist = Arc::new(PlayListModel::new(&tree));

        let window: ApplicationWindow = builder.object("app_win").unwrap();
        window.set_application(Some(application));

        let player = Player::new(&app.rt, &builder);
        tree.connect_row_activated(move |tree, _path, _col| {
            if let Some((model, iter)) = tree.selection().selected() {
                let cur: usize = model.get(&iter, 2).get::<u32>().unwrap().try_into().unwrap();
                player.down_play(cur);
            }
        });

        let list = playlist.clone();
        app.rt.spawn(async move {
            let collection = SongCollection::new(&String::from("BV135411V7A5"));
            collection.get_songs(list).await?;
            Ok(())
        });
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
