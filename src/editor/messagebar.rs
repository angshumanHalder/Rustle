use std::time::{Duration, Instant};

use super::{
    terminal::{Position, Terminal},
    uicomponent::UIComponent,
};

const DEFAULT_DURATION: Duration = Duration::new(5, 0);

struct Message {
    text: String,
    time: Instant,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            text: String::new(),
            time: Instant::now(),
        }
    }
}

impl Message {
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.time) > DEFAULT_DURATION
    }
}

#[derive(Default)]
pub struct MessageBar {
    current_message: Message,
    needs_redraw: bool,
    cleared: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, new_message: String) {
        self.current_message = Message {
            text: new_message,
            time: Instant::now(),
        };
        self.cleared = false;
        self.mark_redraw(true);
    }
}

impl UIComponent for MessageBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.current_message.is_expired() || self.needs_redraw
    }

    fn set_size(&mut self, _: super::terminal::Size) {}

    #[allow(clippy::cast_possible_truncation)]
    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        if self.current_message.is_expired() {
            self.cleared = false;
        }
        let message = if self.current_message.is_expired() {
            "HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = search"
        } else {
            &self.current_message.text
        };
        Terminal::move_cursor(Position {
            row: origin_y as u16,
            ..Default::default()
        })?;
        let _ = Terminal::clear_line();
        Terminal::print(message)
    }
}
