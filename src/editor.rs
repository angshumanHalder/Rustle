use std::fs::File;
use std::io::Error;
use std::panic::{set_hook, take_hook};
use std::time::Duration;

use commands::EditorCommand;
use crossterm::event::{Event, KeyEvent, KeyEventKind, poll, read};

mod commands;
mod messagebar;
mod searchbar;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use messagebar::MessageBar;
use searchbar::SearchBar;
use statusbar::StatusBar;
use terminal::{Size, Terminal};
use uicomponent::UIComponent;
use view::{View, ViewStatus};

const COUNT_TO_QUIT: i16 = 1;

#[derive(Clone, Copy)]
enum Mode {
    Search,
    Edit,
}

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
    message_bar: MessageBar,
    search_bar: SearchBar,
    quit_press_count: i16,
    mode: Mode,
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

        let mut status_bar = StatusBar::default();
        let size = Terminal::size()?;
        status_bar.set_size(size);
        status_bar.update_status(status);

        let mut message_bar = MessageBar::default();
        message_bar
            .update_message("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = search".to_string());

        let mut search_bar = SearchBar::default();
        search_bar.update_search_query(String::new());

        Ok(Self {
            should_quit: false,
            view,
            file_path: path.cloned(),
            status_bar,
            message_bar,
            search_bar,
            quit_press_count: 0,
            mode: Mode::Edit,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match poll(Duration::from_millis(100)) {
                Ok(true) => match read() {
                    Ok(event) => self.evaluate_event(&event),
                    Err(err) => {
                        #[cfg(debug_assertions)]
                        {
                            panic!("Could not read event: {err:?}")
                        }
                    }
                },
                Ok(false) => {}
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

        assert!(
            should_process,
            "Received and discarded unsupported or non-press event"
        );

        if should_process {
            // Global commands
            match EditorCommand::try_from(event) {
                Ok(command) => {
                    match command {
                        EditorCommand::Quit => {
                            if self.view.buffer.is_dirty && self.quit_press_count < COUNT_TO_QUIT {
                                self.quit_press_count += 1;
                                self.message_bar.update_message(String::from(
                                    "WARNING: File has unsaved changes. Press again to Quit",
                                ));
                                self.search_bar.clear();
                                self.mode = Mode::Edit;
                            } else {
                                self.should_quit = true;
                            }
                        }
                        EditorCommand::Save => match self.save_file() {
                            Ok(v) => self.message_bar.update_message(v),
                            Err(e) => self.message_bar.update_message(format!("Err: {e}")),
                        },
                        EditorCommand::Search => {
                            self.mode = Mode::Search;
                        }
                        EditorCommand::Resize(size) => {
                            self.resize(size);
                        }
                        _ => {}
                    }
                    match self.mode {
                        Mode::Edit => self.view.handle_command(command),
                        Mode::Search => match command {
                            EditorCommand::Insert(c) => {
                                self.search_bar.push_char(c);
                            }
                            EditorCommand::Backspace => {
                                self.search_bar.pop_char();
                            }
                            EditorCommand::Enter => {
                                // trigger search
                                todo!()
                            }
                            EditorCommand::Esc => {
                                self.mode = Mode::Edit;
                            }
                            _ => {}
                        },
                    }
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not handle command: {err}")
                    }
                }
            }
        }
    }

    fn resize(&mut self, size: Size) {
        self.view.resize(Size {
            width: size.width,
            height: size.height.saturating_sub(2),
        });
        self.message_bar.resize(Size {
            width: size.width,
            height: 1,
        });
        self.status_bar.resize(Size {
            height: 1,
            width: size.width,
        });
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_cursor();
        let size = Terminal::size().unwrap();
        if size.height == 0 || size.width == 0 {
            return;
        }
        if size.height > 2 {
            self.view.render(0);
        }
        let cursor_position = match self.mode {
            Mode::Edit => {
                self.message_bar.render(size.height as usize);
                self.view.get_position()
            }
            Mode::Search => {
                self.search_bar.render(size.height as usize);
                self.search_bar.get_position(size)
            }
        };
        if size.height > 1 {
            self.status_bar
                .render(size.height.saturating_sub(2) as usize);
        }
        let _ = Terminal::move_cursor(cursor_position);
        Terminal::show_cursor().unwrap();
        Terminal::execute().unwrap();
    }

    fn save_file(&mut self) -> Result<String, String> {
        if let Some(file_path) = &self.file_path {
            match self.view.buffer.write_to_file(file_path.clone()) {
                Ok(()) => Ok(String::from("File saved successfully.")),
                Err(_) => Err(String::from("Could not save file!")),
            }
        } else {
            Err(String::from("No such file name"))
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
