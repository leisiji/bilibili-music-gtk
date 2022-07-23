use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate, SingleSelection};

use crate::audio::Song;

mod imp {
    use std::rc::Rc;

    use gtk::MenuButton;

    use crate::{audio::AudioPlayer, playback_control::PlaybackControl, playlist::PlayListView};

    use super::*;

    #[derive(CompositeTemplate)]
    #[template(resource = "/org/bilibili/music/window.ui")]
    pub struct Window {
        #[template_child]
        pub add_bv_btn: TemplateChild<MenuButton>,
        #[template_child]
        pub playlist_view: TemplateChild<PlayListView>,
        #[template_child]
        pub playback_ctl: TemplateChild<PlaybackControl>,

        pub player: Rc<AudioPlayer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "BiliBiliMusicWin";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                add_bv_btn: TemplateChild::default(),
                playlist_view: TemplateChild::default(),
                playback_ctl: TemplateChild::default(),
                player: AudioPlayer::new(),
            }
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Window {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)]).expect("Failed to create Window")
    }

    pub fn setup_playlist(&self) {
        let imp = self.imp();
        let selection = SingleSelection::new(Some(imp.player.queue().model()));
        selection.set_can_unselect(false);
        selection.set_selected(gtk::INVALID_LIST_POSITION);
        imp.playlist_view
            .queue_view()
            .set_model(Some(&selection.upcast::<gtk::SelectionModel>()));

        let song = Song::new("dasdsdsada");
        imp.player.queue().add_song(&song);
    }
}
