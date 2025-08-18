use std::fs::File;
use std::io::Error;

use buffer::Buffer;
use location::Location;

use super::commands::{Direction, EditorCommand};
use super::terminal::{Position, Size, Terminal};

mod buffer;
mod location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    need_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Location,
}

impl View {
    pub fn new(file: Option<File>) -> Self {
        Self {
            buffer: Buffer::new(file),
            need_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            location: Location { col: 0, row: 0 },
            scroll_offset: Location { col: 0, row: 0 },
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
        self.scroll_into_view();
        self.need_redraw = true;
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Move(direction) => self.move_cursor(&direction),
            EditorCommand::Quit => {}
        }
    }

    pub fn move_cursor(&mut self, direction: &Direction) {
        let Location { mut col, mut row } = self.location;
        let Size { width, height } = Terminal::size().unwrap();
        match direction {
            Direction::Up => {
                row = row.saturating_sub(1);
            }
            Direction::Down => {
                row = row.saturating_add(1);
            }
            Direction::Left => {
                col = col.saturating_sub(1);
            }
            Direction::Right => {
                col = col.saturating_add(1);
            }
            Direction::PageUp => {
                row = 0;
            }
            Direction::PageDown => {
                row = height.saturating_sub(1);
            }
            Direction::Home => {
                col = 0;
            }
            Direction::End => {
                col = width.saturating_sub(1);
            }
        }
        self.location = Location { col, row };
        self.scroll_into_view();
    }

    pub fn scroll_into_view(&mut self) {
        let Location { col, row } = self.location;
        let Size { width, height } = self.size;
        let mut offset_changed = false;

        // vertical
        if row < self.scroll_offset.row {
            self.scroll_offset.row = row;
            offset_changed = true;
        } else if row >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = row.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        if col < self.scroll_offset.col {
            self.scroll_offset.col = col;
            offset_changed = true;
        } else if col >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = col.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }
        self.need_redraw = offset_changed;
    }

    pub fn get_position(&self) -> Position {
        self.location.subtract(self.scroll_offset).into()
    }

    fn render_line(&self, r: u16, width: u16) -> Result<(), Error> {
        Terminal::move_cursor(&Position { col: 0, row: r })?;
        Terminal::clear_line()?;
        let top = self.scroll_offset.row;
        let left = self.scroll_offset.col as usize;
        let line_opt = self
            .buffer
            .get_line(r.saturating_add(top) as usize, left + width as usize);

        if let Some(line) = line_opt {
            // Only grab the visible portion
            let text_to_print: String = line.chars().skip(left).take(width as usize).collect();
            if text_to_print.is_empty() {
                Self::draw_empty_row()
            } else {
                Terminal::print(&text_to_print)
            }
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
