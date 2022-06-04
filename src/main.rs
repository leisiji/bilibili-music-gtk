mod app;
mod music;

fn main() {
    pretty_env_logger::init();

    glib::set_application_name("bilibili-music-gtk4");
    glib::set_prgname(Some("bilibili-music-gtk4"));
    app::run();
}
