use eframe::egui;

pub struct ItemListConfig<'a, T: Clone, A: FnOnce()> {
    pub label: &'a str,
    pub items: &'a mut Vec<T>,
    pub render_item: Box<dyn Fn(&mut egui::Ui, usize, &mut T) -> egui::Response>,
    pub on_add: A,
}
