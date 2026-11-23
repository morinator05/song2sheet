//! A single beat ("event") of one voice: noteheads, stem, rest, dots,
//! fret numbers and articulations. Beams and flags live in `beams`.

use model::{Beat, DrumHit, NoteValue};

use super::metrics::*;
use crate::display::{Item, Shade};
use crate::fonts::{self, FontId};
use crate::smufl::*;

#[derive(Clone, Copy, Debug)]
pub struct Head {
    pub y: f32,
    pub glyph: char,
    /// Horizontal displacement (for seconds on the same stem).
    pub dx: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Stem {
    pub x: f32,
    /// End at the notehead.
    pub base: f32,
    /// Free end (where flags or beams attach).
    pub tip: f32,
}

pub struct Event<'a> {
    pub beat: &'a Beat,
    pub onset: u32,
    pub x: f32,
    pub up: bool,
    pub heads: Vec<Head>,
    pub head_w: f32,
    pub stem: Option<Stem>,
}

impl<'a> Event<'a> {
    pub fn new(beat: &'a Beat, onset: u32, x: f32, up: bool, style: Style) -> Self {
        let value = beat.duration.value;
        let mut heads = heads_for(beat, style);
        let head_w = match style {
            Style::Tab(_) => TAB_HEAD_W,
            _ => heads.iter().map(|h| glyph_w(h.glyph)).fold(0.0, f32::max),
        };
        displace_seconds(&mut heads, head_w, up);

        let has_stem = !beat.is_rest()
            && value.has_stem()
            && (matches!(style, Style::Tab(_)) || !heads.is_empty());
        let stem = has_stem.then(|| {
            // 32nds need longer stems to fit three beams.
            let extra = value.beams().saturating_sub(2) as f32 * BEAM_STEP;
            if let Style::Tab(_) = style {
                let base = style.height() + 0.6 * SP;
                return Stem { x: x + head_w / 2.0, base, tip: base + 2.5 * SP + extra };
            }
            let top = heads.iter().map(|h| h.y).fold(f32::MAX, f32::min);
            let bottom = heads.iter().map(|h| h.y).fold(f32::MIN, f32::max);
            if up {
                Stem {
                    x: x + head_w - STEM_W / 2.0,
                    base: bottom - 0.17 * SP,
                    tip: top - STEM_LEN - extra,
                }
            } else {
                Stem { x: x + STEM_W / 2.0, base: top + 0.17 * SP, tip: bottom + STEM_LEN + extra }
            }
        });
        Self { beat, onset, x, up, heads, head_w, stem }
    }

    pub fn value(&self) -> NoteValue {
        self.beat.duration.value
    }

    pub fn is_note(&self) -> bool {
        !self.beat.is_rest()
    }

    /// Draws everything except beams, flags and tuplet numbers.
    /// `rest_y` is the baseline for rests of this voice.
    pub fn draw(&self, items: &mut Vec<Item>, style: Style, rest_y: f32) {
        if !self.is_note() {
            self.draw_rest(items, rest_y);
            return;
        }
        if let Style::Tab(_) = style {
            self.draw_frets(items);
        }
        self.draw_heads(items, style);
        self.draw_dots(items);
        if let Some(s) = self.stem {
            items.push(line(s.x, s.base, s.x, s.tip, STEM_W));
        }
        self.draw_articulations(items);
    }

    fn draw_rest(&self, items: &mut Vec<Item>, rest_y: f32) {
        let glyph = rest_glyph(self.value());
        // The whole rest hangs from the line above the middle line.
        let y = if self.value() == NoteValue::Whole { rest_y - SP } else { rest_y };
        items.push(music(self.x, y, glyph));
        for d in 0..self.beat.duration.dots {
            let dx = self.x + glyph_w(glyph) + 0.4 * SP + d as f32 * 0.5 * SP;
            items.push(music(dx, rest_y - SP / 2.0, AUG_DOT));
        }
    }

    /// Fret numbers on a tab staff, knocked out of the string lines.
    fn draw_frets(&self, items: &mut Vec<Item>) {
        let cx = self.x + self.head_w / 2.0;
        for n in &self.beat.tab {
            let y = n.string.saturating_sub(1) as f32 * TAB_SP;
            let text = n.fret.to_string();
            let w = fonts::text_width(FontId::Regular, TAB_FRET_SIZE, &text);
            items.push(Item::Rect {
                x: cx - w / 2.0 - 0.2,
                y: y - TAB_FRET_SIZE * 0.4,
                w: w + 0.4,
                h: TAB_FRET_SIZE * 0.8,
                shade: Shade::WHITE,
            });
            let baseline = y + TAB_FRET_SIZE * 0.36;
            items.push(Item::Text {
                x: cx - w / 2.0,
                y: baseline,
                size: TAB_FRET_SIZE,
                font: FontId::Regular,
                text,
            });
        }
    }

    fn draw_heads(&self, items: &mut Vec<Item>, style: Style) {
        for h in &self.heads {
            let hx = self.x + h.dx;
            items.push(music(hx, h.y, h.glyph));
            if style == Style::Percussion {
                for ly in ledger_lines(y_pos(h.y)) {
                    items.push(line(
                        hx - 0.4 * SP,
                        ly,
                        hx + self.head_w + 0.4 * SP,
                        ly,
                        LINE_W * 1.5,
                    ));
                }
            }
        }
    }

    fn draw_dots(&self, items: &mut Vec<Item>) {
        let dots = self.beat.duration.dots;
        if dots == 0 {
            return;
        }
        let right = self
            .heads
            .iter()
            .map(|h| self.x + h.dx + self.head_w)
            .fold(self.x + self.head_w, f32::max);
        for d in 0..dots {
            let step = d as f32 * 0.5 * SP;
            if self.heads.is_empty() {
                // Tab: dot next to the stem.
                if let Some(s) = self.stem {
                    items.push(music(s.x + 0.5 * SP + step, s.base + 0.8 * SP, AUG_DOT));
                }
            }
            for h in &self.heads {
                // Dots sit in spaces: move up from a line.
                let dy = if y_pos(h.y) % 2 == 0 { -SP / 2.0 } else { 0.0 };
                items.push(music(right + 0.4 * SP + step, h.y + dy, AUG_DOT));
            }
        }
    }

    fn draw_articulations(&self, items: &mut Vec<Item>) {
        let cx = self.x + self.head_w / 2.0;
        for (c, y) in self.articulations().0 {
            items.push(music(cx - glyph_w(c) / 2.0, y, c));
        }
    }

    /// Accent and open hi-hat signs with their baselines, stacked outward
    /// from the stem tip (or the heads if there is no stem), plus the
    /// outermost y used. Tuplet numbers go beyond that edge.
    pub fn articulations(&self) -> (Vec<(char, f32)>, f32) {
        let mut signs = Vec::new();
        if self.beat.accent {
            signs.push(if self.up { ACCENT_ABOVE } else { ACCENT_BELOW });
        }
        if self.beat.drums.contains(&DrumHit::HiHatOpen) {
            signs.push(OPEN);
        }
        let mut edge = self.stem.map(|s| s.tip).unwrap_or_else(|| {
            let ys = self.heads.iter().map(|h| h.y);
            if self.up { ys.fold(0.0, f32::min) - SP } else { ys.fold(4.0 * SP, f32::max) + SP }
        });
        let placed = signs
            .into_iter()
            .map(|c| {
                let y = if self.up { edge - 0.8 * SP } else { edge + 1.4 * SP };
                edge = if self.up { y - SP } else { y + SP };
                (c, y)
            })
            .collect();
        (placed, edge)
    }
}

fn heads_for(beat: &Beat, style: Style) -> Vec<Head> {
    let value = beat.duration.value;
    let head = |y, glyph| Head { y, glyph, dx: 0.0 };
    match style {
        Style::Percussion => {
            let mut v: Vec<(i32, char)> = beat.drums.iter().map(|&d| drum_head(d, value)).collect();
            v.sort_by_key(|h| h.0);
            // Instruments sharing a position (e.g. snare + side stick):
            // only the first listed is drawn.
            v.dedup_by_key(|h| h.0);
            v.into_iter().map(|(pos, glyph)| head(pos_y(pos), glyph)).collect()
        }
        Style::Slash if !beat.is_rest() => {
            let glyph = match value {
                NoteValue::Whole => SLASH_WHOLE,
                NoteValue::Half => SLASH_HALF,
                _ => SLASH_BLACK,
            };
            vec![head(2.0 * SP, glyph)]
        }
        _ => Vec::new(),
    }
}

/// Moves the upper (stem up) or lower (stem down) head of a second to the
/// other side of the stem so the two don't overlap.
fn displace_seconds(heads: &mut [Head], head_w: f32, up: bool) {
    if up {
        heads.sort_by(|a, b| b.y.total_cmp(&a.y));
    } else {
        heads.sort_by(|a, b| a.y.total_cmp(&b.y));
    }
    let mut prev: Option<(f32, bool)> = None;
    for h in heads {
        let collides =
            prev.is_some_and(|(py, displaced)| !displaced && (h.y - py).abs() < SP * 0.6);
        if collides {
            h.dx = if up { head_w } else { -head_w };
        }
        prev = Some((h.y, collides));
    }
}

/// Staff position (half spaces above the bottom line) and notehead of a drum.
fn drum_head(hit: DrumHit, value: NoteValue) -> (i32, char) {
    let pick = |black, half, whole| match value {
        NoteValue::Whole => whole,
        NoteValue::Half => half,
        _ => black,
    };
    let normal = pick(NOTEHEAD_BLACK, NOTEHEAD_HALF, NOTEHEAD_WHOLE);
    let x = pick(NOTEHEAD_X_BLACK, NOTEHEAD_X_HALF, NOTEHEAD_X_WHOLE);
    match hit {
        DrumHit::Kick => (1, normal),
        DrumHit::TomFloor => (3, normal),
        DrumHit::Snare => (5, normal),
        DrumHit::SideStick => (5, NOTEHEAD_CIRCLE_X),
        DrumHit::TomMid => (6, normal),
        DrumHit::TomHigh => (7, normal),
        DrumHit::Cowbell => (7, NOTEHEAD_TRIANGLE_UP),
        DrumHit::Ride => (8, x),
        DrumHit::RideBell => (8, NOTEHEAD_DIAMOND_BLACK),
        DrumHit::HiHatClosed | DrumHit::HiHatOpen => (9, x),
        DrumHit::HiHatPedal => (-1, x),
        DrumHit::Crash => (10, x),
        DrumHit::China | DrumHit::Splash => (11, x),
    }
}

/// y of a staff position (0 = bottom line of a 5-line staff, 8 = top line).
fn pos_y(pos: i32) -> f32 {
    (8 - pos) as f32 * SP / 2.0
}

/// Inverse of [`pos_y`].
fn y_pos(y: f32) -> i32 {
    8 - (y / (SP / 2.0)).round() as i32
}

/// y of the ledger lines needed for a head at `pos`.
fn ledger_lines(pos: i32) -> Vec<f32> {
    let positions: Vec<i32> = if pos >= 10 {
        (10..=pos).step_by(2).collect()
    } else if pos <= -2 {
        (pos..=-2).filter(|p| p % 2 == 0).collect()
    } else {
        Vec::new()
    };
    positions.into_iter().map(pos_y).collect()
}

fn rest_glyph(v: NoteValue) -> char {
    match v {
        NoteValue::Whole => REST_WHOLE,
        NoteValue::Half => REST_HALF,
        NoteValue::Quarter => REST_QUARTER,
        NoteValue::Eighth => REST_8TH,
        NoteValue::Sixteenth => REST_16TH,
        NoteValue::ThirtySecond => REST_32ND,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_lines_only_outside_staff() {
        assert!(ledger_lines(9).is_empty());
        assert_eq!(ledger_lines(10).len(), 1);
        assert_eq!(ledger_lines(12).len(), 2);
        assert!(ledger_lines(-1).is_empty());
        assert_eq!(ledger_lines(-2).len(), 1);
    }

    #[test]
    fn position_roundtrip() {
        for p in -4..14 {
            assert_eq!(y_pos(pos_y(p)), p);
        }
    }

    #[test]
    fn snare_and_mid_tom_are_displaced() {
        let mut heads = vec![
            Head { y: pos_y(5), glyph: NOTEHEAD_BLACK, dx: 0.0 },
            Head { y: pos_y(6), glyph: NOTEHEAD_BLACK, dx: 0.0 },
        ];
        displace_seconds(&mut heads, 2.0, true);
        assert_eq!(heads.iter().filter(|h| h.dx != 0.0).count(), 1);
    }
}
