use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate, ListView};

mod imp {
    use super::*;
    use gtk::ListView;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/bilibili/music/playlist.ui")]
    pub struct PlayListView {
        #[template_child]
        pub queue_view: TemplateChild<ListView>,
        #[template_child]
        pub queue_actionbar: TemplateChild<gtk::ActionBar>,
        #[template_child]
        pub queue_remove_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub queue_selected_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub scroll_win: TemplateChild<gtk::ScrolledWindow>,
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

    impl ObjectImpl for PlayListView {
        /*
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
        */
    }
    impl WidgetImpl for PlayListView {}
}

glib::wrapper! {
    pub struct PlayListView(ObjectSubclass<imp::PlayListView>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for PlayListView {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create PlayListView")
    }
}

impl PlayListView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_view(&self) -> ListView {
        self.imp().queue_view.get()
    }

    pub fn queue_actionbar(&self) -> gtk::ActionBar {
        self.imp().queue_actionbar.get()
    }

    pub fn queue_remove_button(&self) -> gtk::Button {
        self.imp().queue_remove_button.get()
    }

    pub fn queue_selected_label(&self) -> gtk::Label {
        self.imp().queue_selected_label.get()
    }

    pub fn scroll_adjust(&self) -> gtk::Adjustment {
        let win = self.imp().scroll_win.get();
        win.vadjustment()
    }
}
