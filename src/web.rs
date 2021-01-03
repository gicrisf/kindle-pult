use std::io;
use std::fs;
use tempfile::Builder;

extern crate soup;
use soup::prelude::*;

extern crate url;
use url::{Url, ParseError};

extern crate serde;
extern crate serde_json;
use serde::{Deserialize};

use crate::cmd::{ReadabiliPyCmd, ReadabiliPyParser};

mod errors {
    error_chain! {
         foreign_links {
             Io(std::io::Error);
             HttpRequest(reqwest::Error);
         }
    }
}

use errors::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Article {
    title: Option<String>,  // The article title
    byline: Option<String>,  // Author information
    date: Option<String>,
    content: Option<String>,
    plain_content: Option<String>,  // plain text content of the article, preserving the HTML structure
}

impl Article {
    pub fn get_from_url(target: String) -> Result<()> {
        // Sanitize target url
        let target_url = Url::parse(&target);
        match target_url {
            Ok(url) => { println!("{}", url) },
            Err(e) => {
                println!("Error {}, return.", e);
                return Ok(())  // Implement Error InvalidURL
            }
        };

        println!("Valid URL, going on.");

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
        println!("ReadabiliPy output: {}", output);  // Print feedback to GUI

        // Read Json, deserialize and print Rust data structure.
        let json_file = fs::File::open(outfpath).expect("file not found");
        let article: Article = serde_json::from_reader(json_file).expect("error reading json");
        // println!("{:#?}", article);  // TODO: print to GUI

        // Get absolute image urls
        let image_urls = match article.clone().content {
            Some(content) => {
                let mut urls = Vec::new();
                let soup = Soup::new(&content);

                for img in soup.tag("img").find_all() {
                    // Can be relative url;
                    let image_url = img.get("src").expect("Couldn't find link with 'src' attribute");
                    // println!("{}", image_url);
                    // Make sure url is absolute and add it to urls vector;
                    match Url::parse(&image_url) {
                        Ok(url) => {
                            urls.push(url);
                        },
                        Err(e) => {
                            match e {
                                ParseError::RelativeUrlWithoutBase => {
                                    println!("Relative URL: {}", &image_url);
                                    let target_url = Url::parse(&target);  // Second parsing
                                    let absolute_url = target_url.unwrap().join(&image_url)
                                        .expect("Can't make absolute URL of image");

                                    println!("absolute URL: {}", &absolute_url);
                                    urls.push(absolute_url);
                                },
                                _ => {
                                    println!("errore: {}", e);
                                    return Ok(())
                                }
                            };

                        }
                    }
                };

                println!("Image URLS: {:?}", urls);
                urls
            },
            None => {
                Vec::new()
            }
        };

        // Download images
        for url in image_urls {
            let mut response = reqwest::blocking::get(url.as_str()).expect("request failed");

            // Give images proper name, then put them in temp dir with the article.
            let mut dest = {
                let img_fname = response
                    .url()
                    .path_segments()
                    .and_then(|segments| segments.last())
                    .and_then(|name| if name.is_empty() { None } else { Some(name) })
                    .unwrap_or("tmp.jpg");  // Need better management

                println!("file to download: '{:?}'", fname);
                let img_fname = tmp_dir.path().join(fname);
                println!("will be located under: '{:?}'", fname);
                fs::File::create(fname)?
            };

            io::copy(&mut response, &mut dest).expect("failed to copy content");

            // Image optimization
        }

        Ok(())
    }
}
