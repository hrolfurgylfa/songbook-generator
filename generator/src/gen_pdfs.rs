use genpdf::Alignment;
use genpdf::Element as _;
use genpdf::{elements, fonts, style};

use crate::config::TableOfContentsSortOrder;
use crate::config::{FrontPage, Page, Preface, Song, TableOfContents};
use crate::fonts::FontError;

const TITLE_FONT_SIZE: u8 = 36;
const SONG_TITLE_FONT_SIZE: u8 = 28;
const BODY_FONT_SIZE: u8 = 24;

type Font = fonts::FontFamily<fonts::FontData>;
pub fn load_font(font_name: &str) -> Result<Font, FontError> {
    crate::fonts::get_font(font_name)
}

fn get_empty_pdf(font: &Font) -> genpdf::Document {
    // Configure the document
    let mut doc = genpdf::Document::new(font.clone());
    doc.set_title("temp");
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);

    // Add the line around the page
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    // decorator.set_header(|page| {
    //     let mut layout = elements::LinearLayout::vertical();
    //     if page > 1 {
    //         layout.push(
    //             elements::Paragraph::new(format!("Page {}", page)).aligned(Alignment::Center),
    //         );
    //         layout.push(elements::Break::new(1));
    //     }
    //     layout.styled(style::Style::new().with_font_size(10))
    // });
    doc.set_page_decorator(decorator);
    return doc;
}

pub fn pdf_to_bytes(doc: genpdf::Document) -> Vec<u8> {
    // Render the PDF to bytes
    let mut bytes = Vec::new();
    doc.render(&mut bytes).expect("Failed to render PDF");
    return bytes;
}

pub fn generate_songs(font: &Font, songs: &[Song]) -> Vec<u8> {
    let mut doc = get_empty_pdf(font);

    for (i, song) in songs.iter().enumerate() {
        // Generate the title on the first page
        let mut layout = elements::GlueLayout::vertical();
        layout.push(
            elements::Paragraph::new(format!("{}. {}", i + 1, song.title))
                .aligned(Alignment::Center)
                .styled(style::Style::new().with_font_size(SONG_TITLE_FONT_SIZE)),
        );

        // Generate the song lines
        for (_, verse) in song.body.iter().enumerate() {
            for line in verse.lines() {
                layout.push(
                    elements::Text::new(line)
                        .styled(style::Style::new().with_font_size(BODY_FONT_SIZE)),
                );
            }
            layout.push(elements::Break::new(1.5));
            doc.push(layout);
            layout = elements::GlueLayout::vertical();
        }

        doc.push(elements::Break::new(1.0));
    }

    return pdf_to_bytes(doc);
}

pub fn generate_preface(doc: &mut genpdf::Document, preface: &Preface) {
    doc.push(
        elements::Paragraph::new(&preface.title)
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(TITLE_FONT_SIZE)),
    );
    doc.push(elements::Break::new(1.5));
    for line in preface.body.lines() {
        doc.push(
            elements::Paragraph::new(line)
                .aligned(Alignment::Left)
                .styled(style::Style::new().with_font_size(BODY_FONT_SIZE)),
        );
    }
}

pub fn generate_front_page(doc: &mut genpdf::Document, front_page: &FrontPage) {
    doc.push(
        elements::Paragraph::new(&front_page.title)
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(TITLE_FONT_SIZE)),
    );
    doc.push(elements::Break::new(1.5));
    doc.push(
        elements::Paragraph::new(&front_page.version)
            .aligned(Alignment::Center)
            .styled(style::Style::new().with_font_size(BODY_FONT_SIZE)),
    );
}

pub fn generate_table_of_contents(
    doc: &mut genpdf::Document,
    songs: &[Song],
    toc: &TableOfContents,
) {
    let mut songs_and_numbers = songs
        .iter()
        .enumerate()
        .map(|(num, song)| (num + 1, song))
        .collect::<Vec<_>>();
    match toc.order {
        TableOfContentsSortOrder::SongNumber => {}
        TableOfContentsSortOrder::Alphabetical => {
            songs_and_numbers.sort_by(|a, b| a.1.title.cmp(&b.1.title));
        }
    };
    let format_song_title: Box<dyn Fn(usize, &str) -> String> = Box::new(match toc.order {
        TableOfContentsSortOrder::SongNumber => |num, title| format!("{}. {}", num, title),
        TableOfContentsSortOrder::Alphabetical => |num, title| format!("{}. {}", title, num),
    });

    doc.push(
        elements::Paragraph::new(&toc.title)
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(TITLE_FONT_SIZE)),
    );
    doc.push(elements::Break::new(1.5));
    for (num, song) in songs_and_numbers {
        doc.push(
            elements::Text::new(format_song_title(num, &song.title))
                .styled(style::Style::new().with_font_size(BODY_FONT_SIZE)),
        );
    }
}

pub fn generate_page(font: &Font, songs: &[Song], page: &Page) -> Vec<u8> {
    let mut doc = get_empty_pdf(font);

    match page {
        Page::Preface(preface) => generate_preface(&mut doc, preface),
        Page::FrontPage(front_page) => generate_front_page(&mut doc, front_page),
        Page::TableOfContents(table_of_contents) => {
            generate_table_of_contents(&mut doc, songs, table_of_contents)
        }
    }

    return pdf_to_bytes(doc);
}
