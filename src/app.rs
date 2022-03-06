use async_std::prelude::*;
use glib::{clone, MainContext};
use gtk::{prelude::*, Application, ApplicationWindow};
use std::sync::Arc;

use crate::music::model::PlayList;

pub(crate) struct App {
    playlist: PlayList,
}

impl App {
    pub(crate) fn new(application: &gtk::Application) -> Arc<Self> {
        let glade_src = include_str!("../ui/window.ui");
        let builder = gtk::Builder::from_string(glade_src);

        let playlist = PlayList::new();

        let window: ApplicationWindow = builder
            .object("applicationwindow")
            .expect("Couldn't get window");
        window.set_application(Some(application));
        window.set_child(Some(&playlist.tree));

        let app = App { playlist };
        Arc::new(app)
    }

    pub(crate) fn init(app: &Arc<Self>) {
        let dir = "/home/ye/github/BBDown_v1.4.0_20210710_linux-x64";

        let app = app.clone();
        MainContext::default().spawn_local(async move {
            let mut entries = async_std::fs::read_dir(dir)
                .await
                .expect("failed to read dir");
            while let Some(res) = entries.next().await {
                let res = res.expect("failed to get file name");
                let s = res.file_name().to_str().unwrap().to_string();
                if let Some(_is_mp3) = s.find("mp3") {
                    let file = format!("{}/{}", dir, s);
                    app.playlist.add(file);
                }
            }
        });
    }

    pub(crate) fn run() {
        let application = Application::builder()
            .application_id("bilibili-music-gtk4")
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
