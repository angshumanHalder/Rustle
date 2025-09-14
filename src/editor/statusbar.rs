use std::path::Path;

use super::{
    DocumentStatus,
    terminal::{Position, Size, Terminal},
};

pub struct StatusBar {
    current_status: DocumentStatus,
    margin_bottom: usize,
    width: usize,
    position_y: usize,
    needs_redraw: bool,
    is_visible: bool,
}

impl StatusBar {
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        let mut status_bar = Self {
            current_status: DocumentStatus::default(),
            margin_bottom,
            width: size.width as usize,
            position_y: 0,
            needs_redraw: true,
            is_visible: false,
        };
        status_bar.resize(size);
        status_bar
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn resize(&mut self, size: Size) {
        self.width = size.width as usize;
        let mut position_y = 0;
        let mut is_visible = false;
        if let Some(result) = size
            .height
            .checked_sub(self.margin_bottom as u16)
            .and_then(|result| result.checked_sub(1))
        {
            position_y = result;
            is_visible = true;
        }
        self.position_y = position_y as usize;
        self.is_visible = is_visible;
        self.needs_redraw = true;
    }

    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.current_status != status {
            self.current_status = status;
            self.needs_redraw = true;
        }
    }

    pub fn redraw(&mut self) {
        self.needs_redraw = true;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn render(&mut self) {
        if !self.needs_redraw || !self.is_visible {
            return;
        }
        if let Ok(size) = Terminal::size() {
            let (beginning, end) = self.format_status();
            let remainder_len = self.width.saturating_sub(beginning.len());
            let status = format!("{beginning}{end:>remainder_len$}");
            let to_print = if status.len() <= size.width as usize {
                status
            } else {
                String::new()
            };
            let _ = Terminal::move_cursor(Position {
                row: self.position_y as u16,
                ..Default::default()
            });
            let result = Terminal::print_inverted_row(&to_print);
            debug_assert!(result.is_ok(), "Failed to render status bar");
            self.needs_redraw = false;
        }
    }

    fn format_status(&self) -> (String, String) {
        let file_name = self
            .current_status
            .path
            .as_ref()
            .and_then(|p| Path::new(p).file_name())
            .and_then(|s| s.to_str())
            .map_or_else(|| String::from("[No Name]"), String::from);

        let end = format!(
            "{}/{}",
            self.current_status.current_line_idx.saturating_add(1),
            self.current_status.total_lines
        );
        let mut beginning = format!("{} - {} lines", file_name, self.current_status.total_lines);
        if self.current_status.is_modified {
            beginning = format!("{beginning} (modified)");
        }
        (beginning, end)
    }
}
