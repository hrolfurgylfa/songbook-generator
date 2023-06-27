use std::fs;

use generator::config;
use pdfium_render::prelude::PdfiumError;
use serde::Deserialize;

fn true_func() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileBookConfig {
    #[serde(rename = "front")]
    pub front_pages: Vec<config::Page>,
    #[serde(rename = "back")]
    pub back_pages: Vec<config::Page>,
    pub preferred_font: String,
    #[serde(default = "true_func")]
    pub reorder_pages: bool,
}

fn parse_args() -> config::BookConfig {
    let mut args = std::env::args().skip(1);
    let mut config = None;
    let mut songs = Vec::new();

    while let Some(arg) = args.next() {
        match arg.to_lowercase().as_str() {
            "-h" | "--help" => {
                println!("Usage: skata-songbok settings.toml song1.txt song2.txt ...");
                std::process::exit(0);
            }
            "-v" | "--version" => {
                println!(
                    "skata-songbok {}",
                    std::option_env!("CARGO_PKG_VERSION").unwrap()
                );
                std::process::exit(0);
            }
            filename if filename.ends_with(".toml") => {
                let toml_str =
                    std::fs::read_to_string(&arg).expect("Failed to open .toml configuration file");
                config = Some(match toml::from_str::<FileBookConfig>(&toml_str) {
                    Ok(file_book_config) => file_book_config,
                    Err(e) => {
                        println!(
                            "Failed when reading configuration file {}:\n{}",
                            &arg,
                            e.to_string()
                        );
                        std::process::exit(10);
                    }
                });
            }
            filename if filename.ends_with(".txt") => {
                let title = std::path::Path::new(&arg)
                    .file_stem()
                    .expect(&format!("Invalid song file name: \"{}\"", &arg))
                    .to_string_lossy()
                    .to_string();
                songs.push(generator::parse_song_body(
                    title,
                    &std::fs::read_to_string(&arg)
                        .expect(&format!("Failed to open song file: \"{}\"", &arg)),
                ));
            }
            _ => panic!("Invalid argument: \"{}\"", arg),
        }
    }

    let file_book_config = config.expect("No .toml configuration file provided.");
    return config::BookConfig {
        front_pages: file_book_config.front_pages,
        back_pages: file_book_config.back_pages,
        preferred_font: file_book_config.preferred_font,
        reorder_pages: file_book_config.reorder_pages,
        songs,
    };
}

fn main() -> Result<(), PdfiumError> {
    let config = parse_args();
    let pdf = generator::generate_book_pdf(&config)?;
    fs::write("output.pdf", pdf).expect("Failed to write output.pdf");
    Ok(())
}
