use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Preface {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum TableOfContentsSortOrder {
    Alphabetical,
    SongNumber,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableOfContents {
    pub title: String,
    pub order: TableOfContentsSortOrder,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrontPage {
    pub title: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Page {
    Preface(Preface),
    TableOfContents(TableOfContents),
    FrontPage(FrontPage),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Song {
    pub title: String,
    pub body: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookConfig {
    pub front_pages: Vec<Page>,
    pub back_pages: Vec<Page>,
    pub songs: Vec<Song>,
    pub preferred_font: String,
}
