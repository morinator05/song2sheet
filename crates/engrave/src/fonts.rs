//! Bundled fonts and simple metrics (no shaping; Latin text and SMuFL glyphs only).

use std::sync::OnceLock;
use ttf_parser::Face;

use crate::display::BBox;

pub static BRAVURA: &[u8] = include_bytes!("../../../assets/fonts/Bravura.otf");
pub static TEXT_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
pub static TEXT_BOLD: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Bold.ttf");
pub static TEXT_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Italic.ttf");

/// Font used by a text item. `Music` is Bravura (SMuFL code points).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontId {
    Music,
    Regular,
    Bold,
    Italic,
}

impl FontId {
    pub const ALL: [FontId; 4] = [FontId::Music, FontId::Regular, FontId::Bold, FontId::Italic];

    pub fn data(self) -> &'static [u8] {
        match self {
            FontId::Music => BRAVURA,
            FontId::Regular => TEXT_REGULAR,
            FontId::Bold => TEXT_BOLD,
            FontId::Italic => TEXT_ITALIC,
        }
    }

    /// Position in [`FontId::ALL`], for per-font lookup tables.
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Parsed font faces, shared by every renderer.
pub fn face(font: FontId) -> &'static Face<'static> {
    static FACES: OnceLock<Vec<Face<'static>>> = OnceLock::new();
    &FACES.get_or_init(|| {
        FontId::ALL
            .iter()
            .map(|f| Face::parse(f.data(), 0).expect("bundled font is valid"))
            .collect()
    })[font.index()]
}

/// One positioned glyph of a [`glyph_run`].
#[derive(Clone, Debug, PartialEq)]
pub struct RunGlyph {
    pub id: ttf_parser::GlyphId,
    /// Byte range of the source character in the text.
    pub text_range: std::ops::Range<usize>,
    /// Advance in em units.
    pub advance: f32,
}

/// Maps `text` to glyphs with plain advances: no shaping, kerning or
/// ligatures. Every renderer places glyphs with this, so measurement,
/// preview and PDF agree exactly. Characters missing from the font are
/// skipped.
pub fn glyph_run(font: FontId, text: &str) -> Vec<RunGlyph> {
    let face = face(font);
    let upem = face.units_per_em() as f32;
    text.char_indices()
        .filter_map(|(i, c)| {
            let id = face.glyph_index(c)?;
            let advance = face.glyph_hor_advance(id).unwrap_or(0) as f32 / upem;
            Some(RunGlyph { id, text_range: i..i + c.len_utf8(), advance })
        })
        .collect()
}

/// Width of `text` at em size `size` (same unit as the result).
pub fn text_width(font: FontId, size: f32, text: &str) -> f32 {
    glyph_run(font, text).iter().map(|g| g.advance).sum::<f32>() * size
}

/// Glyph bounding box in em units with y pointing **up** (font convention).
pub fn glyph_bbox(font: FontId, c: char) -> Option<BBox> {
    let face = face(font);
    let upem = face.units_per_em() as f32;
    let r = face.glyph_bounding_box(face.glyph_index(c)?)?;
    Some(BBox {
        x0: r.x_min as f32 / upem,
        y0: r.y_min as f32 / upem,
        x1: r.x_max as f32 / upem,
        y1: r.y_max as f32 / upem,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_matches_all() {
        for (i, f) in FontId::ALL.iter().enumerate() {
            assert_eq!(f.index(), i);
        }
    }
}
