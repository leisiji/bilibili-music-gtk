use gtk::{prelude::*, Button, Popover};

pub fn init_input(builder: &gtk::Builder) {
    let button: Button = builder.object("add-button").unwrap();
    let bv_input: Popover = builder.object("bv-input").unwrap();
    button.connect_clicked(move |_| {
        bv_input.set_visible(true);
    });
}
