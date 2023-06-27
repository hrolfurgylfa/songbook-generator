pub mod config;
pub mod gen_pdfs;
pub mod tile;

use std::fs;

use pdfium_render::prelude::{Pdfium, PdfiumError};
use wasm_bindgen::prelude::*;

pub fn generate_book_pdfs(config: &config::BookConfig) -> Vec<Vec<u8>> {
    let font = gen_pdfs::load_font(&config.preferred_font);
    let mut pdfs = Vec::with_capacity(
        config.front_pages.len()
            + config.back_pages.len()
            + (if config.songs.is_empty() { 0 } else { 1 }),
    );

    for page in &config.front_pages {
        pdfs.push(gen_pdfs::generate_page(&font, &config.songs, page));
    }

    if !config.songs.is_empty() {
        let songs = gen_pdfs::generate_songs(&font, &config.songs);
        pdfs.push(songs);
    }

    for page in config.back_pages.iter() {
        pdfs.push(gen_pdfs::generate_page(&font, &config.songs, page));
    }

    return pdfs;
}

pub fn generate_book_pdf(config: &config::BookConfig) -> Result<Vec<u8>, PdfiumError> {
    let pdfs = generate_book_pdfs(&config);

    #[cfg(target_family = "wasm")]
    let pdfium = Pdfium::new(Pdfium::bind_to_system_library()?);
    #[cfg(not(target_family = "wasm"))]
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );

    let mut pdfium_doc = tile::merge_pdfs(&pdfium, pdfs)?;
    if config.reorder_pages {
        pdfium_doc = tile::mix_first_and_last(&pdfium, &pdfium_doc)?;
    }
    let tiled_doc = tile::tile_pages(pdfium_doc.pages(), 0.9)?;
    return Ok(tiled_doc.save_to_bytes()?);
}

#[wasm_bindgen]
pub fn generate_book_pdf_wasm(config_json: String) -> Vec<u8> {
    let config: config::BookConfig = serde_json::from_str(&config_json).unwrap();
    return generate_book_pdf(&config).unwrap();
}

pub fn load_song(title: &str) -> Result<config::Song, String> {
    let songs =
        fs::read_dir("./songs/").map_err(|e| format!("Failed to read songs directory: {}", e))?;
    for res in songs {
        let path = match res {
            Ok(song) => song.path(),
            Err(e) => {
                println!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let name = path
            .file_stem()
            .map(|f| f.to_str().unwrap_or(""))
            .unwrap_or("");

        if name == title {
            let body = fs::read_to_string(path.clone()).unwrap();
            return Ok(parse_song_body(name.to_owned(), &body));
        }
    }

    Err("Song not found".to_owned())
}

pub fn parse_song_body(title: impl ToString, body: &str) -> config::Song {
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

    config::Song {
        title: title.to_string(),
        body: verses,
    }
}
