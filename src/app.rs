use anyhow::{Ok, Result};
use gtk::{prelude::*, Application, ApplicationWindow, TreeView};
use reqwest::header;
use std::borrow::Borrow;
use std::io::Write;
use std::sync::Arc;
use std::{fs::File, rc::Rc};
use tokio::runtime::Runtime;

use crate::music::config::{self, CACHE_DIR};
use crate::music::{data::SongCollection, model::PlayList, utils::Player};

pub(crate) struct App {
    player: Arc<Player>,
    rt: Runtime,
}

impl App {
    pub(crate) fn new() -> Rc<Self> {
        let player = Player::new();
        let rt = Runtime::new().unwrap();

        let app = App { player, rt };
        Rc::new(app)
    }

    async fn download_song(url: &str, name: &str) -> Result<String> {
        let mut headers = header::HeaderMap::default();
        headers.insert(
            header::REFERER,
            header::HeaderValue::from_static(config::BILIBILI_REFERER),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(config::BILIBILI_UA),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let response = client.get(url).send().await?;

        let path = CACHE_DIR.join(name);
        let s = path.clone().into_os_string().into_string().unwrap();

        let mut dest = { File::create(path)? };
        let buf = response.bytes().await?;
        dest.write(buf.borrow())?;

        Ok(s)
    }

    pub(crate) fn init(app: Rc<Self>, application: &gtk::Application) {
        let glade_src = include_str!("../ui/window.ui");
        let builder = gtk::Builder::from_string(glade_src);
        let tree: TreeView = builder.object("music_list").unwrap();

        let window: ApplicationWindow = builder.object("app_win").unwrap();
        window.set_application(Some(application));

        let strong_app = Rc::clone(&app);
        tree.connect_row_activated(move |tree, _path, _col| {
            if let Some((model, iter)) = tree.selection().selected() {
                let play_url = model.get(&iter, 2).get::<String>().unwrap();
                let player = strong_app.player.clone();
                let name = model.get(&iter, 0).get::<String>().unwrap();
                strong_app.rt.spawn(async move {
                    let s = Self::download_song(&play_url, name.as_str()).await?;
                    player.play(s.as_str());
                    Ok(())
                });
            }
        });

        let playlist = PlayList::new(tree);

        app.rt.spawn(async move {
            let collection = SongCollection::new(&String::from("BV135411V7A5"));
            collection
                .get_songs(|song| {
                    playlist.add(song);
                })
                .await?;
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
