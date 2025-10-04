use super::{
    terminal::{Position, Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct SearchBar {
    search_query: String,
    needs_redraw: bool,
}

impl SearchBar {
    pub fn update_search_query(&mut self, search_query: String) {
        self.search_query = search_query;
        self.mark_redraw(true);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn get_position(&self, size: Size) -> Position {
        Position {
            row: size.height.saturating_sub(1), // Last line of the screen
            col: ("Search: ".len() + self.search_query.len()) as u16,
        }
    }

    pub fn push_char(&mut self, ch: char) {
        self.search_query.push(ch);
        self.mark_redraw(true);
    }

    pub fn pop_char(&mut self) {
        self.search_query.pop();
        self.mark_redraw(true);
    }

    pub fn clear(&mut self) {
        self.search_query = String::new();
    }

    pub fn get_query(&self) -> String {
        self.search_query.clone()
    }
}

impl UIComponent for SearchBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, _: super::terminal::Size) {}

    #[allow(clippy::cast_possible_truncation)]
    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        Terminal::move_cursor(Position {
            row: origin_y as u16,
            ..Default::default()
        })?;
        let text = format!("Search: {}", self.search_query);
        let _ = Terminal::clear_line();
        Terminal::print(text.as_str())
    }
}
