use std::fs::File;
use std::io::Error;

use super::buffer::Buffer;
use super::terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    need_redraw: bool,
    size: Size,
}

impl View {
    pub fn new(file: Option<File>) -> Self {
        Self {
            buffer: Buffer::new(file),
            need_redraw: true,
            size: Terminal::size().unwrap_or_default(),
        }
    }

    pub fn render(&mut self) {
        if !self.need_redraw {
            return;
        }
        let Size { height, width } = Terminal::size().unwrap();
        if height == 0 || width == 0 {
            return;
        }
        for r in 0..height {
            self.render_line(r, width).unwrap();
        }
        if self.buffer.is_empty() {
            Self::draw_welcome_message().unwrap();
        }
        Terminal::move_cursor(&Position { col: 0, row: 0 }).unwrap();
        Terminal::execute().unwrap();
        self.need_redraw = false;
    }

    pub fn resize(&mut self, size: Size) {
        self.size = size;
        self.need_redraw = true;
    }

    fn render_line(&self, r: u16, width: u16) -> Result<(), Error> {
        Terminal::move_cursor(&Position { col: 0, row: r })?;
        Terminal::clear_line()?;
        if let Some(text) = self.buffer.get_line(r as usize, width as usize) {
            Terminal::print(&text)
        } else {
            Self::draw_empty_row()
        }
    }

    fn draw_welcome_message() -> Result<(), Error> {
        let Size { width, height } = Terminal::size()?;
        let mut message = format!("{NAME} editor - version {VERSION}");
        message.truncate(width as usize);
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
