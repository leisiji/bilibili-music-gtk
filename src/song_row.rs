use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

mod imp {
    use std::cell::RefCell;

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString};

    use crate::audio::Song;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/bilibili/music/song-row.ui")]
    pub struct SongRow {
        #[template_child]
        pub song_title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub selected_button: TemplateChild<gtk::CheckButton>,
        pub song: RefCell<Option<Song>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongRow {
        const NAME: &'static str = "SongRow";
        type Type = super::SongRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_css_name("songrow");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SongRow {
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
                    ParamSpecString::new("song-title", "", "", None, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("selected", "", "", true, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }
        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "song" => self.song.borrow().to_value(),
                "song-title" => self.song_title_label.label().to_value(),
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
                "selected" => {
                    let p = value.get::<bool>().expect("selected needs to be a boolean");
                    self.selected_button.set_active(p);
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.init_widgets();
        }
    }

    impl WidgetImpl for SongRow {}
}

glib::wrapper! {
    pub struct SongRow(ObjectSubclass<imp::SongRow>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for SongRow {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create queuerow")
    }
}

impl SongRow {
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
    }
}
