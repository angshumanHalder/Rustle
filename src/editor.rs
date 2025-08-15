use std::cmp::min;
use std::io::Error;

use crossterm::event::{
    Event::{self, Key},
    KeyCode::{self},
    KeyEvent, KeyModifiers, read,
};

mod terminal;
use terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    location: Location,
}

struct Location {
    col: u16,
    row: u16,
}

impl Editor {
    pub const fn default() -> Self {
        Self {
            should_quit: false,
            location: Location { col: 0, row: 0 },
        }
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    pub fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event)?;
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End => {
                    self.move_point(*code)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn move_point(&mut self, key_code: KeyCode) -> Result<(), Error> {
        let Location { mut col, mut row } = self.location;
        let Size { width, height } = Terminal::size()?;
        match key_code {
            KeyCode::Up => {
                row = row.saturating_sub(1);
            }
            KeyCode::Down => {
                row = min(height.saturating_sub(1), row.saturating_add(1));
            }
            KeyCode::Left => {
                col = col.saturating_sub(1);
            }
            KeyCode::Right => {
                col = min(width.saturating_sub(1), col.saturating_add(1));
            }
            KeyCode::PageUp => {
                row = 0;
            }
            KeyCode::PageDown => {
                row = height.saturating_sub(1);
            }
            KeyCode::Home => {
                col = 0;
            }
            KeyCode::End => {
                col = width.saturating_sub(1);
            }
            _ => (),
        }
        self.location = Location { col, row };
        Ok(())
    }

    fn draw_rows() -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;
        for r in 0..height {
            Terminal::clear_line()?;
            Self::draw_empty_row()?;
            if r.saturating_add(1) < height {
                Terminal::print("\r\n".to_string())?;
            }
        }
        Self::draw_welcome_message()?;
        Terminal::move_cursor(&Position { col: 0, row: 0 })?;
        Terminal::execute()?;
        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        Terminal::move_cursor(&Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n".to_string())?;
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor(&Position {
                col: self.location.col,
                row: self.location.row,
            })?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn draw_welcome_message() -> Result<(), Error> {
        let Size { width, height } = Terminal::size()?;
        let message = format!("{NAME} editor - version {VERSION}");
        let offset_height_pos = height / 3;
        let offset_width_pos = (width / 2).saturating_sub((message.len() / 2) as u16);

        Terminal::move_cursor(&Position {
            col: offset_width_pos,
            row: offset_height_pos,
        })?;
        Terminal::print(message)?;
        Terminal::move_cursor(&Position { col: 0, row: 0 })?;
        Terminal::execute()?;

        Ok(())
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~".to_string())
    }
}
