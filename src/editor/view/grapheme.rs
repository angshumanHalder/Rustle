use ropey::RopeSlice;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::LanguageConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightType {
    Normal,
    Keyword,
    Type,
    Number,
    String,
    Comment,
}

#[derive(Clone, Copy)]
pub enum GraphemeWidth {
    Zero,
    Half,
    Full,
}

impl GraphemeWidth {
    pub const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Zero => other.saturating_add(0),
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

pub struct TextFragment {
    pub grapheme: String,
    pub rendered_width: GraphemeWidth,
    replacement: Option<char>,
    pub highlight_type: HighlightType,
}

pub struct Line {
    pub fragments: Vec<TextFragment>,
}

impl Line {
    pub fn get_fragments_from_rope_slice(r_slice: RopeSlice, config: &LanguageConfig) -> Self {
        let text = String::from(r_slice);
        let fragments: Vec<TextFragment> = Line::highlight(text.as_str(), config);
        Self { fragments }
    }

    pub fn get_fragments_from_string(s: &str, config: &LanguageConfig) -> Self {
        let fragments = Line::highlight(s, config);
        Self { fragments }
    }

    pub fn grapheme_count(&self) -> usize {
        if self.fragments.is_empty() {
            return 0;
        }
        if self.fragments[self.fragments.len().saturating_sub(1)]
            .replacement
            .is_some()
        {
            self.fragments.len().saturating_sub(1)
        } else {
            self.fragments.len()
        }
    }

    pub fn width_until(&self, grapheme_idx: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|f| match f.rendered_width {
                GraphemeWidth::Zero => 0,
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn grapheme_to_char_idx(&self, grapheme_idx: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|f| f.grapheme.chars().count())
            .sum()
    }

    pub fn char_to_grapheme_idx(&self, char_idx: usize) -> usize {
        self.fragments
            .iter()
            .scan(0, |acc, f| {
                *acc += f.grapheme.chars().count();
                Some(*acc)
            })
            .position(|count| count > char_idx)
            .unwrap_or(self.fragments.len())
    }

    pub fn grapheme_char_len(&self, grapheme_idx: usize) -> Option<usize> {
        self.fragments
            .get(grapheme_idx)
            .map(|f| f.grapheme.chars().count())
    }

    fn detect_width(grapheme: &str) -> GraphemeWidth {
        match grapheme.width() {
            0 => GraphemeWidth::Zero,
            1 => GraphemeWidth::Half,
            _ => GraphemeWidth::Full,
        }
    }

    fn replacement_character(for_str: &str) -> Option<char> {
        let width = for_str.width();
        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 => {
                if for_str.trim().is_empty() {
                    return Some('␣');
                }
                let mut chars = for_str.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                None
            }
            _ if width == 0 => Some('.'),
            _ => None,
        }
    }

    pub fn highlight(line: &str, config: &LanguageConfig) -> Vec<TextFragment> {
        let mut fragments = Vec::new();
        let graphemes: Vec<(usize, &str)> = line.grapheme_indices(true).collect();
        let mut i = 0;

        while i < graphemes.len() {
            let (byte_idx, grapheme) = graphemes[i];
            let first_char = grapheme.chars().next().unwrap_or_default();

            if let Some(comment_start) = &config.single_line_comment_start {
                if line[byte_idx..].starts_with(comment_start) {
                    let comment_text = &line[byte_idx..];
                    fragments.extend(Line::create_fragments(comment_text, HighlightType::Comment));
                    break;
                }
            }

            if first_char == '"' {
                // Moved this out of an else-if to be safe
                // String literal
                let mut j = i + 1;
                while j < graphemes.len() {
                    if graphemes[j].1 == "\"" {
                        // This is the corrected, safe loop for checking backslashes
                        let mut backslashes = 0;
                        let mut k = j;
                        while k > 0 {
                            k -= 1; // Decrement first
                            if graphemes[k].1 == "\\" {
                                backslashes += 1;
                            } else {
                                break; // Stop when a non-backslash is found
                            }
                        }
                        if backslashes % 2 == 0 {
                            j += 1; // Include the closing quote
                            break;
                        }
                    }
                    j += 1;
                }
                let string_text = &line[byte_idx..graphemes.get(j).map_or(line.len(), |(b, _)| *b)];
                fragments.extend(Line::create_fragments(string_text, HighlightType::String));
                i = j;
            } else if first_char.is_ascii_digit() {
                // Number
                let mut j = i;
                while j < graphemes.len() {
                    let next_char = graphemes[j].1.chars().next().unwrap_or_default();
                    if !(next_char.is_ascii_digit() || (next_char == '.' && j > i)) {
                        break;
                    }
                    j += 1;
                }
                let number_text = &line[byte_idx..graphemes.get(j).map_or(line.len(), |(b, _)| *b)];
                fragments.extend(Line::create_fragments(number_text, HighlightType::Number));
                i = j;
            } else if first_char.is_alphabetic() || first_char == '_' {
                // Identifier, keyword, or type
                let mut j = i;
                while j < graphemes.len() {
                    let next_char = graphemes[j].1.chars().next().unwrap_or_default();
                    if !(next_char.is_alphanumeric() || next_char == '_') {
                        break;
                    }
                    j += 1;
                }
                let identifier_end_byte = graphemes.get(j).map_or(line.len(), |(b, _)| *b);
                let identifier = &line[byte_idx..identifier_end_byte];
                let highlight_type = config
                    .keywords
                    .get(identifier)
                    .copied()
                    .unwrap_or(HighlightType::Normal);
                fragments.extend(Line::create_fragments(identifier, highlight_type));
                i = j;
            } else {
                // Normal grapheme (punctuation, whitespace, etc.)
                fragments.extend(Line::create_fragments(grapheme, HighlightType::Normal));
                i += 1;
            }
        }

        fragments
    }

    fn create_fragments(text: &str, highlight_type: HighlightType) -> Vec<TextFragment> {
        text.graphemes(true)
            .map(|g| {
                let width = Line::detect_width(g);
                let replacement = Line::replacement_character(g);
                TextFragment {
                    grapheme: String::from(g),
                    rendered_width: width,
                    replacement,
                    highlight_type,
                }
            })
            .collect()
    }
}
