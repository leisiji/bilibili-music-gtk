use glib::clone;
use gtk::{
    gio::{self, ListModel},
    glib,
    prelude::*,
    subclass::prelude::*,
};
use rand::{prelude::SliceRandom, thread_rng};

mod imp {
    use std::cell::RefCell;

    use gstreamer::glib::once_cell::sync::Lazy;
    use gtk::glib::{ParamFlags, ParamSpec, ParamSpecObject};

    use super::*;

    #[derive(Default)]
    pub struct ShuffleListModel {
        pub model: RefCell<Option<gio::ListModel>>,
        pub shuffle: RefCell<Option<Vec<u32>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShuffleListModel {
        const NAME: &'static str = "ShuffleListModel";
        type Type = super::ShuffleListModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ShuffleListModel {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "model",
                    "",
                    "",
                    gio::ListModel::static_type(),
                    ParamFlags::READWRITE | ParamFlags::EXPLICIT_NOTIFY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "model" => self.model.borrow().to_value(),
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
                "model" => obj.set_model(value.get::<gio::ListModel>().ok().as_ref()),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for ShuffleListModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            if let Some(ref model) = *self.model.borrow() {
                return model.item_type();
            }
            glib::Object::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            if let Some(ref model) = *self.model.borrow() {
                return model.n_items();
            }
            0
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            if let Some(ref model) = *self.model.borrow() {
                if let Some(ref shuffle) = *self.shuffle.borrow() {
                    if let Some(shuffed_pos) = shuffle.get(position as usize) {
                        return model.item(*shuffed_pos);
                    }
                } else {
                    return model.item(position);
                }
            }
            None
        }
    }
}

glib::wrapper! {
    pub struct ShuffleListModel(ObjectSubclass<imp::ShuffleListModel>)
        @implements gio::ListModel;
}

impl Default for ShuffleListModel {
    fn default() -> Self {
        Self::new(gio::ListModel::NONE)
    }
}

impl ShuffleListModel {
    pub fn new(model: Option<&impl IsA<ListModel>>) -> ShuffleListModel {
        glib::Object::new(&[("model", &model.map(|m| m.as_ref()))])
            .expect("Failed to create ShuffleListModel")
    }

    pub fn shuffled(&self) -> bool {
        self.imp().shuffle.borrow().is_some()
    }

    pub fn unshuffle(&self) {
        if let Some(ref model) = *self.imp().model.borrow() {
            self.imp().shuffle.replace(None);
            self.items_changed(0, model.n_items(), model.n_items());
        }
    }

    pub fn reshuffle(&self) {
        if let Some(ref model) = *self.imp().model.borrow() {
            let mut positions: Vec<u32> = (0..model.n_items()).collect();
            let mut rng = thread_rng();
            positions.shuffle(&mut rng);

            self.imp().shuffle.replace(Some(positions));
            self.items_changed(0, model.n_items(), model.n_items());
        } else {
            self.imp().shuffle.replace(None);
        }
    }

    pub fn set_model(&self, model: Option<&gio::ListModel>) {
        if let Some(model) = model {
            self.imp().model.replace(Some(model.clone()));
            /* 实现了 ListModel 必须调用该方法去通知 model items_changed */
            model.connect_items_changed(
                clone!(@strong self as this => move |_, position, removed, added| {
                    if let Some(ref shuffle) = *this.imp().shuffle.borrow() {
                        if let Some(shuffled_pos) = shuffle.get(position as usize) {
                            this.items_changed(*shuffled_pos, removed, added);
                        }
                    } else {
                        this.items_changed(position, removed, added);
                    }
                }),
            );
        } else {
            self.imp().model.replace(None);
        }
        self.notify("model");
    }
}
