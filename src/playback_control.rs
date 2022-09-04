use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::utils;

mod imp {
    use super::*;
    use gtk::{Box, Button, Scale};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/bilibili/music/playback-control.ui")]
    pub struct PlaybackControl {
        #[template_child]
        pub playback_box: TemplateChild<Box>,
        #[template_child]
        pub backward_btn: TemplateChild<Button>,
        #[template_child]
        pub forward_btn: TemplateChild<Button>,
        #[template_child]
        pub pause_btn: TemplateChild<Button>,
        #[template_child]
        pub seek: TemplateChild<Scale>,
        #[template_child]
        pub elapsed_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackControl {
        const NAME: &'static str = "PlaybackControl";
        type Type = super::PlaybackControl;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("playbackcontrol");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackControl {
        /*
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
        */
    }

    impl WidgetImpl for PlaybackControl {}
}

glib::wrapper! {
    pub struct PlaybackControl(ObjectSubclass<imp::PlaybackControl>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for PlaybackControl {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create PlaybackControl")
    }
}

impl PlaybackControl {
    pub fn pause_btn(&self) -> gtk::Button {
        self.imp().pause_btn.get()
    }

    pub fn set_elapsed(&self, elapsed: Option<u64>) {
        if let Some(elapsed) = elapsed {
            self.imp()
                .elapsed_label
                .set_text(&utils::format_time(elapsed));
        } else {
            self.imp().elapsed_label.set_text("0:00");
        }
    }
}
