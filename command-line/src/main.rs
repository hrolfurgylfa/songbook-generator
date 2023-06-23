use generator::{config, tile};
use pdfium_render::prelude::{Pdfium, PdfiumError, PdfDocument, PdfPoints};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FileBookConfig {
    #[serde(rename = "front")]
    pub front_pages: Vec<config::Page>,
    #[serde(rename = "back")]
    pub back_pages: Vec<config::Page>,
    pub preferred_font: String,
}

fn parse_song_body(body: &str) -> Vec<String> {
    let mut verses = Vec::new();
    let mut current_verse = String::new();

    for line in body.lines().map(|l| l.trim()) {
        // If we have double new line, a new verse has started. Verses can't be empty though, so
        // only push the current verse if it's not empty.
        if line.is_empty() && !current_verse.is_empty() {
            verses.push(current_verse);
            current_verse = String::new();
            continue;
        }

        // Add the new line character  if this isn't the first line.
        if !current_verse.is_empty() {
            current_verse.push('\n');
        }
        // Add the line to the current verse.
        current_verse.push_str(line);
    }

    // Push the last verse if it's not empty.
    if !current_verse.is_empty() {
        verses.push(current_verse);
    }

    verses
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
                songs.push(config::Song {
                    title: std::path::Path::new(&arg)
                        .file_stem()
                        .expect(&format!("Invalid song file name: \"{}\"", &arg))
                        .to_string_lossy()
                        .to_string(),
                    body: parse_song_body(
                        &std::fs::read_to_string(&arg)
                            .expect(&format!("Failed to open song file: \"{}\"", &arg)),
                    ),
                });
            }
            _ => panic!("Invalid argument: \"{}\"", arg),
        }
    }

    let file_book_config = config.expect("No .toml configuration file provided.");
    return config::BookConfig {
        front_pages: file_book_config.front_pages,
        back_pages: file_book_config.back_pages,
        preferred_font: file_book_config.preferred_font,
        songs,
    };
}

fn _print_pdf_info(pdf: &PdfDocument) {
    println!(
        "PDF length: {}kb",
        pdf.save_to_bytes().unwrap().len() as f32 / 1000.0
    );

    let font_size = PdfPoints::new(12.0);
    if pdf.pages().is_empty() {
        println!("There are no pages in this pdf.");
    }
    for (page_index, page) in pdf.pages().iter().enumerate() {
        if page.fonts().is_empty() {
            println!("There are no fonts on page {}", page_index);
        }
        for (font_index, font) in page.fonts().iter().enumerate() {
            println!(
                    "Font {} on page {} is embedded: name = {}, is symbolic? {}, is non-symbolic? {}, ascent {:?}, descent {:?}, number of glyphs: {}",
                    font_index,
                    page_index,
                    font.name(),
                    font.is_symbolic(),
                    font.is_non_symbolic(),
                    font.ascent(font_size),
                    font.descent(font_size),
                    font.glyphs().len()
                );
        }
    }
}

fn main() -> Result<(), PdfiumError> {
    let pdfium = Pdfium::new(Pdfium::bind_to_library(
        Pdfium::pdfium_platform_library_name_at_path("./"),
    )?);
    let config = parse_args();
    let pdfs = generator::generate_book_pdfs(&config);

    let linear_doc = tile::merge_pdfs(&pdfium, pdfs)?;
    // let merged_doc = tile::mix_first_and_last(&pdfium, linear_doc)?;
    let tiled_doc = tile::tile_pages(linear_doc.pages(), 0.9)?;
    tiled_doc.save_to_file("output.pdf")?;
    // print_pdf_info(&tiled_doc);

    Ok(())
}
