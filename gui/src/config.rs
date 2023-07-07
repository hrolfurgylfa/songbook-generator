use eframe::egui;

pub struct ItemListConfig<'a, T: Clone, A, B>
where
    A: FnOnce(),
    B: Fn(&mut egui::Ui, usize, &mut T) -> egui::Response,
{
    pub label: &'a str,
    pub items: &'a mut Vec<T>,
    pub render_item: B,
    pub on_add: A,
}
