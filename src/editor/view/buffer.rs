use std::{fs::File, io::Error};

use ropey::{Rope, RopeSlice};

pub struct Buffer {
    pub document: Rope,
    pub is_dirty: bool,
}

impl Buffer {
    pub fn new(file: Option<File>) -> Self {
        let mut document = Rope::new();
        if let Some(file) = file {
            document = match Rope::from_reader(file) {
                Ok(rope) => rope,
                Err(_) => Rope::new(),
            };
        }
        Self {
            document,
            is_dirty: false,
        }
    }

    pub fn get_line(&self, idx: usize) -> Option<RopeSlice> {
        self.document.get_line(idx)
    }

    pub fn line_count(&self) -> usize {
        self.document.len_lines()
    }

    pub fn is_empty(&self) -> bool {
        self.document.len_chars() == 0
    }

    pub fn insert_char(&mut self, pos: usize, ch: char) {
        self.document.insert_char(pos, ch);
        self.is_dirty = true;
    }

    pub fn delete_range_char(&mut self, pos: usize, len: usize) {
        let end = pos.saturating_add(len);
        self.document.remove(pos..end);
        self.is_dirty = true;
    }

    pub fn write_to_file(&mut self, path: String) -> Result<(), Error> {
        let file = File::create(path)?;
        self.document.write_to(file)?;
        self.is_dirty = false;
        Ok(())
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.document = Rope::new();
    }
}
