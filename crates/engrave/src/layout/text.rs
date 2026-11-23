//! Text helpers: item constructors, word wrapping, tempo marks.

use model::Meta;

use crate::display::{Item, Shade};
use crate::fonts::{self, FontId};
use crate::smufl::MET_QUARTER_UP;

pub fn text(x: f32, y: f32, size: f32, font: FontId, s: impl Into<String>) -> Item {
    Item::Text { x, y, size, font, text: s.into() }
}

/// Text centred horizontally on `center_x`.
pub fn centered(center_x: f32, y: f32, size: f32, font: FontId, s: &str) -> Item {
    let w = fonts::text_width(font, size, s);
    text(center_x - w / 2.0, y, size, font, s)
}

/// Text whose right edge is at `right_x`.
pub fn right_aligned(right_x: f32, y: f32, size: f32, font: FontId, s: &str) -> Item {
    let w = fonts::text_width(font, size, s);
    text(right_x - w, y, size, font, s)
}

/// Horizontal rule from `x0` to `x1`.
pub fn hrule(x0: f32, x1: f32, y: f32, width: f32, shade: Shade) -> Item {
    Item::Line { x1: x0, y1: y, x2: x1, y2: y, width, shade }
}

/// The song title, or a placeholder if none is set.
pub fn display_title(meta: &Meta) -> &str {
    let t = meta.title.trim();
    if t.is_empty() { "Untitled" } else { t }
}

/// Tempo marking "♩ = 120" with its baseline at `y`.
pub fn tempo_mark(x: f32, y: f32, bpm: u32, size: f32) -> Vec<Item> {
    let note = MET_QUARTER_UP.to_string();
    let note_size = size * 1.5;
    let w = fonts::text_width(FontId::Music, note_size, &note);
    vec![
        text(x, y, note_size, FontId::Music, note),
        text(x + w + 0.6, y, size, FontId::Regular, format!("= {bpm}")),
    ]
}

/// Greedy word wrap to `width`. Explicit newlines start new lines, words
/// longer than a line are broken between characters, and trailing empty
/// lines are dropped.
pub fn wrap(s: &str, font: FontId, size: f32, width: f32) -> Vec<String> {
    let fits = |t: &str| fonts::text_width(font, size, t) <= width;
    let mut out = Vec::new();
    for para in s.lines() {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let candidate =
                if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if fits(&candidate) {
                line = candidate;
                continue;
            }
            if !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            // Start the word on a fresh line, breaking it if it is too long.
            for c in word.chars() {
                line.push(c);
                if !fits(&line) && line.chars().count() > 1 {
                    line.pop();
                    out.push(std::mem::replace(&mut line, c.to_string()));
                }
            }
        }
        out.push(line);
    }
    while out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: f32 = 3.0;

    #[test]
    fn wraps_words_and_keeps_newlines() {
        let lines = wrap("one two three\nfour\n\n", FontId::Regular, SIZE, 1000.0);
        assert_eq!(lines, ["one two three", "four"]);
        let narrow = fonts::text_width(FontId::Regular, SIZE, "one two");
        assert_eq!(wrap("one two three", FontId::Regular, SIZE, narrow), ["one two", "three"]);
    }

    #[test]
    fn breaks_overlong_words() {
        let width = fonts::text_width(FontId::Regular, SIZE, "abcd");
        let lines = wrap("abcdefghij", FontId::Regular, SIZE, width);
        assert!(lines.len() >= 3);
        assert_eq!(lines.concat(), "abcdefghij");
        assert!(lines.iter().all(|l| fonts::text_width(FontId::Regular, SIZE, l) <= width));
    }

    #[test]
    fn empty_text_has_no_lines() {
        assert!(wrap("", FontId::Regular, SIZE, 50.0).is_empty());
    }
}
