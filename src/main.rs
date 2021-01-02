extern crate glib;
extern crate gio;
extern crate gtk;
use gio::prelude::*;
use std::env::args;

mod gui;
mod web;
mod cmd;
mod config;

use crate::gui::Gui;

fn main() {
    if gtk::init().is_err() { println!("Failed to initialize GTK."); return; }
    let application = gtk::Application::new(Some("kindle-pult.zwitterio.it"), Default::default())
    .expect("Initialization failed...");

    application.connect_activate(move |app| {
        let gui = Gui::new(app);
        gui.build();
    });
    application.run(&args().collect::<Vec<_>>());
}
