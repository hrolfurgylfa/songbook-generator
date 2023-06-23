pub mod config;
pub mod gen_pdfs;
pub mod tile;

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
