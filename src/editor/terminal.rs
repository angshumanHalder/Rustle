use std::io::{Error, Write, stdout};

use crossterm::{
    Command, cursor, execute, queue,
    style::Print,
    terminal::{self, Clear, disable_raw_mode, enable_raw_mode},
};

#[derive(Default)]
pub struct Terminal {}

#[derive(Default)]
pub struct Position {
    pub col: u16,
    pub row: u16,
}

pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor(&Position { col: 0, row: 0 })?;
        Self::execute()
    }

    pub fn terminate() -> Result<(), Error> {
        Self::execute()?;
        disable_raw_mode()
    }

    pub fn move_cursor(pos: &Position) -> Result<(), std::io::Error> {
        Self::queue_command(cursor::MoveTo(pos.col, pos.row))
    }

    pub fn size() -> Result<Size, Error> {
        let (width, height) = terminal::size()?;
        Ok(Size { width, height })
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(crossterm::terminal::ClearType::All))
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(crossterm::terminal::ClearType::CurrentLine))
    }

    pub fn hide_cursor() -> Result<(), std::io::Error> {
        execute!(stdout(), cursor::Hide)
    }

    pub fn show_cursor() -> Result<(), std::io::Error> {
        execute!(stdout(), cursor::Show)
    }

    pub fn print(s: &str) -> Result<(), std::io::Error> {
        Self::queue_command(Print(s))
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()
    }

    fn queue_command<T: Command>(cmd: T) -> Result<(), Error> {
        queue!(stdout(), cmd)
    }
}
