use std::fs::File;
use std::io::Error;

use super::buffer::Buffer;
use super::terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
}

impl View {
    pub fn new(file: Option<File>) -> Self {
        Self {
            buffer: Buffer::new(file),
        }
    }

    pub fn render(&self) -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;
        for r in 0..height {
            Terminal::move_cursor(&Position { col: 0, row: r })?;
            Terminal::clear_line()?;
            // FIX: refactor with more efficient logic to render document rather than querying for
            // each line
            // edge case empty document first line doesn't render empty row. Fix it when
            // refactoring
            if let Some(text) = self.buffer.get_line(r as usize) {
                Terminal::print(&text)?;
            } else {
                Self::draw_empty_row()?;
            }
            if r.saturating_add(1) < height && self.buffer.document.len_chars() > 0 {
                Terminal::print("\r\n")?;
            }
        }
        if self.buffer.document.len_chars() == 0 {
            Self::draw_welcome_message()?;
        }
        Terminal::move_cursor(&Position { col: 0, row: 0 })?;
        Terminal::execute()?;
        Ok(())
    }

    fn draw_welcome_message() -> Result<(), Error> {
        let Size { width, height } = Terminal::size()?;
        let message = format!("{NAME} editor - version {VERSION}");
        let offset_height_pos = height / 3;

        #[allow(clippy::cast_possible_truncation)]
        let offset_width_pos = (width / 2).saturating_sub((message.len() / 2) as u16);

        Terminal::move_cursor(&Position {
            col: offset_width_pos,
            row: offset_height_pos,
        })?;
        Terminal::print(&message)?;
        Terminal::move_cursor(&Position { col: 0, row: 0 })?;
        Terminal::execute()?;

        Ok(())
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")
    }
}
