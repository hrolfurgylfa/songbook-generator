use eframe::egui;

#[derive(Debug)]
pub struct SelectFont {
    fonts: Vec<String>,
}

impl Default for SelectFont {
    fn default() -> Self {
        SelectFont {
            fonts: generator::fonts::get_fonts().unwrap(),
        }
    }
}

impl SelectFont {
    pub fn ui(&mut self, ui: &mut egui::Ui, selected: &mut String) -> egui::Response {
        egui::ComboBox::from_label("")
            .selected_text(format!("{}", selected))
            .show_ui(ui, |ui| {
                for font in &self.fonts {
                    ui.selectable_value(selected, font.to_owned(), font);
                }
            })
            .response
    }
}
