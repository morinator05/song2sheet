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

/// Gap between the staves of a system on the sheet.
pub const STAFF_GAP: f32 = 1.5;
/// Gap between systems on the sheet.
pub const SYSTEM_GAP: f32 = 3.0;
/// Gap between staves on screen, where many staves must not run into
/// each other.
pub const SCREEN_STAFF_GAP: f32 = 4.0;
/// Gap between systems on screen, with room for a separator line.
pub const SCREEN_SYSTEM_GAP: f32 = 8.0;

/// Vertical room between staves and between systems.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spacing {
    pub staff_gap: f32,
    pub system_gap: f32,
}

impl Spacing {
    /// The printed sheet.
    pub const PRINT: Spacing = Spacing { staff_gap: STAFF_GAP, system_gap: SYSTEM_GAP };
    /// A section shown on its own in the app.
    pub const SCREEN: Spacing =
        Spacing { staff_gap: SCREEN_STAFF_GAP, system_gap: SCREEN_SYSTEM_GAP };
}

/// Bars are never stretched wider than this on average.
pub const MAX_BAR_W: f32 = 55.0;
/// Bars may be squeezed down to this fraction of their natural width.
pub const MIN_SCALE: f32 = 0.72;
pub const CHORD_SIZE: f32 = 3.0;

// --- Staff positions -------------------------------------------------
// Positions count half spaces ("steps") up from the bottom line of a
// 5-line staff: 0 = bottom line, 8 = top line.

/// Height of a 5-line staff.
pub const STAFF5_H: f32 = 4.0 * SP;
/// One staff step: line to adjacent space.
pub const HALF_SPACE: f32 = SP / 2.0;
pub const TOP_LINE_POS: i32 = 8;
pub const FIRST_LEDGER_ABOVE: i32 = 10;
pub const FIRST_LEDGER_BELOW: i32 = -2;
/// Ledger lines stick out this far on both sides of the notehead.
pub const LEDGER_OVERHANG: f32 = 0.4 * SP;
pub const LEDGER_W: f32 = LINE_W * 1.5;
/// Heads closer than this (a second apart) are put on both stem sides.
pub const SECOND_MAX_DIST: f32 = 0.6 * SP;

// --- Stems, dots, rests --------------------------------------------------

/// The stem starts this far inside the notehead.
pub const STEM_HEAD_OVERLAP: f32 = 0.17 * SP;
/// Tab rhythm stems: gap below the staff and length.
pub const TAB_STEM_GAP: f32 = 0.6 * SP;
pub const TAB_STEM_LEN: f32 = 2.5 * SP;
/// Space between a notehead (or rest) and its first dot, and between dots.
pub const DOT_GAP: f32 = 0.4 * SP;
pub const DOT_STEP: f32 = 0.5 * SP;
/// Dot position next to a tab stem (right of it, below its base).
pub const TAB_DOT_DX: f32 = 0.5 * SP;
pub const TAB_DOT_DY: f32 = 0.8 * SP;
/// The whole rest hangs from the line above the middle line.
pub const WHOLE_REST_RAISE: f32 = SP;
/// With two voices, rests move up (voice 1) or down (voice 2).
pub const REST_SHIFT_UP: f32 = SP;
pub const REST_SHIFT_DOWN: f32 = 1.5 * SP;

// --- Tab fret numbers ----------------------------------------------------

/// White box behind fret numbers: horizontal padding and height.
pub const FRET_KNOCKOUT_PAD: f32 = 0.2;
pub const FRET_KNOCKOUT_H: f32 = TAB_FRET_SIZE * 0.8;
/// Baseline offset that centres the digits on the string line.
pub const FRET_BASELINE_SHIFT: f32 = TAB_FRET_SIZE * 0.36;

// --- Articulations, beams, tuplets ---------------------------------------

/// Gap between the stem tip and the first articulation, above or below.
pub const ARTIC_GAP_ABOVE: f32 = 0.8 * SP;
pub const ARTIC_GAP_BELOW: f32 = 1.4 * SP;
/// Distance between stacked articulations, and from stemless heads.
pub const ARTIC_STEP: f32 = SP;
/// A beam segment of a lone short note (e.g. the 16th after a dotted 8th).
pub const BEAM_STUB: f32 = 1.2 * SP;
pub const TUPLET_SCALE: f32 = 0.75;
pub const TUPLET_GAP_ABOVE: f32 = 0.6 * SP;
pub const TUPLET_GAP_BELOW: f32 = 1.8 * SP;

// --- Staff frame ---------------------------------------------------------

pub const CLEF_X: f32 = 0.5 * SP;
/// The 4-string tab clef is drawn smaller to fit the narrower staff.
pub const TAB4_CLEF_SCALE: f32 = 0.8;
/// Time signature digits sit one space above and below the middle.
pub const TIME_DIGIT_OFFSET: f32 = SP;
/// Gap between the time signature and an opening repeat sign.
pub const REPEAT_GAP: f32 = 0.3 * SP;
/// Repeat sign: offsets of the thin line and of the dots from the thick line.
pub const REPEAT_THIN_X: f32 = 0.4 * SP;
pub const REPEAT_DOT_X: f32 = 0.8 * SP;
/// Clickable beat areas start this far left of the notehead.
pub const BEAT_BOX_LEAD: f32 = 0.3 * SP;
/// Gap below chord symbols and between neighbouring chord symbols.
pub const CHORD_GAP: f32 = 1.0;

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
            _ => STAFF5_H,
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
