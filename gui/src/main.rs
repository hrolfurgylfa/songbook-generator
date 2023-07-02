use config::AddOption;
use iced::alignment::Horizontal;
use serde_json;
use std::path::Path;
use std::{fs, future};

use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input, Column};
use iced::{executor, Alignment, Application, Command, Element, Length, Settings, Theme};

use generator::config::{BookConfig, FrontPage, Page, Preface, TableOfContents};

mod config;

#[derive(Debug, Default)]
struct State {
    book: BookConfig,
}

#[derive(Debug, Clone, Copy)]
pub enum PageLoc {
    Front,
    Back,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeFont(String),
    ChangeReorderPages(bool),
    WriteSettings,
    GeneratePdf,
    AddSong(String),
    MoveSong(usize, usize),
    RemoveSong(usize),
    AddFrontPage(Page),
    MoveFrontPage(usize, usize),
    RemoveFrontPage(usize),
    ChangePageTitle(PageLoc, usize, String),
}

#[must_use]
fn write_settings() -> Command<Message> {
    Command::perform(future::ready(()), |()| Message::WriteSettings)
}

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

fn get_available_songs() -> Vec<String> {
    let mut songs = Vec::new();
    for song in fs::read_dir("./songs").unwrap() {
        let path = song.unwrap().path();
        let title = path.file_stem().unwrap();
        songs.push(title.to_string_lossy().into_owned());
    }
    songs
}

impl Application for State {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
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
        (Self { book }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Skáta Söngbókin Þín")
    }

    fn view<'a>(&'a self) -> Element<Message> {
        let content = self.view_settings(
            vec![
                (
                    "Reorder Pages for Printing",
                    checkbox("", self.book.reorder_pages, |b| {
                        Message::ChangeReorderPages(b)
                    })
                    .into(),
                ),
                (
                    "Preferred Font",
                    pick_list(
                        &AVAILABLE_FONTS[..],
                        Some(
                            AVAILABLE_FONTS
                                .iter()
                                .find(|f| *f == &self.book.preferred_font)
                                .unwrap_or_else(|| {
                                    println!(
                                        "Font \"{}\" not found in view!",
                                        self.book.preferred_font
                                    );
                                    &DEFAULT_FONT
                                }),
                        ),
                        |f| Message::ChangeFont(f.to_owned()),
                    )
                    .into(),
                ),
            ],
            vec![
                self.view_item_list(config::ItemListConfig {
                    label: "Songs",
                    items: &self.book.songs,
                    render_item: Box::new(|_, s| text(&s.title).into()),
                    add_options: get_available_songs()
                        .into_iter()
                        .map(|s| AddOption::new(s.clone(), s))
                        .collect(),
                    on_add: Box::new(|s| Message::AddSong(s)),
                    on_move: Box::new(|from, to| Message::MoveSong(from, to)),
                    on_remove: Box::new(|i| Message::RemoveSong(i)),
                }),
                self.view_item_list(config::ItemListConfig::<'a> {
                    label: "Front Pages",
                    items: &self.book.front_pages,
                    render_item: Box::new(|i, p| view_page(PageLoc::Front, i, p)),
                    add_options: vec![
                        AddOption::new(
                            "Efnisyfirlit".to_owned(),
                            Page::TableOfContents(TableOfContents::default()),
                        ),
                        AddOption::new("Formáli".to_owned(), Page::Preface(Preface::default())),
                        AddOption::new("Forsíða".to_owned(), Page::FrontPage(FrontPage::default())),
                    ],
                    on_add: Box::new(|s| Message::AddFrontPage(s)),
                    on_move: Box::new(|from, to| Message::MoveFrontPage(from, to)),
                    on_remove: Box::new(|i| Message::RemoveFrontPage(i)),
                }),
                container(button(text("Generate")).on_press(Message::GeneratePdf))
                    .center_x()
                    .into(),
            ],
        );

        container(container(content).width(Length::Fixed(500.0)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::ChangePageTitle(loc, i, new_name) => {
                match loc {
                    PageLoc::Front => change_page_name(&mut self.book.front_pages[i], new_name),
                    PageLoc::Back => change_page_name(&mut self.book.back_pages[i], new_name),
                };
                write_settings()
            }
            Message::AddFrontPage(page) => {
                self.book.front_pages.push(page);
                write_settings()
            }
            Message::MoveFrontPage(from, to) => {
                let elem = self.book.front_pages.remove(from);
                let new_index = if to >= from { to - 1 } else { to };
                self.book.front_pages.insert(new_index, elem);
                write_settings()
            }
            Message::RemoveFrontPage(i) => {
                self.book.front_pages.remove(i);
                write_settings()
            }
            Message::AddSong(title) => {
                let song = match generator::load_song(&title) {
                    Ok(song) => song,
                    Err(e) => {
                        println!("Failed to load song: {}", e);
                        return Command::none();
                    }
                };
                self.book.songs.push(song);
                write_settings()
            }
            Message::MoveSong(from, to) => {
                let elem = self.book.songs.remove(from);
                let new_index = if to >= from { to - 1 } else { to };
                self.book.songs.insert(new_index, elem);
                write_settings()
            }
            Message::RemoveSong(i) => {
                self.book.songs.remove(i);
                write_settings()
            }
            Message::GeneratePdf => {
                // Make sure we have some pages to generate
                if self.book.songs.is_empty()
                    && self.book.front_pages.is_empty()
                    && self.book.back_pages.is_empty()
                {
                    println!("No songs or pages to generate!");
                    return Command::none();
                }

                // Generate the songbook PDF
                let pdf = match generator::generate_book_pdf(&self.book) {
                    Ok(pdf) => pdf,
                    Err(e) => {
                        println!("Error generating PDF: {}", e);
                        return Command::none();
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

                Command::none()
            }
            Message::ChangeFont(f) => {
                if !font_exists(&f) {
                    println!("Font \"{}\" not found; ignoring", f);
                    return Command::none();
                }
                self.book.preferred_font = f;
                write_settings()
            }
            Message::ChangeReorderPages(b) => {
                self.book.reorder_pages = b;
                write_settings()
            }
            Message::WriteSettings => {
                fs::write(
                    "settings.json",
                    serde_json::to_string_pretty(&self.book).unwrap(),
                )
                .unwrap();
                Command::none()
            }
        }
    }
}

fn change_page_name(page: &mut Page, new_name: String) {
    match page {
        Page::Preface(p) => {
            p.title = new_name;
        }
        Page::FrontPage(p) => {
            p.title = new_name;
        }
        Page::TableOfContents(p) => {
            p.title = new_name;
        }
    }
}

fn view_page(loc: PageLoc, i: usize, page: &generator::config::Page) -> Element<Message> {
    match page {
        generator::config::Page::Preface(p) => column![
            text("Title:"),
            text_input("Formáli", &p.title)
                .on_input(move |title| Message::ChangePageTitle(loc, i, title)),
        ]
        .into(),
        generator::config::Page::TableOfContents(p) => text(&p.title).into(),
        generator::config::Page::FrontPage(p) => text(&p.title).into(),
    }
}

impl State {
    fn view_settings<'a>(
        &'a self,
        options: Vec<(&str, Element<'a, Message>)>,
        end_elements: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut settings = Column::new();
        for (label, element) in options.into_iter() {
            settings = settings.push(
                row![
                    text(label).width(Length::Shrink),
                    container(element)
                        .width(Length::Fill)
                        .align_x(Horizontal::Right)
                ], // .width(Length::Fixed(550.0)),
            );
        }
        for element in end_elements.into_iter() {
            settings = settings.push(element);
        }

        settings
            .align_items(Alignment::Center)
            .padding(10)
            .spacing(22)
            .into()
    }

    fn view_item_list<'a, T, A>(
        &self,
        config: config::ItemListConfig<'a, T, A>,
    ) -> Element<'a, Message>
    where
        T: Clone + std::cmp::Eq,
        A: Clone + std::cmp::Eq + 'static,
    {
        let config::ItemListConfig {
            label,
            items,
            render_item,
            add_options,
            on_add,
            on_move,
            on_remove,
        } = config;

        let mut songs = Column::new();

        for (i, item) in items.iter().enumerate() {
            songs = songs.push(row![
                container(render_item(i, item)).width(Length::Fill),
                button("^").on_press(on_move(i, i.saturating_sub(1))),
                button("v").on_press(on_move(i, i + 1)),
                button("x").on_press(on_remove(i)),
            ]);
        }

        column![
            row![
                text(label),
                container(
                    container(pick_list(add_options, None, move |opt| on_add(opt.value)))
                        .width(Length::Shrink)
                )
                .align_x(Horizontal::Right)
                .width(Length::Fill),
            ],
            songs,
        ]
        .into()
    }
}

pub fn main() -> Result<(), String> {
    State::run(Settings::default()).map_err(|e| e.to_string())
}
