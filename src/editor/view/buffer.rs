use std::{fs::File, io::Error};

use ropey::{Rope, RopeSlice};

pub struct Buffer {
    pub document: Rope,
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
        Self { document }
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
    }

    pub fn delete_range_char(&mut self, pos: usize, len: usize) {
        let end = pos.saturating_add(len);
        self.document.remove(pos..end);
    }

    pub fn write_to_file(&self, path: String) -> Result<(), Error> {
        let file = File::create(path)?;
        self.document.write_to(file)?;
        Ok(())
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.document = Rope::new();
    }
}
