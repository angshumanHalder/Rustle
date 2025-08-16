use std::io::Error;
use std::{cmp::min, fs::File};

use crossterm::event::{
    Event::{self, Key},
    KeyCode::{self},
    KeyEvent, KeyEventKind, KeyModifiers, read,
};

mod buffer;
mod terminal;
mod view;

use terminal::{Position, Size, Terminal};
use view::View;

pub struct Editor {
    should_quit: bool,
    location: Location,
    view: View,
}

struct Location {
    col: u16,
    row: u16,
}

impl Editor {
    pub fn new(path: Option<&String>) -> Self {
        let mut file: Option<File> = None;
        if let Some(p) = path {
            file = (File::open(p)).ok();
        }
        Self {
            should_quit: false,
            location: Location { col: 0, row: 0 },
            view: View::new(file),
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
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
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

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        Terminal::move_cursor(&Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            Terminal::move_cursor(&Position {
                col: self.location.col,
                row: self.location.row,
            })?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }
}
