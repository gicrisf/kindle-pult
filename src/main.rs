extern crate glib;
extern crate gio;
extern crate gtk;
use glib::clone;
use gtk::prelude::*;
use gio::prelude::*;

use std::sync::{Arc, Mutex};
use std::env;
use std::env::args;
use std::fs;

mod cmd;
mod config;

use cmd::CalibreCmd;
use config::KindlePultConf;

struct GuiField {
    label: gtk::Label,
    buffer: gtk::EntryBuffer,
    entry: gtk::Entry,
}

impl GuiField {
    fn new(lbl_string: &str, bfr_string: &str) -> Self {
        let label = gtk::Label::new(Some(lbl_string));
        let buffer = gtk::EntryBuffer::new(Some(bfr_string));
        let entry = gtk::Entry::with_buffer(&buffer);

        entry.set_size_request(220, 20);
        label.set_hexpand(true);

        Self {
            label,
            entry,
            buffer,
        }
    }
}

struct Gui {
    win: gtk::ApplicationWindow,
}

impl Gui {
    fn new(application: &gtk::Application) -> Self {
        let win = gtk::ApplicationWindow::new(application);  // some?

        Self {
            win,
        }
    }

    fn build_file_chooser_btn(&self, sender: glib::Sender<Vec<std::path::PathBuf>>) -> gtk::Button {
        let button = gtk::Button::with_label("Select");

        // Dialog for getting source files
        let win = &self.win;
        button.connect_clicked(clone!(@weak win => move |_| {
            let dialog = gtk::FileChooserDialog::new(
                Some("Choose a file"),
                Some(&win),
                gtk::FileChooserAction::Open
            );

            dialog.add_buttons(&[
                ("Open", gtk::ResponseType::Ok),
                ("Cancel", gtk::ResponseType::Cancel)
            ]);

            dialog.set_select_multiple(true);
            let sender_clone = sender.clone();

            dialog.connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Ok {
                    let files = dialog.get_filenames();
                    sender_clone.send(files).unwrap();
                }
                dialog.close();
            });
            dialog.show_all();
        }));

        button
    }  // build_file_chooser_btn

    fn build(&self) {
        let cfg = KindlePultConf::reload();

        // Select files; TODO: use RefCell
        let source_files = Arc::new(Mutex::new(Vec::new())); // Store String paths

        let (open_sender, open_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let select_files_btn = self.build_file_chooser_btn(open_sender);

        // Receiver from dialog sender
        let source_files_clone = Arc::clone(&source_files);
        open_receiver.attach(None, move |files: Vec<std::path::PathBuf>| {
            for file in files {
                // Push to source_files vector
                let mut m = source_files_clone.lock().unwrap();
                m.push(file);
            }
            glib::Continue(true)
        });

        // Vertical box
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10 as i32);
        vbox.set_margin_top(10 as i32);
        vbox.set_margin_start(10 as i32);
        vbox.set_margin_end(10 as i32);
        vbox.set_margin_bottom(10 as i32);

        // Header bar
        let headerbar = gtk::HeaderBar::new();
        headerbar.add(&select_files_btn);  // Add select button to headerbar
        headerbar.set_show_close_button(true);  // Show close/extend/minimize in headerbar
        self.win.set_titlebar(Some(&headerbar));  // Set this headerbar as title bar (the top one)

        // Grid
        let grid = gtk::Grid::new();

        grid.set_row_homogeneous(true);
        // grid.set_column_homogeneous(true);

        grid.set_row_spacing(10);
        grid.set_column_spacing(10);

        let from_mail = GuiField::new("From:", cfg.get("from_mail").unwrap());
        grid.attach(&from_mail.label, 0, 0, 1, 1);  // column 0, row 0
        grid.attach(&from_mail.entry, 1, 0, 1, 1);  // column 1, row 0
        let to_mail = GuiField::new("To:", cfg.get("to_mail").unwrap());
        grid.attach(&to_mail.label, 2, 0, 1, 1);
        grid.attach(&to_mail.entry, 3, 0, 1, 1);

        let smtp = GuiField::new("Protocol:", cfg.get("smtp").unwrap());
        grid.attach(&smtp.label, 0, 1, 1, 1);
        grid.attach(&smtp.entry, 1, 1, 1, 1);
        let port = GuiField::new("Port:", cfg.get("port").unwrap());
        grid.attach(&port.label, 2, 1, 1, 1);
        grid.attach(&port.entry, 3, 1, 1, 1);

        let user = GuiField::new("User:", cfg.get("username").unwrap());
        grid.attach(&user.label, 0, 2, 1, 1);
        grid.attach(&user.entry, 1, 2, 1, 1);
        let password = GuiField::new("Password:", cfg.get("password").unwrap());
        &password.entry.set_visibility(false);
        grid.attach(&password.label, 2, 2, 1, 1);
        grid.attach(&password.entry, 3, 2, 1, 1);

        let to_ext = GuiField::new("Extension:", cfg.get("to_ext").unwrap());
        grid.attach(&to_ext.label, 0, 3, 1, 1);  // column 0, row 0
        grid.attach(&to_ext.entry, 1, 3, 1, 1);  // column 1, row 0

        let del_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let del_sent = gtk::Switch::new();
        let del_sent_lbl = gtk::Label::new(Some("Delete sent files?"));
        del_sent.set_active(cfg.get("del_sent").unwrap().parse().unwrap());
        del_box.add(&del_sent_lbl);
        del_box.add(&del_sent);
        // grid.attach(&del_sent_lbl, 2, 3, 1, 1);
        grid.attach(&del_box, 3, 3, 1, 1);

        vbox.add(&grid);

        let btn_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        btn_box.set_margin_top(10 as i32);
        btn_box.set_layout(gtk::ButtonBoxStyle::Spread);

        // Save button
        let save_button = gtk::Button::with_label("Save settings");
        save_button.set_property_expand(false);

        save_button.connect_clicked(move |_| {
            let new_conf = KindlePultConf {
                del_sent: {if del_sent.get_state() { "true".into() } else { "false".into() }},
                to_ext: to_ext.buffer.get_text(),
                smtp: smtp.buffer.get_text(),
                port: port.buffer.get_text(),
                username: user.buffer.get_text(),
                password: password.buffer.get_text(),
                from_mail: from_mail.buffer.get_text(),
                to_mail: to_mail.buffer.get_text(),
            };

            let _ = confy::store("kindle-pult", new_conf);
            let _ = KindlePultConf::reload();
        });  // Connect clicked button

        btn_box.add(&save_button);

        // Send button
        let send_button = gtk::Button::with_label("Send");
        send_button.set_property_expand(false);
        send_button.set_widget_name("suggested-action");  // Mark as primary

        // On clicked button
        let source_files_clone = Arc::clone(&source_files);
        send_button.connect_clicked(move |_| {
            let files = source_files_clone.lock().unwrap();

            for file in &*files {
                // Check file and its extension
                if file.exists() {
                    // CD in file directory
                    let file_dir = file.parent().unwrap();
                    let _cd_success = env::set_current_dir(&file_dir);
                    let from_ext = file.extension().unwrap().to_str().unwrap();
                    let to_ext = cfg.get("to_ext").unwrap();

                    if from_ext == to_ext {
                        println!("Conversion unnecessary");
                    } else {
                        let _conv_output = CalibreCmd::convert(file.to_str().unwrap(), &to_ext);
                    }
                } else {
                    println!("File not found.");
                    return
                }

                let filename = file.file_stem().unwrap();
                let _send_output = CalibreCmd::send(filename.to_str().unwrap(), cfg.clone());

                // Delete sent file?
                if cfg.get("del_sent").unwrap().parse().unwrap() {
                    let _del_result = fs::remove_file(file);
                }
            }
        });

        btn_box.add(&send_button);
        vbox.add(&btn_box);

        // Window final settings
        select_files_btn.grab_focus();
        self.win.add(&vbox);
        self.win.set_title("Kindle-pult");
        self.win.set_position(gtk::WindowPosition::Center);
        self.win.show_all();
    }
}

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
