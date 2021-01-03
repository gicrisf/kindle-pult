use std::process::Command;
use std::collections::HashMap;

pub struct CalibreCmd {}

impl CalibreCmd {
    pub fn convert(file: &str, to_ext: &str) -> String {
        // TODO: use format, more elegance
        let convert_arg = format!(
            r#"ebook-convert "{file}" .{ext}"#,
            file = file,
            ext = to_ext,
        );

        println!("***** conversion *****");
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").arg("/C").arg(&convert_arg).output()
            .expect("Windows failed to execute send cmd")
        } else {
            Command::new("sh").arg("-c").arg(&convert_arg).output()
            .expect("Linux failed to execute send cmd")
        };

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    pub fn send(filename: &str, cfg: HashMap<String, String>) -> String {
        println!("***** sending... *****");

        // Calibre Send smtp commands
        let smtp_arg = format!(
            r#"{cmd} {a} {s} {r} {port} {u} {p} {mail} {tomail} """#,
            cmd = "calibre-smtp",
            a = format!(r#"-a "{}.mobi""#, "filename"),  // Attachment must be mobi
            s = format!(r#"-s "{}""#, "filename"),  // Subject can be epub/azw3/other
            r = format!(r#"-r "{}""#, cfg.get("smtp").unwrap()),
            port = format!(r#"--port {}"#, cfg.get("port").unwrap()),
            u = format!(r#"-u "{}""#, cfg.get("username").unwrap()),
            p = format!(r#"-p "{}""#, cfg.get("password").unwrap()),
            mail = format!(r#""{}""#, cfg.get("from_mail").unwrap()),
            tomail = format!(r#""{}""#, cfg.get("to_mail").unwrap()),
        );

        let this_smtp_arg = smtp_arg.replace("filename", &filename);
        // println!("{}\n", this_smtp_arg);

        // Launch command
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").arg("/C").arg(&this_smtp_arg).output()
            .expect("Windows failed to execute send cmd")
        } else {
            Command::new("sh").arg("-c").arg(&this_smtp_arg).output()
            .expect("Linux failed to execute send cmd")
        };

        // Shell output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        println!("{}", stdout);
        stdout
    }
}  // CalibreCmd

pub enum ReadabiliPyParser {
    Python,
    Mozilla,
}

pub struct ReadabiliPyCmd {
    parser: ReadabiliPyParser,
}

impl ReadabiliPyCmd {
    pub fn new(parser: ReadabiliPyParser) -> Self {
        Self {
            parser,
        }
    }

    pub fn json_from_file(&self, html_fpath: String, json_fpath: String) -> String {

        let parser_arg = match self.parser {
            ReadabiliPyParser::Python => { "-p" },
            ReadabiliPyParser::Mozilla => { "" },
        };

        let arg = format!(
            r#"readabilipy {parser} -i {in} -o {out}"#,
            parser = parser_arg,
            in = html_fpath,
            out = json_fpath,
        );

        // Launch command. TODO: Add to trait for all commands!
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").arg("/C").arg(&arg).output()
            .expect("Windows failed to execute send cmd")
        } else {
            Command::new("sh").arg("-c").arg(&arg).output()
            .expect("Linux failed to execute send cmd")
        };

        // Shell output
        String::from_utf8_lossy(&output.stdout).to_string()
    }
}
