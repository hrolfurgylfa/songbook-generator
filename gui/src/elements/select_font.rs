use eframe::egui;

const AVAILABLE_FONTS: [&str; 2] = ["Roboto", "RobotoStripped"];

#[derive(Debug, Default)]
pub struct SelectFont {}

impl SelectFont {
    pub fn ui(&mut self, ui: &mut egui::Ui, selected: &mut String) -> egui::Response {
        egui::ComboBox::from_label("")
            .selected_text(format!("{}", selected))
            .show_ui(ui, |ui| {
                for font in AVAILABLE_FONTS {
                    ui.selectable_value(selected, font.to_owned(), font);
                }
            })
            .response
    }
}
