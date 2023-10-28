pub mod config;
pub mod fonts;
pub mod gen_pdfs;
pub mod tile;

use std::{collections::HashMap, fmt::Display, fs};

use fonts::FontError;
use pdfium_render::prelude::{Pdfium, PdfiumError};
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub enum GenerationError {
    PdfiumError(PdfiumError),
    FontError(FontError),
}

impl From<PdfiumError> for GenerationError {
    fn from(value: PdfiumError) -> Self {
        Self::PdfiumError(value)
    }
}

impl From<FontError> for GenerationError {
    fn from(value: FontError) -> Self {
        Self::FontError(value)
    }
}

impl Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenerationError::")?;
        match self {
            Self::PdfiumError(e) => write!(f, "PdfiumError({})", e),
            Self::FontError(e) => write!(f, "FontError({})", e),
        }
    }
}

pub fn generate_book_pdfs(config: &config::BookConfig) -> Result<Vec<Vec<u8>>, FontError> {
    let font = gen_pdfs::load_font(&config.preferred_font)?;
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

    return Ok(pdfs);
}

pub fn generate_book_pdf(config: &config::BookConfig) -> Result<Vec<u8>, GenerationError> {
    let pdfs = generate_book_pdfs(&config)?;

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
    let tiled_doc = tile::tile_pages(
        pdfium_doc.pages(),
        1.0 - ((config.padding as f32) / 100.0),
        config.add_separator,
        config.tiled_page_size,
    )?;
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
            return parse_song_body(name.to_owned(), &body);
        }
    }

    Err("Song not found".to_owned())
}

pub fn parse_song_body(title: impl ToString, body: &str) -> Result<config::Song, String> {
    let title = title.to_string();
    let tag_err =
        |i: usize, msg: &str| Err(format!("{} in line {} of song {}", msg, i + 1, &title));
    let mut verses = Vec::new();
    let mut current_verse = String::new();
    let mut tags = HashMap::new();
    let mut tag_parsing_mode = false;

    for (i, line) in body.lines().map(|l| l.trim()).enumerate() {
        // We don't want any verse logic in tag parsing mode
        if tag_parsing_mode {
            if let Some(splitter_loc) = line.find(':') {
                if line.len() <= splitter_loc + 1 {
                    return tag_err(i, "Expected some string after : separator");
                }
                let key = line[..splitter_loc].trim().to_lowercase();
                let value = line[splitter_loc + 1..]
                    .split(";")
                    .map(|s| s.trim().to_owned())
                    .collect();
                tags.insert(key, value);
            } else {
                return tag_err(i, "Expected : to separate key from value");
            }
            continue;
        }

        // If we have double new line, a new verse has started. Verses can't be empty though, so
        // only push the current verse if it's not empty.
        if line.is_empty() && !current_verse.is_empty() {
            verses.push(current_verse);
            current_verse = String::new();
            continue;
        }

        // Check if we should switch to tag parsing
        if line.chars().all(|c| c == '-') && line.len() >= 4 {
            tag_parsing_mode = true;
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

    Ok(config::Song {
        title,
        body: verses,
        tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SONG_BODY: &str = r"
        Hann ljótur er á litinn 
        og líka er striginn slitinn, 
        þó bragðast vel hver bitinn 
        úr bakpokanum enn. 

        Á mörgum fjallatindi 
        í miklu frosti og vindi 
        hann var það augnayndi, 
        sem elska svangir menn. 
    ";

    const PARSED_SONG_BODY: &[&str] = &[
        r"Hann ljótur er á litinn
og líka er striginn slitinn,
þó bragðast vel hver bitinn
úr bakpokanum enn.",
        r"Á mörgum fjallatindi
í miklu frosti og vindi
hann var það augnayndi,
sem elska svangir menn.",
    ];

    fn default_parse_song(extra_body: &str) -> (Result<config::Song, String>, config::Song) {
        let parsed_song = parse_song_body("Aa", &(SONG_BODY.to_owned() + extra_body));
        let expected_song = config::Song {
            title: "Aa".to_owned(),
            body: PARSED_SONG_BODY.iter().map(|v| (*v).to_owned()).collect(),
            tags: HashMap::new(),
        };
        (parsed_song, expected_song)
    }

    #[test]
    fn test_parse_song_body_no_tags() {
        let (parsed, expected) = default_parse_song("");
        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn test_parse_song_body_simple_tags() {
        let tags = r"----
        hÖfUnDur: Jón Jónsson
        árTal  : 1976";
        let (parsed, expected) = default_parse_song(tags);
        let parsed = parsed.unwrap();
        assert_eq!(
            parsed.tags,
            [("höfundur", ["Jón Jónsson"]), ("ártal", ["1976"])]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.map(|s| s.to_owned()).to_vec()))
                .collect()
        );
    }
}
