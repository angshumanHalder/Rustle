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
}

impl StatusBar {
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        Self {
            current_status: DocumentStatus::default(),
            margin_bottom,
            width: size.width as usize,
            position_y: size
                .height
                .saturating_sub(margin_bottom as u16)
                .saturating_sub(1) as usize,
            needs_redraw: true,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn resize(&mut self, size: Size) {
        self.width = size.width as usize;
        self.position_y = size
            .height
            .saturating_sub(self.margin_bottom as u16)
            .saturating_sub(1) as usize;
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
        if !self.needs_redraw {
            return;
        }
        let mut status = format!("{:?}", self.current_status);
        status.truncate(self.width);
        let _ = Terminal::move_cursor(Position {
            row: self.position_y as u16,
            ..Default::default()
        });
        let result = Terminal::print(&status);
        self.needs_redraw = false;
        debug_assert!(result.is_ok(), "Failed to render status bar");
    }
}
