use eframe::egui;
use generator::tile;

#[derive(Debug, Default)]
pub struct SelectTilePageSize {}

impl SelectTilePageSize {
    pub fn ui(&mut self, ui: &mut egui::Ui, selected: &mut tile::PageSize) -> egui::Response {
        egui::ComboBox::from_label("")
            .selected_text(format!("{}", selected))
            .show_ui(ui, |ui| {
                for font in tile::PAGE_SIZE_VARIANTS {
                    ui.selectable_value(selected, *font, format!("{}", *font));
                }
            })
            .response
    }
}
