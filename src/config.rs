use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::result::Result;

// Config file serialization
// PultConf is for sending and converting
#[derive(Serialize, Deserialize, Debug)]
pub struct PultConf {
    pub del_sent: String,
    pub to_ext: String,
    pub smtp: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub from_mail: String,
    pub to_mail: String,
}

/// `PultConf` implements `Default`
impl Default for PultConf {
    fn default() -> Self {
        Self {
            del_sent: "false".into(),
            to_ext: "mobi".into(),
            smtp: "smtp.gmail.com".into(),
            port: "587".into(),
            username: "user.name".into(),
            password: "your-password".into(),
            from_mail: "user.name@gmail.com".into(),
            to_mail: "ebook-mail@kindle.com".into(),
        }
    }
}

impl PultConf {
    fn dump_to_hashmap(&self) -> HashMap<String, String> {
        let mut values = HashMap::new();
        values.insert(String::from("del_sent"), String::from(&self.del_sent));
        values.insert(String::from("to_ext"), String::from(&self.to_ext));
        values.insert(String::from("smtp"), String::from(&self.smtp));
        values.insert(String::from("port"), String::from(&self.port));
        values.insert(String::from("username"), String::from(&self.username));
        values.insert(String::from("password"), String::from(&self.password));
        values.insert(String::from("from_mail"), String::from(&self.from_mail));
        values.insert(String::from("to_mail"), String::from(&self.to_mail));

        values
    }

    pub fn reload() -> HashMap<String, String> {
        // Load config file info
        let confy_loaded: Result<PultConf, confy::ConfyError> = confy::load("kindle-pult");

        // Reset to default if some error occurs
        let conf_hashmap: HashMap<String, String> = match confy_loaded {
            Ok(c) => {
                c.dump_to_hashmap()
            },
            Err(e) => {
                println!("{:?}", e);
                println!("Replacing with default config values and dumping.");
                let _ = confy::store("kindle-pult", PultConf::default());
                let default_c: PultConf = confy::load("kindle-pult").unwrap();
                default_c.dump_to_hashmap()
            },
        };

        conf_hashmap
    }
}
