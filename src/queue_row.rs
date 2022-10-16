use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

mod imp {
    use std::cell::{Cell, RefCell};

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString};

    use crate::audio::Song;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/bilibili/music/queue-row.ui")]
    pub struct QueueRow {
        #[template_child]
        pub row_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub song_title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub song_artist_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub selection_artist_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub selection_title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub selected_button: TemplateChild<gtk::CheckButton>,

        pub song: RefCell<Option<Song>>,
        pub playing: Cell<bool>,
        pub selection_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QueueRow {
        const NAME: &'static str = "BiliBiliQueueRow";
        type Type = super::QueueRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_css_name("queuerow");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for QueueRow {
        /*
        fn dispose(&self, _obj: &Self::Type) {
            self.row_stack.unparent();
        }
        */

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.init_widgets();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "song",
                        "",
                        "",
                        Song::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new("song-artist", "", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("song-title", "", "", None, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("playing", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("selection-mode", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("selected", "", "", false, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "song" => self.song.borrow().to_value(),
                "song-title" => self.song_title_label.label().to_value(),
                "song-artist" => self.song_artist_label.label().to_value(),
                "playing" => self.playing.get().to_value(),
                "selection-mode" => self.selection_mode.get().to_value(),
                "selected" => self.selected_button.is_active().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "song" => {
                    let song = value.get::<Option<Song>>().unwrap();
                    self.song.replace(song);
                }
                "song-title" => {
                    let p = value
                        .get::<&str>()
                        .expect("song-title needs to be a string");
                    obj.set_song_title(p);
                }
                "song-artist" => {
                    let p = value
                        .get::<&str>()
                        .expect("song-artist needs to be a string");
                    obj.set_song_artist(p);
                }
                "playing" => {
                    let p = value.get::<bool>().expect("playing needs to be a boolean");
                    obj.set_playing(p);
                }
                "selection-mode" => {
                    let p = value
                        .get::<bool>()
                        .expect("selection-mode needs to be a boolean");
                    obj.set_selection_mode(p);
                }
                "selected" => {
                    let p = value.get::<bool>().expect("selected needs to be a boolean");
                    self.selected_button.set_active(p);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for QueueRow {}
}

glib::wrapper! {
    pub struct QueueRow(ObjectSubclass<imp::QueueRow>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for QueueRow {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create queuerow")
    }
}

impl QueueRow {
    fn init_widgets(&self) {
        self.imp().selected_button.connect_active_notify(
            clone!(@strong self as this => move |button| {
                if let Some(ref song) = *this.imp().song.borrow() {
                    song.set_selected(button.is_active());
                }
                this.notify("selected");
            }),
        );
    }

    fn set_song_title(&self, title: &str) {
        let imp = self.imp();
        imp.song_title_label.set_label(title);
        imp.selection_title_label.set_label(title);
    }

    fn set_song_artist(&self, artist: &str) {
        let imp = self.imp();
        imp.song_artist_label.set_label(artist);
        imp.selection_artist_label.set_label(artist);
    }

    fn set_playing(&self, playing: bool) {
        if playing != self.imp().playing.replace(playing) {
            self.update_mode();
            self.notify("playing");
        }
    }

    fn set_selection_mode(&self, selection_mode: bool) {
        if selection_mode != self.imp().selection_mode.replace(selection_mode) {
            self.update_mode();
            self.notify("selection-mode");
        }
    }

    fn update_mode(&self) {
        let imp = self.imp();
        if imp.selection_mode.get() {
            imp.row_stack.set_visible_child_name("selection-mode");
        } else if imp.playing.get() {
            imp.row_stack.set_visible_child_name("currently-playing");
        } else {
            imp.row_stack.set_visible_child_name("song-details");
        }
    }
}
