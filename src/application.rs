use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::config::APPLICATION_ID;

mod imp {

    use gstreamer_player::prelude::Cast;
    use gtk::traits::GtkApplicationExt;

    use crate::window::{self, Window};

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
            window.downcast::<Window>().unwrap().setup_playlist();
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
