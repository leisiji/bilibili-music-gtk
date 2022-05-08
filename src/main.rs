mod app;
mod music;

fn main() {
    glib::set_application_name("bilibili-music-gtk4");
    glib::set_prgname(Some("bilibili-music-gtk4"));
    app::run();
}
