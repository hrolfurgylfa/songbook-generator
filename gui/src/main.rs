use config::ItemListConfig;
use serde_json;
use std::fs;
use std::path::Path;

use generator::config::BookConfig;

use eframe::egui;

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

fn _get_available_songs() -> Vec<String> {
    let mut songs = Vec::new();
    for song in fs::read_dir("./songs").unwrap() {
        let path = song.unwrap().path();
        let title = path.file_stem().unwrap();
        songs.push(title.to_string_lossy().into_owned());
    }
    songs
}

fn update_move_list<T: Clone>(ui: &mut egui::Ui, config: ItemListConfig<T>) -> egui::Response {
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

            ui.horizontal(|ui| {
                ui.label(label);
                if ui.button("Bæta við lagi").clicked() {
                    items.extend(on_add(ui));
                    written = true;
                }
            });

            for i in 0..items_len {
                ui.horizontal(|ui| {
                    let song = match items.get_mut(i) {
                        Some(i) => i,
                        None => return,
                    };
                    render_item(ui, i, song);
                    if ui.button("^").clicked() {
                        println!("Move song up: {}", i);
                        if i != 0 {
                            items.swap(i, i - 1);
                            written = true;
                        }
                    }
                    if ui.button("v").clicked() {
                        println!("Move song down: {}", i);
                        if i + 1 < items.len() {
                            items.swap(i, i + 1);
                            written = true;
                        }
                    }
                    if ui.button("x").clicked() {
                        println!("Remove song: {}", i);
                        items.remove(i);
                        written = true;
                    }
                });
            }
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

#[derive(Debug, Default)]
struct State {
    book: BookConfig,
}

impl State {
    fn write_settings(&self) {
        fs::write(
            "settings.json",
            serde_json::to_string_pretty(&self.book).unwrap(),
        )
        .unwrap();
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                    label: "Songs",
                    items: &mut self.book.songs,
                    render_item: Box::new(|ui, _, song| ui.label(&song.title)),
                    on_add: Box::new(|_| match generator::load_song("Bakpokinn") {
                        Ok(song) => vec![song],
                        Err(e) => {
                            println!("Failed to load song: {}", e);
                            vec![]
                        }
                    }),
                },
            )
            .write(self);
            update_move_list(
                ui,
                ItemListConfig {
                    label: "Framsíður",
                    items: &mut self.book.front_pages,
                    render_item: Box::new(|ui, _, page| view_page(ui, page)),
                    on_add: Box::new(|_| vec![]),
                },
            )
            .write(self);

            if ui.button("Generate PDF").clicked() {
                generate_pdf(&self.book);
            }
        });
    }
}

fn view_page(ui: &mut egui::Ui, page: &mut generator::config::Page) -> egui::Response {
    ui.vertical(|ui| match page {
        generator::config::Page::Preface(p) => {
            ui.label("Formáli");
            ui.text_edit_singleline(&mut p.title);
            ui.text_edit_multiline(&mut p.body);
        }
        generator::config::Page::TableOfContents(p) => {
            ui.label("Efnisyfirlit");
            ui.text_edit_singleline(&mut p.title);
        }
        generator::config::Page::FrontPage(p) => {
            ui.label("Forsíða");
            ui.text_edit_singleline(&mut p.title);
            ui.text_edit_singleline(&mut p.version);
        }
    })
    .response
}
//
// impl State {
//     fn view_settings<'a>(
//         &'a self,
//         options: Vec<(&str, Element<'a, Message>)>,
//         end_elements: Vec<Element<'a, Message>>,
//     ) -> Element<'a, Message> {
//         let mut settings = Column::new();
//         for (label, element) in options.into_iter() {
//             settings = settings.push(
//                 row![
//                     text(label).width(Length::Shrink),
//                     container(element)
//                         .width(Length::Fill)
//                         .align_x(Horizontal::Right)
//                 ], // .width(Length::Fixed(550.0)),
//             );
//         }
//         for element in end_elements.into_iter() {
//             settings = settings.push(element);
//         }
//
//         settings
//             .align_items(Alignment::Center)
//             .padding(10)
//             .spacing(22)
//             .into()
//     }
//
//     fn view_item_list<'a, T, A>(
//         &self,
//         config: config::ItemListConfig<'a, T, A>,
//     ) -> Element<'a, Message>
//     where
//         T: Clone + std::cmp::Eq,
//         A: Clone + std::cmp::Eq + 'static,
//     {
//         let config::ItemListConfig {
//             label,
//             items,
//             render_item,
//             add_options,
//             on_add,
//             on_move,
//             on_remove,
//         } = config;
//
//         let mut songs = Column::new();
//
//         for (i, item) in items.iter().enumerate() {
//             songs = songs.push(row![
//                 container(render_item(i, item)).width(Length::Fill),
//                 button("^").on_press(on_move(i, i.saturating_sub(1))),
//                 button("v").on_press(on_move(i, i + 1)),
//                 button("x").on_press(on_remove(i)),
//             ]);
//         }
//
//         column![
//             row![
//                 text(label),
//                 container(
//                     container(pick_list(add_options, None, move |opt| on_add(opt.value)))
//                         .width(Length::Shrink)
//                 )
//                 .align_x(Horizontal::Right)
//                 .width(Length::Fill),
//             ],
//             songs,
//         ]
//         .into()
//     }
// }

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
    eframe::run_native(
        "Skáta Söngbókin Þín",
        options,
        Box::new(|_cc| Box::new(State { book: load_book() })),
    )
}
