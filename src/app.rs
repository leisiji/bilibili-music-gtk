use anyhow::Ok;
use gtk::{prelude::*, Application, ApplicationWindow, TreeView};
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::music::{
    data::{download_song, SongCollection},
    model::PlayList,
    utils::Player,
};

pub(crate) struct App {
    player: Arc<Player>,
}

impl App {
    pub(crate) fn new() -> Rc<Self> {
        let player = Player::new();

        let app = App { player };
        Rc::new(app)
    }

    pub(crate) fn init(app: Rc<Self>, application: &gtk::Application) {
        let glade_src = include_str!("../ui/window.ui");
        let builder = gtk::Builder::from_string(glade_src);
        let tree: TreeView = builder.object("music_list").unwrap();
        let rt = Arc::new(Runtime::new().unwrap());

        let window: ApplicationWindow = builder.object("app_win").unwrap();
        window.set_application(Some(application));

        let strong_app = Rc::clone(&app);
        let runtime = rt.clone();
        tree.connect_row_activated(move |tree, _path, _col| {
            if let Some((model, iter)) = tree.selection().selected() {
                let play_url = model.get(&iter, 2).get::<String>().unwrap();
                let name = model.get(&iter, 0).get::<String>().unwrap();
                let player = strong_app.player.clone();
                runtime.spawn(async move {
                    let s = download_song(&play_url, name.as_str()).await?;
                    player.play(s.as_str());
                    Ok(())
                });
            }
        });

        let list = Arc::new(PlayList::new(tree));
        rt.spawn(async move {
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
