use config::ItemListConfig;
use serde_json;
use std::fs;

use generator::config::{
    BookConfig, FrontPage, Page, Preface, TableOfContents, TableOfContentsSortOrder,
};

use eframe::egui;

mod config;
mod elements;
mod helpers;

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
    add_song: elements::AddSong,
    add_page: Option<PageLocation>,
    select_font: elements::SelectFont,
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
        let scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]);
        scroll_area.show(ui, |ui| {
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
                    for (label, page) in options {
                        if ui.button(label).clicked() {
                            match location {
                                PageLocation::Front => self.book.front_pages.push(page),
                                PageLocation::Back => self.book.back_pages.push(page),
                            }
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
        egui::SidePanel::new(egui::panel::Side::Left, "presets_panel")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Þínar Bækur");
                ui.heading("Standard Bækur");
            });
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "generate_panel")
            .min_height(8.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        egui::Grid::new("gen_settings").show(ui, |ui| {
                            ui.label("Endurraða síðum");
                            ui.checkbox(&mut self.book.reorder_pages, "").write(self);
                            ui.end_row();

                            ui.label("Leturgerð");
                            self.select_font
                                .ui(ui, &mut self.book.preferred_font)
                                .write(self);
                            ui.end_row();
                        });
                    });
                    ui.centered_and_justified(|ui| {
                        if ui.button("Búa til PDF").clicked() {
                            helpers::generate_pdf(&self.book);
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_generate_panel(ctx, ui);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.write_settings();
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

fn update_move_list<T, A, B>(ui: &mut egui::Ui, config: ItemListConfig<T, A, B>) -> egui::Response
where
    T: Clone,
    A: FnOnce(),
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

pub fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 640.0)),
        min_window_size: Some(egui::vec2(640.0, 640.0)),
        ..Default::default()
    };

    let mut state = State::default();
    state.book = helpers::load_book();

    eframe::run_native(
        "Skáta Söngbókin Þín",
        options,
        Box::new(|_cc| Box::new(state)),
    )
}
