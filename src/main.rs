mod application;
mod audio;
mod config;
mod playback_control;
mod playlist;
mod window;

use application::Application;
use gtk::{gio, prelude::*};

fn main() {
    gio::resources_register_include!("bilibili-music-gtk4.gresource").unwrap();
    let app = Application::new();
    app.run();
}
