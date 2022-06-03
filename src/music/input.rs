use std::sync::Arc;

use gtk::{prelude::*, Builder, Button, Entry, Popover};

use super::{
    config::{parse_config, write_config, Bv, Config},
    data::SongCollection,
    model::PlayListModel,
};

pub fn add_bv(bv: &String) {
    let res = parse_config();
    match res {
        Ok(mut config) => {
            config.bv_list.push(Bv::new(bv));
            write_config(&config).unwrap();
        }
        _error => {
            let mut config = Config::new();
            config.bv_list.push(Bv::new(bv));
            write_config(&config).unwrap();
        }
    }
}

pub fn init_input(builder: &Builder, model: &Arc<PlayListModel>) {
    let btn: Button = builder.object("add_confirm").unwrap();
    let input: Entry = builder.object("bv_input").unwrap();
    let popover: Popover = builder.object("popover-add").unwrap();

    let model = model.clone();
    btn.connect_clicked(move |_| {
        let buffer = input.buffer();
        let bv = buffer.text();

        buffer.delete_text(0, None);
        popover.set_visible(false);

        let model_strong = model.clone();
        model.rt.spawn(async move {
            let collection = SongCollection::new(bv.as_str());
            let res = collection.get_songs(&model_strong).await;
            match res {
                Ok(()) => add_bv(&bv),
                _error => println!("wrong bv"),
            }
        });
    });
}
