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

    pub fn search_document(&self, query: &str) -> Vec<(usize, usize)> {
        let document_string = self.document.to_string();
        let document_string_iter = document_string.match_indices(query);
        let mut results = Vec::new();
        for (byte_idx, _) in document_string_iter {
            let line_idx = self.document.byte_to_line(byte_idx);
            let start_of_line = self.document.line_to_char(line_idx);
            let match_char_idx = self.document.byte_to_char(byte_idx);
            let char_idx_line = match_char_idx.saturating_sub(start_of_line);
            results.push((line_idx, char_idx_line));
        }
        results
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.document = Rope::new();
    }
}
