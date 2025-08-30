use std::ops::Range;

use ropey::RopeSlice;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum GraphemeWidth {
    Zero,
    Half,
    Full,
}

impl GraphemeWidth {
    const fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Zero => other.saturating_add(0),
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

pub struct Line {
    fragments: Vec<TextFragment>,
}

impl From<RopeSlice<'_>> for Line {
    fn from(r_slice: RopeSlice) -> Self {
        let text = String::from(r_slice);
        let fragments: Vec<TextFragment> = text
            .graphemes(true)
            .map(|g| {
                let width = Line::detect_width(g);
                let replacement = Line::replacement_character(g);
                TextFragment {
                    grapheme: String::from(g),
                    rendered_width: width,
                    replacement,
                }
            })
            .collect();
        Self { fragments }
    }
}

impl Line {
    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        if range.start >= range.end {
            return String::new();
        }
        let mut text = String::new();
        let mut curr_pos = 0;
        for fragment in &self.fragments {
            let frag_end = fragment.rendered_width.saturating_add(curr_pos);
            if curr_pos >= range.end {
                break;
            }
            if frag_end > range.start {
                if frag_end > range.end || curr_pos < range.start {
                    text.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    text.push(char);
                } else {
                    text.push_str(&fragment.grapheme);
                }
            }
            curr_pos = frag_end;
        }
        text
    }

    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
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
}
