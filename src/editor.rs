use std::fs::File;
use std::io::Error;
use std::panic::{set_hook, take_hook};

use commands::EditorCommand;
use crossterm::event::{Event, KeyEvent, KeyEventKind, read};

mod commands;
mod terminal;
mod view;

use terminal::Terminal;
use view::View;

pub struct Editor {
    should_quit: bool,
    view: View,
    file_path: Option<String>,
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
            view: View::new(file),
            file_path: path.cloned(),
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
        let should_process = match event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            match EditorCommand::try_from(event) {
                Ok(command) => {
                    if matches!(command, EditorCommand::Quit) {
                        self.should_quit = true;
                    }
                    if matches!(command, EditorCommand::Save) {
                        self.save_file();
                    } else {
                        self.view.handle_command(command);
                    }
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not handle command: {err}")
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Received and discarded unsupported or non-press event")
            }
        }
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render();
            let _ = Terminal::move_cursor(self.view.get_position());
        }
        Terminal::show_cursor().unwrap();
        Terminal::execute().unwrap();
        Ok(())
    }

    fn save_file(&self) {
        if let Some(file_path) = self.file_path.clone() {
            self.view.buffer.write_to_file(file_path);
        }
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
