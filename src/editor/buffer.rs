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

    pub fn get_line(&self, idx: usize) -> Option<String> {
        if let Some(line) = self.document.get_line(idx) {
            let text = line.to_string();
            Some(text)
        } else {
            None
        }
    }
}
