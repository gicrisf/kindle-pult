use std::process::Command;
use std::collections::HashMap;

pub struct CalibreCmd {}

impl CalibreCmd {

    pub fn convert(file: &str, to_ext: &str) -> String {
        // Use format, more elegance
        let convert_arg = r#"ebook-convert "file" .ext"#;
        let this_convert_arg = convert_arg.replace("file", file);
        let this_convert_arg = this_convert_arg.replace("ext", to_ext);

        println!("***** conversion *****");
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").arg("/C").arg(&this_convert_arg).output()
            .expect("Windows failed to execute send cmd")
        } else {
            Command::new("sh").arg("-c").arg(&this_convert_arg).output()
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

        String::from_utf8_lossy(&output.stdout).to_string()
    }

}

pub struct ReadabiliPyCmd {}

impl ReadabiliPyCmd {
    pub fn simple_json_from_file(html_fpath: String, json_fpath: String) -> String {

        let arg = format!(
            r#"readabilipy -p -i {in} -o {out}"#,
            in = html_fpath,
            out = json_fpath,
        );

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").arg("/C").arg(&arg).output()
            .expect("Windows failed to execute send cmd")
        } else {
            Command::new("sh").arg("-c").arg(&arg).output()
            .expect("Linux failed to execute send cmd")
        };

        String::from_utf8_lossy(&output.stdout).to_string()
    }
}
