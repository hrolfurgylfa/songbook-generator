use std::{fs, io};

use eframe::egui;
use generator::config::Song;

fn get_available_song(res: io::Result<fs::DirEntry>) -> Result<Song, String> {
    let path = res
        .map_err(|e| format!("Failed to load song from the songs folder: {}", e))?
        .path();
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Failed to get song file stem"))?;
    return generator::load_song(title)
        .map_err(|e| format!("Failed to load song from the songs folder: {}", e));
}

fn get_available_songs() -> Vec<Song> {
    let mut songs = Vec::new();
    for res in fs::read_dir("./songs").unwrap() {
        match get_available_song(res) {
            Ok(song) => songs.push(song),
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
    }
    songs
}

#[derive(Debug, Default)]
pub struct AddSong {
    available_songs: Vec<Song>,
    open: bool,
}

impl AddSong {
    pub fn open(&mut self) {
        self.open = true;
        self.available_songs = get_available_songs();
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<Song> {
        let a = egui::Window::new("Bæta við lagi")
            .vscroll(true)
            .collapsible(false)
            .open(&mut self.open)
            .show(ui.ctx(), |ui| {
                let mut selected = None;
                for song in &self.available_songs {
                    if ui.button(&song.title).clicked() {
                        selected = Some(song.clone());
                    }
                }
                selected
            });
        let new_song = a.and_then(|ir| ir.inner.and_then(|f| f));
        if new_song.is_some() {
            self.open = false;
        }
        new_song
    }
}
