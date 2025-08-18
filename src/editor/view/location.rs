use crate::editor::terminal::Position;

#[derive(Clone, Copy, Default)]
pub struct Location {
    pub col: u16,
    pub row: u16,
}

impl From<Location> for Position {
    fn from(value: Location) -> Self {
        Self {
            col: value.col,
            row: value.row,
        }
    }
}

impl Location {
    pub const fn subtract(self, other: Self) -> Self {
        Self {
            col: self.col.saturating_sub(other.col),
            row: self.row.saturating_sub(other.row),
        }
    }
}
