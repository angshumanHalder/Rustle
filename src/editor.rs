use std::fs::File;
use std::io::Error;
use std::panic::{set_hook, take_hook};

use commands::EditorCommand;
use crossterm::event::{Event, KeyEvent, KeyEventKind, read};

mod commands;
mod statusbar;
mod terminal;
mod view;

use statusbar::StatusBar;
use terminal::Terminal;
use view::{View, ViewStatus};

#[derive(Default, PartialEq, Eq, Debug)]
pub struct DocumentStatus {
    total_lines: usize,
    current_line_idx: usize,
    is_modified: bool,
    path: Option<String>,
}

impl From<ViewStatus> for DocumentStatus {
    fn from(view_status: ViewStatus) -> Self {
        Self {
            total_lines: view_status.total_lines,
            current_line_idx: view_status.current_line_idx,
            is_modified: view_status.is_modified,
            path: None,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    view: View,
    file_path: Option<String>,
    status_bar: StatusBar,
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

        // update the status on load
        let view = View::new(file, 2);
        let view_status = view.get_status();
        let mut status: DocumentStatus = view_status.into();
        status.path = path.cloned();
        let mut status_bar = StatusBar::new(1);
        status_bar.update_status(status);

        Ok(Self {
            should_quit: false,
            view,
            file_path: path.cloned(),
            status_bar,
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
            let view_status = self.view.get_status();
            let mut status: DocumentStatus = view_status.into();
            status.path.clone_from(&self.file_path);
            self.status_bar.update_status(status);
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
                        if let EditorCommand::Resize(size) = command {
                            self.status_bar.resize(size);
                        }
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
        if self.view.need_redraw {
            self.status_bar.redraw();
        }
        self.view.render();
        self.status_bar.render();
        let _ = Terminal::move_cursor(self.view.get_position());
        Terminal::show_cursor().unwrap();
        Terminal::execute().unwrap();
        Ok(())
    }

    fn save_file(&mut self) {
        if let Some(file_path) = &self.file_path {
            let _ = self.view.buffer.write_to_file(file_path.clone());
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
