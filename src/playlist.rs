use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate, ListView};

mod imp {
    use super::*;
    use gtk::ListView;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/bilibili/music/playlist.ui")]
    pub struct PlayListView {
        #[template_child]
        pub queue_view: TemplateChild<ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlayListView {
        const NAME: &'static str = "PlayListView";
        type Type = super::PlayListView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("playlistview");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlayListView {}
    impl WidgetImpl for PlayListView {}
}

glib::wrapper! {
    pub struct PlayListView(ObjectSubclass<imp::PlayListView>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PlayListView {
    pub fn queue_view(&self) -> ListView {
        self.imp().queue_view.get()
    }
}
