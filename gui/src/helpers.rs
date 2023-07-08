use std::{fs, path::Path};

use generator::config::BookConfig;

const DEFAULT_FONT: &str = "RobotoStripped";

pub fn font_exists(font: &str) -> bool {
    for style in ["Regular", "Italic", "Bold", "BoldItalic"].iter() {
        let path_str = format!("./fonts/{}/{}-{}.ttf", font, font, style);
        let path = Path::new(&path_str);
        if !path.exists() {
            println!(
                "Font \"{}\" not found; font file \"{}\" does not exist",
                font,
                path.display()
            );
            return false;
        }
    }

    return true;
}

pub fn load_book() -> BookConfig {
    // Load settings from file
    let mut book = match fs::read_to_string("settings.json") {
        Ok(c) => match serde_json::from_str::<BookConfig>(&c) {
            Ok(b) => b,
            Err(_) => {
                fs::rename("settings.json", "settings.old.json").unwrap();
                BookConfig::default()
            }
        },
        Err(_) => BookConfig::default(),
    };

    // Make sure we have a valid font
    if !font_exists(&book.preferred_font) {
        let old_font = book.preferred_font.clone();
        book.preferred_font = DEFAULT_FONT.to_owned();
        println!(
            "Font \"{}\" not found; using default font {}",
            old_font, DEFAULT_FONT
        );
    }

    // Add the song bodies, and remove any that can't be found
    book.songs = book
        .songs
        .into_iter()
        .filter_map(|s| match generator::load_song(&s.title) {
            Ok(s) => Some(s),
            Err(e) => {
                println!(
                    "Failed to find song \"{}\": {}. Removing it from the current songbook",
                    s.title, e
                );
                None
            }
        })
        .collect();

    // Save, in case we had to change the file while loading
    fs::write(
        "settings.json",
        serde_json::to_string_pretty(&book).unwrap(),
    )
    .unwrap();

    // Start the program
    book
}

pub fn generate_pdf(book: &BookConfig) {
    // Make sure we have some pages to generate
    if book.songs.is_empty() && book.front_pages.is_empty() && book.back_pages.is_empty() {
        println!("No songs or pages to generate!");
        return;
    }

    // Generate the songbook PDF
    let pdf = match generator::generate_book_pdf(&book) {
        Ok(pdf) => pdf,
        Err(e) => {
            println!("Error generating PDF: {}", e);
            return;
        }
    };

    // Write the PDF to disk
    match fs::write("book.pdf", pdf) {
        Ok(_) => {}
        Err(e) => println!("Error writing PDF: {}", e),
    };

    // Open the PDF
    match open::that("book.pdf") {
        Ok(_) => {}
        Err(e) => println!("Error opening PDF: {}", e),
    };
}
