use adw::subclass::prelude::*;
use glib::clone;
use gtk::{
    gdk, gio,
    glib::{self, MainContext},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate, SingleSelection,
};

use crate::audio::{PlayerAction, Song, SongData};
use crate::queue_row::QueueRow;

mod imp {
    use crate::{
        audio::AudioPlayer, bilibili::BvidInputView, playback_control::PlaybackControl,
        playlist_view::PlayListView,
    };
    use std::{cell::Cell, rc::Rc};

    use super::*;

    #[derive(CompositeTemplate)]
    #[template(resource = "/org/bilibili/music/window.ui")]
    pub struct Window {
        #[template_child]
        pub bvid_input_view: TemplateChild<BvidInputView>,
        #[template_child]
        pub playlist_view: TemplateChild<PlayListView>,
        #[template_child]
        pub playback_ctl: TemplateChild<PlaybackControl>,

        pub player: Rc<AudioPlayer>,
        pub provider: gtk::CssProvider,
        pub context: MainContext,
        pub playlist_selection: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "BiliBiliMusicWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("win.play", None, move |win, _, _| {
                win.imp().player.toggle_play();
            });
            klass.install_action("win.previous", None, move |win, _, _| {
                win.imp().player.skip_previous();
            });
            klass.install_action("win.next", None, move |win, _, _| {
                win.imp().player.skip_next();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                playlist_view: TemplateChild::default(),
                playback_ctl: TemplateChild::default(),
                player: AudioPlayer::new(),
                provider: gtk::CssProvider::new(),
                context: MainContext::default(),
                playlist_selection: Cell::new(false),
                bvid_input_view: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_playlist();
            obj.bind_state();
            obj.connect_signals();
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

    fn connect_signals(&self) {
        let volume_control = self.imp().playback_ctl.volume_control();
        volume_control.connect_notify_local(
            Some("volume"),
            clone!(@weak self as win => move |control, _| {
                win.imp().player.set_volume(control.volume());
            }),
        );
    }

    // Bind the PlayerState to the UI
    fn bind_state(&self) {
        let imp = self.imp();
        let state = imp.player.state();

        // Use the PlayerState:playing property to control the play/pause button
        state.connect_notify_local(
            Some("playing"),
            clone!(@weak self as win => move |state, _| {
                win.set_playlist_selection(false);
                let pause_btn = win.imp().playback_ctl.pause_btn();
                if state.playing() {
                    pause_btn.set_icon_name("media-playback-pause-symbolic");
                } else {
                    pause_btn.set_icon_name("media-playback-start-symbolic");
                }
            }),
        );
        // Update the position label
        state.connect_notify_local(
            Some("position"),
            clone!(@weak self as win => move |state, _| {
                let elapsed = state.position();
                if elapsed == 0 {
                    if let Some(song) = state.current_song() {
                        win.imp().playback_ctl.set_range(song.duration());
                    }
                }
                win.imp().playback_ctl.set_elapsed(elapsed);
            }),
        );
        self.imp().playback_ctl.seek().connect_change_value(
            clone!(@strong self as win => move |seek, _, value| {
                let percent = value / seek.adjustment().upper();
                win.imp().player.seek(percent);
                gtk::Inhibit(true)
            }),
        );
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

        self.imp().playlist_view.queue_view().connect_activate(
            clone!(@weak self as win => move |_, pos| {
                let imp = win.imp();
                let queue = imp.player.queue();
                if win.playlist_selection() {
                    queue.select_song_at(pos);
                } else if queue.current_song_index() != Some(pos) {
                    imp.player.skip_to(pos);
                }
            }),
        );

        self.imp().bvid_input_view.confirm_btn().connect_clicked(
            clone!(@weak self as win => move |_| {
                let imp = win.imp();
                let bvid = imp.bvid_input_view.get_input_bvid();
                let tx = imp.player.tx.clone();
                win.imp().context.spawn(async move {
                    if let Ok(data) = SongData::from_bvid(bvid.as_str()) {
                        tx.send(PlayerAction::AddSong(data)).unwrap();
                    }
                });
            }),
        );
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

    fn playlist_selection(&self) -> bool {
        self.imp().playlist_selection.get()
    }

    fn set_playlist_selection(&self, selection: bool) {
        let imp = self.imp();

        if selection != imp.playlist_selection.replace(selection) {
            if !selection {
                let queue = imp.player.queue();
                queue.unselect_all_songs();
            }

            /*
            self.imp()
                .playlist_view
                .queue_actionbar()
                .set_revealed(selection);
            */

            self.notify("playlist-selection");
        }
    }
}
