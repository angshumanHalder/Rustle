use std::fs::File;
use std::io::Error;
use std::{char, cmp};

use buffer::Buffer;
use crossterm::style::Color;
use grapheme::{HighlightType, Line};
pub use languageconfig::LanguageConfig;

use super::commands::{Direction, EditorCommand};
use super::terminal::{Position, Size, Terminal};
use super::uicomponent::UIComponent;

mod buffer;
mod grapheme;
mod languageconfig;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    margin_bottom: usize,
    size: Size,
    location: Location,
    scroll_offset: Position,
    needs_redraw: bool,
    search_locations: Vec<Location>,
    current_search_idx: Option<usize>,
    search_query: String,
    language_config: LanguageConfig,
}

impl View {
    pub fn new(file: Option<File>, margin_bottom: u16, config: LanguageConfig) -> Self {
        let terminal_size = Terminal::size().unwrap_or_default();
        Self {
            buffer: Buffer::new(file),
            size: Size {
                width: terminal_size.width,
                height: terminal_size.height.saturating_sub(margin_bottom),
            },
            margin_bottom: margin_bottom as usize,
            location: Location::default(),
            scroll_offset: Position { col: 0, row: 0 },
            needs_redraw: true,
            search_locations: Vec::new(),
            current_search_idx: None,
            search_query: String::new(),
            language_config: config,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn resize(&mut self, size: Size) {
        self.size = Size {
            width: size.width,
            height: size.height.saturating_sub(self.margin_bottom as u16),
        };
        self.scroll_into_view();
        self.mark_redraw(true);
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => {
                self.resize(size);
            }
            EditorCommand::Move(direction) => {
                self.move_cursor(direction);
            }
            EditorCommand::Insert(c) => self.add_character(c),
            EditorCommand::Backspace => {
                self.remove_backward();
            }
            EditorCommand::Enter => {
                self.add_character('\n');
            }
            EditorCommand::Delete => {
                self.remove_forward();
            }
            EditorCommand::Esc
            | EditorCommand::Quit
            | EditorCommand::Ignore
            | EditorCommand::Save
            | EditorCommand::Search => {}
        }
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

    pub fn search_document(&mut self, query: &str) {
        if query.is_empty() {
            self.clear_search();
            return;
        }
        let raw_results = self.buffer.search_document(query);
        let mut locations = Vec::new();
        for (line_index, char_idx_line) in raw_results {
            if let Some(line_slice) = self.buffer.get_line(line_index) {
                let grapheme_index =
                    Line::get_fragments_from_rope_slice(line_slice, &self.language_config)
                        .char_to_grapheme_idx(char_idx_line);
                locations.push(Location {
                    grapheme_index,
                    line_index,
                });
            }
        }
        self.search_locations = locations;
        self.search_query = String::from(query);
        self.mark_redraw(true);

        if self.search_locations.is_empty() {
            self.current_search_idx = None;
            self.search_query = String::new();
            return;
        }

        let start_position = self.location;
        let new_idx = self.search_locations.iter().position(|loc| {
            loc.line_index > start_position.line_index
                || (loc.line_index == start_position.line_index
                    && loc.grapheme_index >= start_position.grapheme_index)
        });

        let final_idx = new_idx.unwrap_or(0);
        self.current_search_idx = Some(final_idx);
        self.jump_to_match(final_idx);
        self.mark_redraw(true);
    }

    pub fn clear_search(&mut self) {
        self.search_locations = vec![];
        self.current_search_idx = None;
        self.search_query = String::new();
        self.mark_redraw(true);
    }

    pub fn jump_to_match(&mut self, idx: usize) {
        if let Some(location) = self.search_locations.get(idx) {
            self.location = *location;
            self.scroll_into_view();
            self.mark_redraw(true);
        }
    }

    pub fn find_next(&mut self) {
        if self.search_locations.is_empty() {
            return;
        }
        let new_idx = self
            .current_search_idx
            .map_or(0, |idx| (idx + 1) % self.search_locations.len());
        self.current_search_idx = Some(new_idx);
        self.jump_to_match(new_idx);
    }

    pub fn find_previous(&mut self) {
        if self.search_locations.is_empty() {
            return;
        }
        let new_idx = self.current_search_idx.map_or(0, |idx| {
            if idx == 0 {
                self.search_locations.len() - 1
            } else {
                idx - 1
            }
        });
        self.current_search_idx = Some(new_idx);
        self.jump_to_match(new_idx);
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
            .map_or(0, |line| {
                Line::get_fragments_from_rope_slice(line, &self.language_config).grapheme_count()
            });
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
        self.location.grapheme_index =
            self.buffer
                .get_line(self.location.line_index)
                .map_or(0, |line| {
                    Line::get_fragments_from_rope_slice(line, &self.language_config)
                        .grapheme_count()
                });
    }

    fn add_character(&mut self, ch: char) {
        if let Some(line) = self.buffer.get_line(self.location.line_index) {
            let char_idx_in_line = Line::get_fragments_from_rope_slice(line, &self.language_config)
                .grapheme_to_char_idx(self.location.grapheme_index);
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
        self.mark_redraw(true);
    }

    fn remove_backward(&mut self) {
        if self.location.line_index == 0 && self.location.grapheme_index == 0 {
            return;
        }

        self.move_left();

        if let Some(line) = self.buffer.get_line(self.location.line_index) {
            let view_line = Line::get_fragments_from_rope_slice(line, &self.language_config);
            let char_idx_in_line = view_line.grapheme_to_char_idx(self.location.grapheme_index);
            let grapheme_char_len = view_line
                .grapheme_char_len(self.location.grapheme_index)
                .unwrap_or(0);
            let pos =
                self.buffer.document.line_to_char(self.location.line_index) + char_idx_in_line;
            self.buffer.delete_range_char(pos, grapheme_char_len);
            self.scroll_into_view();
            self.mark_redraw(true);
        }
    }

    fn remove_forward(&mut self) {
        if let Some(line) = self.buffer.get_line(self.location.line_index) {
            let view_line = Line::get_fragments_from_rope_slice(line, &self.language_config);
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
                    self.mark_redraw(true);
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
            self.mark_redraw(true);
        }
    }

    // doesn't trigger scroll
    fn snap_to_valid_grapheme(&mut self) {
        self.location.grapheme_index =
            self.buffer
                .get_line(self.location.grapheme_index)
                .map_or(0, |line| {
                    cmp::min(
                        Line::get_fragments_from_rope_slice(line, &self.language_config)
                            .grapheme_count(),
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
            Line::get_fragments_from_rope_slice(line, &self.language_config)
                .width_until(self.location.grapheme_index) as u16
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
        self.mark_redraw(offset_changed);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn get_position(&self) -> Position {
        let row = self.location.line_index;
        let col = self.buffer.get_line(row).map_or(0, |line| {
            Line::get_fragments_from_rope_slice(line, &self.language_config)
                .width_until(self.location.grapheme_index)
        });
        let pos = Position {
            col: col as u16,
            row: row as u16,
        };
        pos.saturating_sub(self.scroll_offset)
    }

    fn render_line(&self, r: u16) -> Result<(), Error> {
        Terminal::move_cursor(Position { col: 0, row: r })?;
        Terminal::clear_line()?;
        let top = self.scroll_offset.row;
        let line_idx = r.saturating_add(top) as usize;

        let Some(line_slice) = self.buffer.get_line(line_idx) else {
            if self.buffer.is_empty() && r == self.size.height / 3 {
                Self::draw_welcome_message()?;
            } else {
                Self::draw_empty_row()?;
            }
            return Ok(());
        };

        let line = Line::get_fragments_from_rope_slice(line_slice, &self.language_config);
        let visible_start_width = self.scroll_offset.col as usize;
        let visible_end_width = visible_start_width.saturating_add(self.size.width as usize);

        let matches_on_line: Vec<_> = self
            .search_locations
            .iter()
            .enumerate()
            .filter(|(_, loc)| loc.line_index == line_idx)
            .collect();

        if matches_on_line.is_empty() {
            Self::render_segment(
                &line,
                0,
                line.grapheme_count(),
                visible_start_width,
                visible_end_width,
                None,
            )?;
        } else {
            let query_grapheme_len = Line::get_fragments_from_string(
                self.search_query.as_str(),
                &LanguageConfig::text(),
            )
            .grapheme_count();
            let mut current_grapheme_idx = 0;

            for (match_list_idx, location) in matches_on_line {
                let match_start = location.grapheme_index;
                let match_end = match_start.saturating_add(query_grapheme_len);

                Self::render_segment(
                    &line,
                    current_grapheme_idx,
                    match_start,
                    visible_start_width,
                    visible_end_width,
                    None,
                )?;

                // Render the match itself with a background color
                let background_color = if self.current_search_idx == Some(match_list_idx) {
                    Color::Yellow
                } else {
                    Color::DarkGrey
                };
                Self::render_segment(
                    &line,
                    match_start,
                    match_end,
                    visible_start_width,
                    visible_end_width,
                    Some(background_color),
                )?;

                current_grapheme_idx = match_end;
            }

            // Render any text remaining AFTER the last match
            Self::render_segment(
                &line,
                current_grapheme_idx,
                line.grapheme_count(),
                visible_start_width,
                visible_end_width,
                None,
            )?;
        }

        Ok(())
    }

    fn render_segment(
        line: &Line,
        from_grapheme: usize,
        to_grapheme: usize,
        visible_start_width: usize,
        visible_end_width: usize,
        background_color: Option<Color>,
    ) -> Result<(), Error> {
        if from_grapheme >= to_grapheme {
            return Ok(());
        }

        for grapheme_idx in from_grapheme..to_grapheme {
            if let Some(fragment) = line.fragments.get(grapheme_idx) {
                let fragment_start_width = line.width_until(grapheme_idx);
                let fragment_end_width =
                    fragment_start_width + fragment.rendered_width.saturating_add(0);

                if fragment_end_width > visible_start_width
                    && fragment_start_width < visible_end_width
                {
                    if let Some(color) = background_color {
                        Terminal::set_bg_color(color)?;
                    }

                    let fg_color = match fragment.highlight_type {
                        HighlightType::Keyword => Color::Blue,
                        HighlightType::Type => Color::Cyan,
                        HighlightType::Number => Color::Yellow,
                        HighlightType::String => Color::Green,
                        HighlightType::Comment => Color::DarkGrey,
                        HighlightType::Normal => Color::Reset,
                    };
                    Terminal::set_fg_color(fg_color)?;

                    Terminal::print(&fragment.grapheme)?;

                    Terminal::reset_color()?;
                }
            }
        }
        Ok(())
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

impl UIComponent for View {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_into_view();
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let size = Terminal::size().unwrap();
        let end_y = (origin_y as u16)
            .saturating_add(size.height)
            .saturating_sub(self.margin_bottom as u16);
        for r in (origin_y as u16)..end_y {
            self.render_line(r).unwrap();
        }
        if self.buffer.is_empty() {
            Self::draw_welcome_message().unwrap();
        }
        Terminal::move_cursor(Position { col: 0, row: 0 }).unwrap();
        // Terminal::execute().unwrap();
        Ok(())
    }
}
