use adw::subclass::prelude::*;
use glib::clone;
use gtk::{
    gdk, gio,
    glib::{self, MainContext},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate, SingleSelection,
};

use crate::audio::{PlayerAction, Song};
use crate::{audio::SongData, queue_row::QueueRow};

mod imp {
    use crate::{audio::AudioPlayer, playback_control::PlaybackControl, playlist::PlayListView};
    use gtk::MenuButton;
    use std::rc::Rc;

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
        pub provider: gtk::CssProvider,
        pub context: MainContext,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "BiliBiliMusicWindow";
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
                provider: gtk::CssProvider::new(),
                context: MainContext::default(),
            }
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_playlist();
            obj.setup_provider();
            obj.restore_window_state();
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

    fn init_playlist(&self) {
        let tx = self.imp().player.tx.clone();
        self.imp().context.spawn(async move {
            let data = SongData::from_bvid("BV1qf4y1d7d1");
            if let Ok(data) = data {
                tx.send(PlayerAction::AddSong(data)).unwrap();
            }
        });
    }

    fn setup_playlist(&self) {
        let imp = self.imp();

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(clone!(@strong self as win => move |_, list_item| {
            let row = QueueRow::default();
            list_item.set_child(Some(&row));

            /*
            row.connect_notify_local(
                Some("selected"),
                clone!(@weak win => move |_, _| {
                    win.update_selected_count();
                }),
            );
            win
                .bind_property("playlist-selection", &row, "selection-mode")
                .flags(glib::BindingFlags::DEFAULT)
                .build();
            */

            /* 将 list_item 的 item 属性绑定 song 属性 */
            list_item
                .bind_property("item", &row, "song")
                .flags(glib::BindingFlags::DEFAULT)
                .build();

            list_item
                .property_expression("item")
                .chain_property::<Song>("artist")
                .bind(&row, "song-artist", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("title")
                .bind(&row, "song-title", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("playing")
                .bind(&row, "playing", gtk::Widget::NONE);
            list_item
                .property_expression("item")
                .chain_property::<Song>("selected")
                .bind(&row, "selected", gtk::Widget::NONE);
        }));
        let queue_view = imp.playlist_view.queue_view();
        queue_view.set_factory(Some(&factory));

        let queue = imp.player.queue();
        let selection_model = SingleSelection::new(Some(queue.model()));
        selection_model.set_can_unselect(false);
        selection_model.set_selected(gtk::INVALID_LIST_POSITION);
        queue_view.set_model(Some(&selection_model));

        self.init_playlist();
    }

    fn setup_provider(&self) {
        let imp = self.imp();
        if let Some(display) = gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(&display, &imp.provider, 400);
        }
    }

    fn restore_window_state(&self) {
        self.set_default_size(600, -1);
    }
}
