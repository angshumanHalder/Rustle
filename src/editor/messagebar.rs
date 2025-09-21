use super::{
    terminal::{Position, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct MessageBar {
    current_message: String,
    needs_redraw: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, new_message: String) {
        if new_message != self.current_message {
            self.current_message = new_message;
        }
    }
}

impl UIComponent for MessageBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        true
    }

    fn set_size(&mut self, _: super::terminal::Size) {}

    #[allow(clippy::cast_possible_truncation)]
    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        Terminal::move_cursor(Position {
            row: origin_y as u16,
            ..Default::default()
        })?;
        Terminal::print(&self.current_message)
    }
}
