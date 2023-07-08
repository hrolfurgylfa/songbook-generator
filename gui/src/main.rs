use add_song::AddSong;
use config::ItemListConfig;
use serde_json;
use std::fs;
use std::path::Path;

use generator::config::{
    BookConfig, FrontPage, Page, Preface, TableOfContents, TableOfContentsSortOrder,
};

use eframe::egui;

mod add_song;
mod config;

const AVAILABLE_FONTS: [&str; 2] = ["Roboto", "RobotoStripped"];
const DEFAULT_FONT: &str = "RobotoStripped";

fn font_exists(font: &str) -> bool {
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

fn update_move_list<T: Clone, A: FnOnce(), B>(
    ui: &mut egui::Ui,
    config: ItemListConfig<T, A, B>,
) -> egui::Response
where
    B: Fn(&mut egui::Ui, usize, &mut T) -> egui::Response,
{
    let mut written = false;
    let mut response = ui
        .vertical(|ui| {
            let ItemListConfig {
                label,
                items,
                render_item,
                on_add,
            } = config;
            let items_len = items.len();

            ui.add_space(12.0);
            egui::Grid::new(format!("move_list_{}_heading", config.label))
                .num_columns(2)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.heading(label);
                    if ui.button("Bæta við").clicked() {
                        on_add();
                    }
                });

            egui::Grid::new(format!("move_list_{}", config.label))
                .num_columns(4)
                .spacing([1.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    for i in 0..items_len {
                        let song = match items.get_mut(i) {
                            Some(i) => i,
                            None => return,
                        };
                        render_item(ui, i, song);

                        if ui.button("Upp").clicked() {
                            if i != 0 {
                                items.swap(i, i - 1);
                                written = true;
                            }
                        }
                        if ui.button("Niður").clicked() {
                            if i + 1 < items.len() {
                                items.swap(i, i + 1);
                                written = true;
                            }
                        }
                        if ui.button("Eyða").clicked() {
                            items.remove(i);
                            written = true;
                        }
                        ui.end_row();
                    }
                });
        })
        .response;
    if written {
        response.mark_changed();
    }
    response
}

fn generate_pdf(book: &BookConfig) {
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

trait WriteOnResponseChange {
    fn write(&self, state: &State);
}

impl WriteOnResponseChange for egui::Response {
    fn write(&self, state: &State) {
        if self.changed() {
            state.write_settings();
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PageLocation {
    Front,
    Back,
}

#[derive(Debug, Default)]
struct State {
    book: BookConfig,
    add_song: AddSong,
    add_page: Option<PageLocation>,
}

impl State {
    fn write_settings(&self) {
        fs::write(
            "settings.json",
            serde_json::to_string_pretty(&self.book).unwrap(),
        )
        .unwrap();
    }
    fn update_generate_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Skáta Söngbókin Þín");

            ui.checkbox(&mut self.book.reorder_pages, "Reorder Pages for Printing")
                .write(self);
            egui::ComboBox::from_label("Preferred Font")
                .selected_text(format!("{}", self.book.preferred_font))
                .show_ui(ui, |ui| {
                    for font in AVAILABLE_FONTS {
                        ui.selectable_value(&mut self.book.preferred_font, font.to_owned(), font);
                    }
                })
                .response
                .write(self);
            update_move_list(
                ui,
                ItemListConfig {
                    label: "Lög",
                    items: &mut self.book.songs,
                    render_item: |ui, _, song| ui.label(&song.title),
                    on_add: || self.add_song.open(),
                },
            )
            .write(self);
            update_move_list(
                ui,
                ItemListConfig {
                    label: "Framsíður",
                    items: &mut self.book.front_pages,
                    render_item: |ui, i, page| {
                        ui.push_id(format!("f_{}", i), |ui| view_page(ui, page))
                            .response
                    },
                    on_add: || self.add_page = Some(PageLocation::Front),
                },
            )
            .write(self);
            update_move_list(
                ui,
                ItemListConfig {
                    label: "Baksíður",
                    items: &mut self.book.back_pages,
                    render_item: |ui, i, page| {
                        ui.push_id(format!("b_{}", i), |ui| view_page(ui, page))
                            .response
                    },
                    on_add: || self.add_page = Some(PageLocation::Back),
                },
            )
            .write(self);

            if ui.button("Generate PDF").clicked() {
                generate_pdf(&self.book);
            }

            if let Some(song) = self.add_song.ui(ui) {
                self.book.songs.push(song);
                self.write_settings();
            }
            if let Some(location) = self.add_page {
                let window_title = match location {
                    PageLocation::Front => "Bæta við forsíðu",
                    PageLocation::Back => "Bæta við baksíðu",
                };
                egui::Window::new(window_title).show(ctx, |ui| {
                    if ui.button("Hætta við").clicked() {
                        self.add_page = None;
                    }

                    let options = [
                        ("Forsíða", Page::FrontPage(FrontPage::default())),
                        ("Formáli", Page::Preface(Preface::default())),
                        (
                            "Efnisyfirlit",
                            Page::TableOfContents(TableOfContents::default()),
                        ),
                    ];
                    for option in options {
                        if ui.button(option.0).clicked() {
                            add_to_page(&mut self.book, location, option.1);
                            self.write_settings();
                            self.add_page = None;
                        }
                    }
                });
            }
        });
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_generate_panel(ctx, ui);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.write_settings();
    }
}

fn add_to_page(book: &mut BookConfig, location: PageLocation, page: Page) {
    match location {
        PageLocation::Front => book.front_pages.push(page),
        PageLocation::Back => book.back_pages.push(page),
    }
}

fn view_page(ui: &mut egui::Ui, page: &mut generator::config::Page) -> egui::Response {
    ui.vertical(|ui| {
        ui.set_min_size(egui::vec2(200.0, 4.0));
        match page {
            generator::config::Page::Preface(p) => {
                ui.label("Formáli");
                ui.text_edit_singleline(&mut p.title);
                ui.text_edit_multiline(&mut p.body);
            }
            generator::config::Page::TableOfContents(p) => {
                ui.label("Efnisyfirlit");
                ui.text_edit_singleline(&mut p.title);
                egui::ComboBox::from_label("Flokkunar röð")
                    .selected_text(format!("{}", p.order))
                    .show_ui(ui, |ui| {
                        let sort_orders = [
                            TableOfContentsSortOrder::SongNumber,
                            TableOfContentsSortOrder::Alphabetical,
                        ];
                        for order in sort_orders {
                            ui.selectable_value(&mut p.order, order, format!("{}", order));
                        }
                    });
            }
            generator::config::Page::FrontPage(p) => {
                ui.label("Forsíða");
                ui.text_edit_singleline(&mut p.title);
                ui.text_edit_singleline(&mut p.version);
            }
        }
    })
    .response
}

fn load_book() -> BookConfig {
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

pub fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    let mut state = State::default();
    state.book = load_book();

    eframe::run_native(
        "Skáta Söngbókin Þín",
        options,
        Box::new(|_cc| Box::new(state)),
    )
}
