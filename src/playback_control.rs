use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::{utils, volume_control::VolumeControl};

mod imp {
    use crate::volume_control::VolumeControl;

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
        #[template_child]
        pub play_time_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub volume_control: TemplateChild<VolumeControl>,
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

    pub fn seek(&self) -> gtk::Scale {
        self.imp().seek.get()
    }

    pub fn set_elapsed(&self, elapsed: u64) {
        let imp = self.imp();
        imp.elapsed_label.set_text(&utils::format_time(elapsed));
        imp.seek.set_value(elapsed as f64);
    }

    pub fn set_range(&self, range: u64) {
        let imp = self.imp();
        imp.elapsed_label.set_text("0:00");
        imp.play_time_label.set_text(&utils::format_time(range));
        imp.seek.set_range(0.0, range as f64);
    }

    pub fn volume_control(&self) -> VolumeControl {
        self.imp().volume_control.get()
    }
}
