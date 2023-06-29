use iced::Element;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddOption<A> {
    pub title: String,
    pub value: A,
}

impl<A> std::fmt::Display for AddOption<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl<A> AddOption<A> {
    pub fn new(title: String, value: A) -> Self {
        return Self { title, value };
    }
}

pub struct ItemListConfig<'a, T, A>
where
    T: Clone,
    A: Clone,
{
    pub label: &'a str,
    pub items: &'a [T],
    pub render_item: Box<dyn Fn(&'a T) -> Element<'a, crate::Message>>,
    pub add_options: Vec<AddOption<A>>,
    pub on_add: Box<dyn Fn(A) -> crate::Message>,
    pub on_move: Box<dyn Fn(usize, usize) -> crate::Message>,
    pub on_remove: Box<dyn Fn(usize) -> crate::Message>,
}
