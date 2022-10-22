use gtk::{gio, glib, prelude::*, subclass::prelude::*, SingleSelection};

use crate::{
    audio::{Queue, Song, SongData},
    song_row::SongRow,
};

mod imp {
    use super::*;
    use gtk::{CompositeTemplate, TemplateChild};

    // BvidInputView
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/bilibili/music/bvid-input-view.ui")]
    pub struct BvidInputView {
        #[template_child]
        pub confirm_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub bv_input: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BvidInputView {
        const NAME: &'static str = "BvidInputView";
        type Type = super::BvidInputView;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("bvibvidinputview");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BvidInputView {}
    impl WidgetImpl for BvidInputView {}
    impl PopoverImpl for BvidInputView {}

    // SongListView
    #[derive(CompositeTemplate)]
    #[template(resource = "/org/bilibili/music/songlist.ui")]
    pub struct SongListView {
        #[template_child]
        pub songs_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub confirm: TemplateChild<gtk::Button>,
        #[template_child]
        pub cancel: TemplateChild<gtk::Button>,
        pub queue: Queue,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for SongListView {
        const NAME: &'static str = "SongListView";
        type Type = super::SongListView;
        type ParentType = gtk::Dialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("songlistview");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            let queue = Queue::default();

            Self {
                songs_view: TemplateChild::default(),
                confirm: TemplateChild::default(),
                cancel: TemplateChild::default(),
                queue,
            }
        }
    }

    impl ObjectImpl for SongListView {}
    impl WidgetImpl for SongListView {}
    impl WindowImpl for SongListView {}
    impl DialogImpl for SongListView {}
}

glib::wrapper! {
    pub struct BvidInputView(ObjectSubclass<imp::BvidInputView>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for BvidInputView {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create BvidInputView")
    }
}

impl BvidInputView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn confirm_btn(&self) -> gtk::Button {
        self.imp().confirm_btn.get()
    }

    pub fn get_input_bvid(&self) -> String {
        let buffer = self.imp().bv_input.buffer();
        let bvid = buffer.text();
        buffer.delete_text(0, None);
        self.set_visible(false);
        bvid
    }
}

// SongListView
glib::wrapper! {
    pub struct SongListView(ObjectSubclass<imp::SongListView>)
        @extends gtk::Widget, gtk::Dialog, gtk::Window,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SongListView {
    pub fn init(&self, songs: Vec<SongData>) {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            let row = SongRow::default();
            list_item.set_child(Some(&row));
            list_item
                .bind_property("item", &row, "song")
                .flags(glib::BindingFlags::DEFAULT)
                .build();
            list_item
                .property_expression("item")
                .chain_property::<Song>("title")
                .bind(&row, "song-title", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("selected")
                .bind(&row, "selected", gtk::Widget::NONE);
        });
        let view = self.imp().songs_view.get();
        view.set_factory(Some(&factory));

        let queue = self.queue();
        let selection_model = SingleSelection::new(Some(queue.model()));
        selection_model.set_can_unselect(false);
        selection_model.set_selected(gtk::INVALID_LIST_POSITION);
        view.set_model(Some(&selection_model));

        view.connect_activate(gtk::glib::clone!(@weak queue => move |_, pos| {
            queue.select_song_at(pos);
        }));

        self.queue().init(songs);
    }

    pub fn new(parent: &gtk::Window) -> Self {
        glib::Object::new(&[("transient-for", parent)]).expect("Failed to create SongListView")
    }

    pub fn queue(&self) -> &Queue {
        &self.imp().queue
    }

    pub fn confirm_btn(&self) -> gtk::Button {
        self.imp().confirm.get()
    }

    pub fn cancel_btn(&self) -> gtk::Button {
        self.imp().cancel.get()
    }

    pub fn selected_songs(&self) -> Option<Vec<Song>> {
        let queue = self.queue();
        if queue.is_empty() {
            return None;
        }

        let mut data = Vec::new();
        for i in 0..queue.n_songs() {
            if let Some(song) = queue.song_at(i) {
                if song.selected() {
                    data.push(song);
                }
            }
        }
        Some(data)
    }
}
