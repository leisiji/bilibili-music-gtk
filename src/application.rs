use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::config::APPLICATION_ID;

mod imp {

    use gstreamer_player::prelude::Cast;
    use gtk::traits::GtkApplicationExt;

    use crate::window::Window;

    use super::*;

    #[derive(Default)]
    pub struct Application {}

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "BiliBiliMusicApp";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_accels_for_action("win.next", &["n"]);
            obj.set_accels_for_action("win.previous", &["p"]);
            obj.set_accels_for_action("win.scroll_to_start", &["g"]);
            obj.set_accels_for_action("win.scroll_to_end", &["<Shift>g"]);
            obj.set_accels_for_action("win.half_page_up", &["u"]);
            obj.set_accels_for_action("win.half_page_down", &["d"]);
        }
    }

    impl ApplicationImpl for Application {
        fn startup(&self, application: &Self::Type) {
            self.parent_startup(application);
        }

        fn activate(&self, application: &Self::Type) {
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = Window::new(application);
                window.upcast()
            };
            window.present();
        }

        fn open(&self, application: &Self::Type, _files: &[gio::File], _hint: &str) {
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = Window::new(application);
                window.upcast()
            };
            window.present();
        }
    }

    impl AdwApplicationImpl for Application {}
    impl GtkApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::new(&[("application-id", &APPLICATION_ID)])
            .expect("Failed to create Application")
    }
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }
}
