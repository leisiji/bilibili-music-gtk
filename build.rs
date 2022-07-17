use gtk::gio;

fn main() {
    gio::compile_resources(
        "ui/",
        "ui/resources.gresource.xml",
        "bilibili-music-gtk4.gresource",
    );
}
