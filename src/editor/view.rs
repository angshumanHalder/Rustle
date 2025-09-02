use std::fs::File;
use std::io::Error;
use std::{char, cmp};

use buffer::Buffer;
use grapheme::Line;

use super::commands::{Direction, EditorCommand};
use super::terminal::{Position, Size, Terminal};

mod buffer;
mod grapheme;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Clone, Copy)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ViewStatus {
    pub total_lines: usize,
    pub current_line_idx: usize,
    pub is_modified: bool,
}

pub struct View {
    pub buffer: Buffer,
    pub need_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Position,
}

impl View {
    pub fn new(file: Option<File>, margin_bottom: u16) -> Self {
        let terminal_size = Terminal::size().unwrap_or_default();
        Self {
            buffer: Buffer::new(file),
            need_redraw: true,
            size: Size {
                width: terminal_size.width,
                height: terminal_size.height.saturating_sub(margin_bottom),
            },
            location: Location::default(),
            scroll_offset: Position { col: 0, row: 0 },
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.size = size;
        self.scroll_into_view();
        self.need_redraw = true;
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Move(direction) => self.move_cursor(direction),
            EditorCommand::Insert(c) => self.add_character(c),
            EditorCommand::Backspace => self.remove_backward(),
            EditorCommand::Enter => self.add_character('\n'),
            EditorCommand::Delete => self.remove_forward(),
            EditorCommand::Quit | EditorCommand::Ignore | EditorCommand::Save => {}
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
        Terminal::move_cursor(Position { col: 0, row: 0 }).unwrap();
        Terminal::execute().unwrap();
        self.need_redraw = false;
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        let Size { height, .. } = Terminal::size().unwrap();
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_add(1)),
            Direction::Home => self.move_to_line_start(),
            Direction::End => self.move_to_line_end(),
        }
        self.scroll_into_view();
    }

    pub fn get_status(&self) -> ViewStatus {
        ViewStatus {
            total_lines: self.buffer.line_count(),
            current_line_idx: self.location.line_index,
            is_modified: self.buffer.is_dirty,
        }
    }

    fn move_up(&mut self, step: u16) {
        self.location.line_index = self.location.line_index.saturating_sub(step as usize);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: u16) {
        self.location.line_index = self.location.line_index.saturating_add(step as usize);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    fn move_left(&mut self) {
        if self.location.grapheme_index > 0 {
            self.location.grapheme_index -= 1;
        } else if self.location.line_index > 0 {
            self.move_up(1);
            self.move_to_line_end();
        }
    }

    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .get_line(self.location.line_index)
            .map_or(0, |line| Line::from(line).grapheme_count());
        if self.location.grapheme_index < line_width {
            self.location.grapheme_index += 1;
        } else {
            self.move_down(1);
            self.move_to_line_start();
        }
    }

    fn move_to_line_start(&mut self) {
        self.location.grapheme_index = 0;
    }

    fn move_to_line_end(&mut self) {
        self.location.grapheme_index = self
            .buffer
            .get_line(self.location.line_index)
            .map_or(0, |line| Line::from(line).grapheme_count());
    }

    fn add_character(&mut self, ch: char) {
        if let Some(line) = self.buffer.get_line(self.location.line_index) {
            let char_idx_in_line =
                Line::from(line).grapheme_to_char_idx(self.location.grapheme_index);
            let pos =
                self.buffer.document.line_to_char(self.location.line_index) + char_idx_in_line;
            self.buffer.insert_char(pos, ch);
        } else {
            let pos = self.buffer.document.len_chars();
            self.buffer.insert_char(pos, ch);
        }
        if ch == '\n' {
            self.location.line_index += 1;
            self.location.grapheme_index = 0;
        } else {
            self.location.grapheme_index += 1;
        }
        self.scroll_into_view();
        self.need_redraw = true;
    }

    fn remove_backward(&mut self) {
        if self.location.line_index == 0 && self.location.grapheme_index == 0 {
            return;
        }

        self.move_left();

        if let Some(line) = self.buffer.get_line(self.location.line_index) {
            let view_line = Line::from(line);
            let char_idx_in_line = view_line.grapheme_to_char_idx(self.location.grapheme_index);
            let grapheme_char_len = view_line
                .grapheme_char_len(self.location.grapheme_index)
                .unwrap_or(0);
            let pos =
                self.buffer.document.line_to_char(self.location.line_index) + char_idx_in_line;
            self.buffer.delete_range_char(pos, grapheme_char_len);
            self.scroll_into_view();
            self.need_redraw = true;
        }
    }

    fn remove_forward(&mut self) {
        if let Some(line) = self.buffer.get_line(self.location.line_index) {
            let view_line = Line::from(line);
            let line_len = view_line.grapheme_count();
            if self.location.grapheme_index >= line_len {
                if self.location.line_index < self.buffer.line_count() - 1 {
                    let pos = self
                        .buffer
                        .document
                        .line_to_char(self.location.line_index + 1)
                        .saturating_sub(1);
                    self.buffer.delete_range_char(pos, 1);
                    self.scroll_into_view();
                    self.need_redraw = true;
                }
                return;
            }
            let char_idx_in_line = view_line.grapheme_to_char_idx(self.location.grapheme_index);
            let grapheme_char_len = view_line
                .grapheme_char_len(self.location.grapheme_index)
                .unwrap_or(0);
            let pos =
                self.buffer.document.line_to_char(self.location.line_index) + char_idx_in_line;
            self.buffer.delete_range_char(pos, grapheme_char_len);
            self.scroll_into_view();
            self.need_redraw = true;
        }
    }

    // doesn't trigger scroll
    fn snap_to_valid_grapheme(&mut self) {
        self.location.grapheme_index =
            self.buffer
                .get_line(self.location.grapheme_index)
                .map_or(0, |line| {
                    cmp::min(
                        Line::from(line).grapheme_count(),
                        self.location.grapheme_index,
                    )
                });
    }

    fn snap_to_valid_line(&mut self) {
        self.location.line_index = cmp::min(
            self.buffer.line_count().saturating_sub(1),
            self.location.line_index,
        );
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn scroll_into_view(&mut self) {
        let row = self.location.line_index as u16;
        let col = self.buffer.get_line(row as usize).map_or(0, |line| {
            Line::from(line).width_until(self.location.grapheme_index) as u16
        });
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

    #[allow(clippy::cast_possible_truncation)]
    pub fn get_position(&self) -> Position {
        let row = self.location.line_index;
        let col = self.buffer.get_line(row).map_or(0, |line| {
            Line::from(line).width_until(self.location.grapheme_index)
        });
        let pos = Position {
            col: col as u16,
            row: row as u16,
        };
        pos.saturating_sub(self.scroll_offset)
    }

    fn render_line(&self, r: u16, width: u16) -> Result<(), Error> {
        Terminal::move_cursor(Position { col: 0, row: r })?;
        Terminal::clear_line()?;
        let top = self.scroll_offset.row;
        let left = self.scroll_offset.col;
        let line_opt = self.buffer.get_line(r.saturating_add(top) as usize);

        if let Some(line) = line_opt {
            // Only grab the visible portion
            let text_to_print: String = Line::from(line).get_visible_graphemes(
                (left as usize)..self.scroll_offset.col.saturating_add(width) as usize,
            );
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

        Terminal::move_cursor(Position {
            col: offset_width_pos,
            row: offset_height_pos,
        })?;
        Terminal::print(&message)?;
        Terminal::move_cursor(Position { col: 0, row: 0 })?;
        Terminal::execute()?;

        Ok(())
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")
    }
}
