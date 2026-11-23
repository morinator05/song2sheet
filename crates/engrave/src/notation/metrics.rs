//! Engraving dimensions, staff styles and small drawing helpers.
//! All lengths are millimetres.

use model::{Staff, StaffKind};

use crate::display::{Item, Shade};
use crate::fonts::{self, FontId};

/// Staff space.
pub const SP: f32 = 1.5;
/// Em size of the music font (SMuFL: 1 em = 4 staff spaces).
pub const MUSIC: f32 = 4.0 * SP;

pub const LINE_W: f32 = 0.13 * SP;
pub const STEM_W: f32 = 0.12 * SP;
pub const BAR_W: f32 = 0.16 * SP;
pub const THICK_W: f32 = 0.5 * SP;
pub const BEAM_H: f32 = 0.5 * SP;
pub const BEAM_STEP: f32 = 0.75 * SP;
pub const STEM_LEN: f32 = 3.5 * SP;

pub const TAB_SP: f32 = 1.3 * SP;
pub const TAB_FRET_SIZE: f32 = 2.4;
pub const TAB_HEAD_W: f32 = 1.4 * SP;

/// Space before the first and after the last onset of a bar.
pub const PAD_L: f32 = 1.5 * SP;
pub const PAD_R: f32 = 1.0 * SP;
/// Horizontal room reserved at the start of systems.
pub const CLEF_W: f32 = 3.0 * SP;
/// Padding after the time signature digits.
pub const TIME_SIG_PAD: f32 = 1.0 * SP;
pub const REPEAT_W: f32 = 1.8 * SP;

pub const STAFF_GAP: f32 = 1.5;
pub const SYSTEM_GAP: f32 = 3.0;
/// Bars are never stretched wider than this on average.
pub const MAX_BAR_W: f32 = 55.0;
/// Bars may be squeezed down to this fraction of their natural width.
pub const MIN_SCALE: f32 = 0.72;
pub const CHORD_SIZE: f32 = 3.0;

/// How a staff is notated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Style {
    /// 5-line percussion staff.
    Percussion,
    /// 5-line staff with rhythm slashes (chord symbols above).
    Slash,
    /// Tab staff with this many strings; rhythm stems below.
    Tab(usize),
}

impl Style {
    pub fn of(staff: &Staff) -> Style {
        if staff.kind == StaffKind::Drums {
            Style::Percussion
        } else if staff.has_tab() {
            Style::Tab(staff.tuning().len().max(1))
        } else {
            Style::Slash
        }
    }

    /// Distance from the top to the bottom staff line.
    pub fn height(self) -> f32 {
        match self {
            Style::Tab(strings) => strings.saturating_sub(1) as f32 * TAB_SP,
            _ => 4.0 * SP,
        }
    }

    pub fn line_ys(self) -> Vec<f32> {
        match self {
            Style::Tab(strings) => (0..strings).map(|i| i as f32 * TAB_SP).collect(),
            _ => (0..5).map(|i| i as f32 * SP).collect(),
        }
    }
}

/// Advance width of a music glyph.
pub fn glyph_w(c: char) -> f32 {
    fonts::glyph_bbox(FontId::Music, c).map_or(SP, |b| b.x1 * MUSIC)
}

pub fn music(x: f32, y: f32, c: char) -> Item {
    music_scaled(x, y, c, 1.0)
}

pub fn music_scaled(x: f32, y: f32, c: char, scale: f32) -> Item {
    Item::Text { x, y, size: MUSIC * scale, font: FontId::Music, text: c.to_string() }
}

/// Baseline y that vertically centres glyph `c` on `mid`.
pub fn centered_baseline(c: char, mid: f32, scale: f32) -> f32 {
    fonts::glyph_bbox(FontId::Music, c).map_or(mid, |b| mid + (b.y0 + b.y1) / 2.0 * MUSIC * scale)
}

pub fn line(x1: f32, y1: f32, x2: f32, y2: f32, width: f32) -> Item {
    Item::Line { x1, y1, x2, y2, width, shade: Shade::BLACK }
}

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Item {
    Item::Rect { x, y, w, h, shade: Shade::BLACK }
}
