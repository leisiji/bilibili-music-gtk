use gtk::{gio, glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    use gtk::{CompositeTemplate, TemplateChild};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/bilibili/music/bvid_input_view.ui")]
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
