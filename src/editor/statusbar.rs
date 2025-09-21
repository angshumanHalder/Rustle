use std::path::Path;

use super::{
    DocumentStatus,
    terminal::{Position, Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct StatusBar {
    current_status: DocumentStatus,
    size: Size,
    needs_redraw: bool,
}

impl StatusBar {
    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.current_status != status {
            self.current_status = status;
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

impl UIComponent for StatusBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        true
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        if let Ok(size) = Terminal::size() {
            let (beginning, end) = self.format_status();
            let remainder_len = (self.size.width as usize).saturating_sub(beginning.len());
            let status = format!("{beginning}{end:>remainder_len$}");
            let to_print = if status.len() <= size.width as usize {
                status
            } else {
                String::new()
            };
            let _ = Terminal::move_cursor(Position {
                row: origin_y as u16,
                ..Default::default()
            });
            let result = Terminal::print_inverted_row(&to_print);
            debug_assert!(result.is_ok(), "Failed to render status bar");
        }
        Ok(())
    }
}
