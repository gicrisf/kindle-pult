extern crate glib;
extern crate gio;
extern crate gtk;
use glib::clone;
use gtk::prelude::*;

use std::sync::{Arc, Mutex};
use std::env;
use std::fs;
use std::collections::HashMap;

use crate::cmd::CalibreCmd;
use crate::web::download_as_epub;
use crate::config::PultConf;

struct CfgField {
    label: gtk::Label,
    buffer: gtk::EntryBuffer,
    entry: gtk::Entry,
}

impl CfgField {
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

struct CfgFields {
    from_mail: CfgField,
    to_mail: CfgField,
    smtp: CfgField,
    port: CfgField,
    user: CfgField,
    password: CfgField,
    to_ext: CfgField,
}

pub struct Gui {
    win: gtk::ApplicationWindow,
    vbox: gtk::Box,
    file_img: gtk::Image,
    source_files: Arc<Mutex<Vec<std::path::PathBuf>>>,  // TODO: use RefCell
    open_sender: glib::Sender<Vec<std::path::PathBuf>>,
    cfg: HashMap<String, String>,
}

impl Gui {
    pub fn new(application: &gtk::Application) -> Self {
        // Main window
        let win = gtk::ApplicationWindow::new(application);

        // Build vbox
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10 as i32);
        vbox.set_margin_top(10 as i32);
        vbox.set_margin_start(10 as i32);
        vbox.set_margin_end(10 as i32);
        vbox.set_margin_bottom(10 as i32);
        win.add(&vbox);

        // Images
        let file_img = gtk::Image::from_icon_name(Some("document-open"), gtk::IconSize::Button);

        // Shared Ref
        let (open_sender, open_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let source_files = Arc::new(Mutex::new(Vec::new())); // Store String paths

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

        // Reload Conf
        let cfg = PultConf::reload();

        Self {
            win,
            vbox,
            file_img,
            source_files,
            open_sender,
            cfg,
        }
    }

    fn build_headerbar(&self) -> gtk::HeaderBar {
        let headerbar = gtk::HeaderBar::new();

        let select_files_btn = gtk::Button::new();
        select_files_btn.add(&self.file_img);

        // Dialog for getting source files
        let win = &self.win;
        let sender_clone = self.open_sender.clone();
        select_files_btn.connect_clicked(clone!(@weak win => move |_| {
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
            let sender_clone = sender_clone.clone();

            dialog.connect_response(move |dialog, response| {
                if response == gtk::ResponseType::Ok {
                    let files = dialog.get_filenames();
                    sender_clone.send(files).unwrap();
                }
                dialog.close();
            });
            dialog.show_all();
        }));

        select_files_btn.grab_focus();

        headerbar.add(&select_files_btn);  // Add select button to headerbar
        headerbar.set_show_close_button(true);  // Show close/extend/minimize in headerbar
        self.win.set_titlebar(Some(&headerbar));  // Set this headerbar as title bar (the top one)

        headerbar
    }

    fn build_url_box(&self) {
        let url_box = gtk::Box::new(gtk::Orientation::Horizontal, 10 as i32);

        let url_field = CfgField::new("URL:", "");
        url_field.entry.set_size_request(420, 20);
        url_field.entry.set_hexpand(true);

        let download_btn = gtk::Button::with_label("Download");
        download_btn.set_property_expand(false);

        let url_buffer_clone = url_field.buffer.clone();
        download_btn.connect_clicked(move |_| {
            let _no = download_as_epub(url_buffer_clone.get_text());
            // println!("{:?}", no.unwrap());
        });  // Connect clicked button

        url_box.add(&url_field.label);
        url_box.add(&url_field.entry);
        url_box.add(&download_btn);
        url_box.set_margin_bottom(20);

        self.vbox.add(&url_box);
    }

    fn make_cfg_fields(&self) -> CfgFields {
        CfgFields {
            from_mail: CfgField::new("From:", self.cfg.get("from_mail").unwrap()),
            to_mail: CfgField::new("To:", self.cfg.get("to_mail").unwrap()),
            smtp: CfgField::new("Protocol:", self.cfg.get("smtp").unwrap()),
            port: CfgField::new("Port:", self.cfg.get("port").unwrap()),
            user: CfgField::new("User:", self.cfg.get("username").unwrap()),
            password: CfgField::new("Password:", self.cfg.get("password").unwrap()),
            to_ext: CfgField::new("Extension:", self.cfg.get("to_ext").unwrap()),
        }
    }

    fn build_cfg_ui(&self, flds: CfgFields) {
        let grid = gtk::Grid::new();

        // Grid spacing
        grid.set_row_homogeneous(true);
        // grid.set_column_homogeneous(true);
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);
        grid.set_margin_bottom(20);

        // Row 0
        grid.attach(&flds.from_mail.label, 0, 0, 1, 1);  // Col 0
        grid.attach(&flds.from_mail.entry, 1, 0, 1, 1);  // Col 1
        grid.attach(&flds.to_mail.label, 2, 0, 1, 1);  // Col 2
        grid.attach(&flds.to_mail.entry, 3, 0, 1, 1);  // Col 3

        // Row 1
        grid.attach(&flds.smtp.label, 0, 1, 1, 1);
        grid.attach(&flds.smtp.entry, 1, 1, 1, 1);
        grid.attach(&flds.port.label, 2, 1, 1, 1);
        grid.attach(&flds.port.entry, 3, 1, 1, 1);

        // Row 2
        grid.attach(&flds.user.label, 0, 2, 1, 1);
        grid.attach(&flds.user.entry, 1, 2, 1, 1);
        grid.attach(&flds.password.label, 2, 2, 1, 1);
        grid.attach(&flds.password.entry, 3, 2, 1, 1);

        &flds.password.entry.set_visibility(false);

        // Row 3
        grid.attach(&flds.to_ext.label, 0, 3, 1, 1);
        grid.attach(&flds.to_ext.entry, 1, 3, 1, 1);

        let del_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let del_sent = gtk::Switch::new();
        let del_sent_lbl = gtk::Label::new(Some("Delete sents"));
        del_sent.set_active(self.cfg.get("del_sent").unwrap().parse().unwrap());
        del_box.add(&del_sent_lbl);
        del_box.add(&del_sent);
        // grid.attach(&del_sent_lbl, 2, 3, 1, 1);
        grid.attach(&del_box, 2, 3, 1, 1);

        self.vbox.add(&grid);

        // Cfg Button Box
        // let btn_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        // btn_box.set_margin_top(10 as i32);
        // btn_box.set_layout(gtk::ButtonBoxStyle::Spread);

        // Save button
        let save_button = gtk::Button::with_label("Save settings");
        save_button.set_property_expand(false);

        save_button.connect_clicked(move |_| {
            let new_conf = PultConf {
                del_sent: {if del_sent.get_state() { "true".into() } else { "false".into() }},
                to_ext: flds.to_ext.buffer.get_text(),
                smtp: flds.smtp.buffer.get_text(),
                port: flds.port.buffer.get_text(),
                username: flds.user.buffer.get_text(),
                password: flds.password.buffer.get_text(),
                from_mail: flds.from_mail.buffer.get_text(),
                to_mail: flds.to_mail.buffer.get_text(),
            };

            let _ = confy::store("kindle-pult", new_conf);
            let _ = PultConf::reload();
        });  // Connect clicked button

        // btn_box.add(&save_button);
        grid.attach(&save_button, 3, 3, 1, 1);
        // self.vbox.add(&btn_box);
    }  // build_cfg_ui

    pub fn build(&self) {
        // HeaderBar
        self.build_headerbar();

        // URL Area
        self.build_url_box();

        // Cfg Area
        let cfg_fields = self.make_cfg_fields();
        self.build_cfg_ui(cfg_fields);

        // Send button
        let send_button = gtk::Button::with_label("Send");

        let source_files_clone = Arc::clone(&self.source_files);
        let cfg_clone = self.cfg.clone();
        send_button.connect_clicked(move |_| {  // On clicked send button...
            let files = source_files_clone.lock().unwrap();

            for file in &*files {
                // Check file and its extension
                if file.exists() {
                    // CD in file directory
                    let file_dir = file.parent().unwrap();
                    let _cd_success = env::set_current_dir(&file_dir);
                    let from_ext = file.extension().unwrap().to_str().unwrap();
                    let to_ext = cfg_clone.get("to_ext").unwrap();

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
                let _send_output = CalibreCmd::send(filename.to_str().unwrap(), cfg_clone.clone());

                // Delete sent file?
                if cfg_clone.get("del_sent").unwrap().parse().unwrap() {
                    let _del_result = fs::remove_file(file);
                }
            }
        });

        send_button.set_property_expand(false);
        send_button.set_widget_name("suggested-action");  // Mark as primary
        self.vbox.add(&send_button);

        // Win final settings
        self.win.set_title("Kindle-pult");
        self.win.set_position(gtk::WindowPosition::Center);
        self.win.show_all();
    }
}
