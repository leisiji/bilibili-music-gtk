use adw::StyleManager;
use gtk::{prelude::*, Application, ApplicationWindow};

use crate::music::{input::init_input, model::PlayListModel};

fn app_init(application: &gtk::Application) {
    let glade_src = include_str!("../ui/window.ui");
    let builder = gtk::Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.object("app_win").unwrap();
    window.set_application(Some(application));

    let playlist_model = PlayListModel::new(&builder);
    PlayListModel::init(&playlist_model);

    init_input(&builder, &playlist_model);
}

pub(crate) fn run() {
    let application = Application::builder()
        .application_id("com.github.leisiji.bilibili-music-gtk4")
        .build();

    let weak_application = application.downgrade();

    application.connect_startup(|_| {
        adw::init();
        StyleManager::default().set_color_scheme(adw::ColorScheme::PreferDark);
    });

    application.connect_activate(move |_| {
        if let Some(application) = weak_application.upgrade() {
            app_init(&application);
        }
    });

    application.run();
}
