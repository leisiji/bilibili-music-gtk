use anyhow::Ok;
use gtk::{prelude::*, Application, ApplicationWindow, TreeView};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::music::{
    data::{download_song, SongCollection},
    model::PlayListModel,
    utils::Player, config::PLAYLIST,
};

pub(crate) struct App {
    player: Arc<Player>,
    rt: Arc<Runtime>,
}

impl App {
    pub(crate) fn new() -> Arc<Self> {
        let rt = Arc::new(Runtime::new().unwrap());
        let player = Player::new(&rt);

        let app = App { player, rt };
        Arc::new(app)
    }

    pub(crate) fn init(app: Arc<Self>, application: &gtk::Application) {
        let glade_src = include_str!("../ui/window.ui");
        let builder = gtk::Builder::from_string(glade_src);
        let tree: TreeView = builder.object("music_list").unwrap();
        let playlist = Arc::new(PlayListModel::new(&tree));

        let window: ApplicationWindow = builder.object("app_win").unwrap();
        window.set_application(Some(application));

        let strong_app = app.clone();
        let runtime = app.rt.clone();
        tree.connect_row_activated(move |tree, _path, _col| {
            if let Some((model, iter)) = tree.selection().selected() {
                let play_url = model.get(&iter, 2).get::<String>().unwrap();
                let name = model.get(&iter, 0).get::<String>().unwrap();
                let cur = model.get(&iter, 3).get::<u32>().unwrap();
                PLAYLIST.lock().unwrap().cur = cur;
                let player = strong_app.player.clone();
                runtime.spawn(async move {
                    let s = download_song(&play_url, name.as_str()).await?;
                    player.play(s.as_str());
                    Ok(())
                });
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
