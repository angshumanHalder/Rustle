use std::fs::File;

use ropey::Rope;

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

    pub fn get_line(&self, idx: usize, end: usize) -> Option<String> {
        if let Some(line) = self.document.get_line(idx) {
            if line.len_chars() == 0 {
                None
            } else {
                let bound = std::cmp::min(line.len_chars(), end);
                let text = line.slice(0..bound).to_string();
                Some(text)
            }
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.document.len_chars() == 0
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.document = Rope::new();
    }
}
