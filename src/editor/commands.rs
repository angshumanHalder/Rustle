use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::terminal::Size;

pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Insert(char),
    Ignore,
    Backspace,
    Delete,
    Quit,
}

impl TryFrom<&Event> for EditorCommand {
    type Error = String;

    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, *modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    Ok(Self::Insert(*c))
                }
                (KeyCode::Delete, _) => Ok(Self::Delete),
                (KeyCode::Backspace, _) => Ok(Self::Backspace),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::End, _) => Ok(Self::Move(Direction::End)),
                _ => Ok(Self::Ignore),
            },
            Event::Resize(width, height) => Ok(Self::Resize(Size {
                width: *width,
                height: *height,
            })),
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}
