use std::io::{Error, Write, stdout};

use crossterm::{
    Command,
    cursor::{self},
    execute, queue,
    style::{Attribute, Print},
    terminal::{
        self, Clear, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};

#[derive(Default)]
pub struct Terminal {}

#[derive(Default, Clone, Copy)]
pub struct Position {
    pub col: u16,
    pub row: u16,
}

impl Position {
    pub const fn saturating_sub(self, other: Position) -> Self {
        Self {
            col: self.col.saturating_sub(other.col),
            row: self.row.saturating_sub(other.row),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::clear_screen()?;
        Self::disable_line_wrap()?;
        Self::move_cursor(Position { col: 0, row: 0 })?;
        Self::execute()
    }

    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::show_cursor()?;
        Self::execute()?;
        Self::enable_line_wrap()?;
        disable_raw_mode()
    }

    pub fn move_cursor(pos: Position) -> Result<(), std::io::Error> {
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

    pub fn print_inverted_row(s: &str) -> Result<(), std::io::Error> {
        let width = Self::size()?.width as usize;
        Self::print(&format!(
            "{}{:width$.width$}{}",
            Attribute::Reverse,
            s,
            Attribute::Reset
        ))?;
        Ok(())
    }

    pub fn set_fg_color(color: crossterm::style::Color) -> Result<(), std::io::Error> {
        Self::queue_command(crossterm::style::SetForegroundColor(color))
    }

    pub fn set_bg_color(color: crossterm::style::Color) -> Result<(), std::io::Error> {
        Self::queue_command(crossterm::style::SetBackgroundColor(color))
    }

    pub fn reset_color() -> Result<(), std::io::Error> {
        Self::queue_command(crossterm::style::ResetColor)
    }

    pub fn enter_alternate_screen() -> Result<(), std::io::Error> {
        Self::queue_command(EnterAlternateScreen)
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()
    }

    fn disable_line_wrap() -> Result<(), Error> {
        Self::queue_command(DisableLineWrap)?;
        Ok(())
    }

    fn enable_line_wrap() -> Result<(), Error> {
        Self::queue_command(EnableLineWrap)?;
        Ok(())
    }

    fn leave_alternate_screen() -> Result<(), std::io::Error> {
        Self::queue_command(LeaveAlternateScreen)
    }

    fn queue_command<T: Command>(cmd: T) -> Result<(), Error> {
        queue!(stdout(), cmd)
    }
}
