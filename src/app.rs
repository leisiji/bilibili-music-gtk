use anyhow::Ok;
use glib::clone;
use gtk::{prelude::*, Application, ApplicationWindow, TreeView};
use std::sync::Arc;
use tokio::{runtime::Runtime, sync::Mutex};

use crate::music::{data::{SongCollection, Song}, model::PlayList, utils::Player};

pub(crate) struct App {
    playlist: Arc<PlayList>,
    player: Arc<Player>,
    rt: Runtime,
}

impl App {
    pub(crate) fn new(application: &gtk::Application) -> Self {
        let glade_src = include_str!("../ui/window.ui");
        let builder = gtk::Builder::from_string(glade_src);
        let tree = TreeView::new();
        let playlist = PlayList::new(&tree);

        let window: ApplicationWindow = builder
            .object("applicationwindow")
            .expect("Couldn't get window");
        window.set_application(Some(application));
        window.set_child(Some(&tree));

        let player = Player::new();
        let rt = Runtime::new().unwrap();

        App { playlist: Arc::new(playlist), player, rt }
    }

    pub(crate) fn init(app: &Self) {
        let playlist = app.playlist.clone();
        let song = Song {
            name: String::from("hello"),
            play_url: String::from("sdasda"),
            duration: 12,
        };
        playlist.add(song);
        /*
        app.rt.spawn(async move {
            let collection = SongCollection::new(&String::from("BV135411V7A5"));
            collection.get_songs(|song| {
                println!("{:?}", song);
                playlist.add(song);
            }).await?;
            Ok(())
        });
        */
    }

    pub(crate) fn run() {
        let application = Application::builder()
            .application_id("com.github.leisiji.bilibili-music-gtk4")
            .build();
        application.connect_startup(clone!(@weak application => @default-panic, move |_|{
            let app = Self::new(&application);
            application.connect_activate(move |_| {
                Self::init(&app);
            });
        }));
        application.run();
    }
}
