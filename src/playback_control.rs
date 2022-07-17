use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use gtk::{Button, Scale, Box};

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
        pub seek: TemplateChild<Scale>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackControl {
        const NAME: &'static str = "BiliBiliPlaybackControl";
        type Type = super::PlaybackControl;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackControl {}
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
