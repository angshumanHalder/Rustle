use std::collections::HashMap;

use super::grapheme::HighlightType;

pub struct LanguageConfig {
    pub keywords: HashMap<String, HighlightType>,
    pub single_line_comment_start: Option<String>,
}

impl LanguageConfig {
    pub fn rust() -> Self {
        let mut keywords = HashMap::new();
        let rust_keywords = [
            "fn", "let", "mut", "struct", "enum", "impl", "use", "mod", "pub", "const", "static",
            "if", "else", "match", "loop", "while", "for", "in", "return", "break", "continue",
        ];
        for kw in rust_keywords {
            keywords.insert(kw.to_string(), HighlightType::Keyword);
        }

        let rust_types = [
            "bool", "char", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
            "u128", "usize", "f32", "f64", "String", "str", "Vec", "Option", "Result", "HashMap",
            "Box", "Rc", "Arc", "Cell", "RefCell",
        ];
        for t in rust_types {
            keywords.insert(t.to_string(), HighlightType::Type);
        }

        Self {
            keywords,
            single_line_comment_start: Some("//".to_string()),
        }
    }

    pub fn text() -> Self {
        Self {
            keywords: HashMap::new(),
            single_line_comment_start: None,
        }
    }
}
