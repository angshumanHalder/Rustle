use std::io::{Error, Write, stdout};

use crossterm::event::{
    Event::{self, Key},
    KeyCode::Char,
    KeyEvent, KeyModifiers, read,
};

mod terminal;
use terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub const fn default() -> Self {
        Self { should_quit: false }
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
            Self::draw_welcome_message()?;
            let event = read()?;
            self.evaluate_event(&event);
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => (),
            }
        }
    }

    fn draw_rows() -> Result<(), Error> {
        let Size { height, .. } = Terminal::size()?;
        for r in 0..height {
            Terminal::clear_line()?;
            Self::draw_empty_row()?;
            if r + 1 < height {
                Terminal::print("\r\n".to_string())?;
            }
        }
        Terminal::move_cursor(&Position { col: 0, row: 0 })?;
        stdout().flush()?;
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n".to_string())?;
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor(&Position { col: 0, row: 0 })?;
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
