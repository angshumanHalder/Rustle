use std::io::Error;
use std::panic::{set_hook, take_hook};
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
    pub fn new(path: Option<&String>) -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;
        let mut file: Option<File> = None;
        if let Some(p) = path {
            file = (File::open(p)).ok();
        }
        Ok(Self {
            should_quit: false,
            location: Location { col: 0, row: 0 },
            view: View::new(file),
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen().unwrap();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => self.evaluate_event(&event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}")
                    }
                }
            }
        }
    }

    fn evaluate_event(&mut self, event: &Event) {
        match event {
            Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, *modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                (
                    KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Home
                    | KeyCode::End,
                    _,
                ) => {
                    self.move_point(*code);
                }
                _ => (),
            },
            Event::Resize(width, height) => self.view.resize(Size {
                width: *width,
                height: *height,
            }),
            _ => {}
        }
    }

    fn move_point(&mut self, key_code: KeyCode) {
        let Location { mut col, mut row } = self.location;
        let Size { width, height } = Terminal::size().unwrap();
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
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        Terminal::move_cursor(&Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render();
            Terminal::move_cursor(&Position {
                col: self.location.col,
                row: self.location.row,
            })?;
        }
        Terminal::show_cursor().unwrap();
        Terminal::execute().unwrap();
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye.\r\n");
        }
    }
}
