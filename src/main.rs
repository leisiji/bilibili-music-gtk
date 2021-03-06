mod application;
mod audio;
mod config;
mod playback_control;
mod playlist;
mod queue_row;
mod window;

use application::Application;
use gtk::{gio, glib, prelude::*};

fn main() {
    gio::resources_register_include!("bilibili-music-gtk4.gresource").unwrap();

    glib::set_application_name("BiliBili");
    glib::set_program_name(Some("BiliBili"));

    let app = Application::new();
    app.run();
}
