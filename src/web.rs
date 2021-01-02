use error_chain::error_chain;
use std::io;
use std::fs;
use tempfile::Builder;

extern crate serde;
extern crate serde_json;
use serde::{Deserialize};

use crate::cmd::{ReadabiliPyCmd, ReadabiliPyParser};

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
     }
}

#[derive(Deserialize, Debug)]
pub struct Article {
    title: Option<String>,  // The article title
    byline: Option<String>,  // Author information
    date: Option<String>,
    content: Option<String>,
    plain_content: Option<String>,  // plain text content of the article, preserving the HTML structure
}

pub fn download_as_epub(target: String) -> Result<Article> {
    // HTTP request
    let tmp_dir = Builder::new().prefix("kindle-pult_").tempdir()?;
    let response = reqwest::blocking::get(&target)?;

    let fpath_string: String;

    // Set up temp
    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{:?}'", fname);
        let fname = tmp_dir.path().join(fname);
        println!("will be located under: '{:?}'", fname);
        fpath_string = fname.clone().into_os_string().into_string().unwrap();
        fs::File::create(fname)?
    };

    // Get HTML string as file and copy to tempdir
    let html_string = response.text()?;
    io::copy(&mut html_string.as_bytes(), &mut dest)
        .expect("Failed to copy HTML file to file");

    // Purify HTML
    let purifier = ReadabiliPyCmd::new(ReadabiliPyParser::Mozilla);  // Select parser

    let outfpath = tmp_dir.path().join("article.json");  // TODO: use fname
    let outfpath_string = outfpath.clone().into_os_string().into_string().unwrap();
    let output = purifier.json_from_file(fpath_string, outfpath_string);
    println!("{}", output);  // Print to GUI some feedback

    // Read Json, deserialize and print Rust data structure.
    let json_file = fs::File::open(outfpath).expect("file not found");
    let article: Article = serde_json::from_reader(json_file).expect("error while reading json");
    println!("{:#?}", article);  // TODO: print to GUI

    Ok(article)
}
