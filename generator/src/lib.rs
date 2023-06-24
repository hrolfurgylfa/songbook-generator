pub mod config;
pub mod gen_pdfs;
pub mod tile;

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
