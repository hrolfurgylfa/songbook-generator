use serde::{Deserialize, Serialize};

use crate::tile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preface {
    pub title: String,
    pub body: String,
}

impl Default for Preface {
    fn default() -> Self {
        Preface {
            title: "Formáli".to_owned(),
            body: "".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableOfContentsSortOrder {
    Alphabetical,
    SongNumber,
}

impl std::fmt::Display for TableOfContentsSortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::SongNumber => "Laganúmer",
            Self::Alphabetical => "Stafrófsröð",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableOfContents {
    pub title: String,
    pub order: TableOfContentsSortOrder,
}

impl Default for TableOfContents {
    fn default() -> Self {
        TableOfContents {
            title: "Efnisyfirlit".to_owned(),
            order: TableOfContentsSortOrder::SongNumber,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontPage {
    pub title: String,
    pub version: String,
}

impl Default for FrontPage {
    fn default() -> Self {
        FrontPage {
            title: "Þín Skáta Söngbók".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Page {
    Preface(Preface),
    TableOfContents(TableOfContents),
    FrontPage(FrontPage),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    #[serde(skip)]
    pub body: Vec<String>,
}

impl std::fmt::Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

fn default_padding() -> u8 {
    10
}

fn default_add_separator() -> bool {
    true
}

fn default_tiled_page_size() -> tile::PageSize {
    tile::PageSize::A7
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookConfig {
    pub front_pages: Vec<Page>,
    pub back_pages: Vec<Page>,
    pub songs: Vec<Song>,
    pub preferred_font: String,
    pub reorder_pages: bool,
    #[serde(default = "default_padding")]
    pub padding: u8,
    #[serde(default = "default_add_separator")]
    pub add_separator: bool,
    #[serde(default = "default_tiled_page_size")]
    pub tiled_page_size: tile::PageSize,
}

impl Default for BookConfig {
    fn default() -> Self {
        BookConfig {
            front_pages: vec![],
            back_pages: vec![],
            songs: vec![],
            preferred_font: "Arial".to_owned(),
            reorder_pages: true,
            padding: default_padding(),
            add_separator: default_add_separator(),
            tiled_page_size: default_tiled_page_size(),
        }
    }
}
