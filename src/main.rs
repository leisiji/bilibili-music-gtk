mod application;
mod audio;
mod bilibili;
mod config;
mod playback_control;
mod playlist_view;
mod queue_row;
mod song_row;
mod utils;
mod volume_control;
mod window;

use application::Application;
use gtk::{gio, glib, prelude::*};

fn main() {
    pretty_env_logger::init();
    gio::resources_register_include!("bilibili-music-gtk4.gresource").unwrap();

    glib::set_application_name("BiliBili");
    glib::set_program_name(Some("BiliBili"));

    let app = Application::new();
    app.run();
}
