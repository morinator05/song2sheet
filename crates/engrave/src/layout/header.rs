//! The page-1 header: title, subtitle, info line and notes.

use model::{PartFilter, Project};

use super::text::{centered, display_title, hrule, right_aligned, tempo_mark, text, wrap};
use super::{CONTENT_W, MARGIN_X};
use crate::display::{A4_WIDTH, Item, Shade, bbox_of};
use crate::fonts::FontId;

const TITLE_SIZE: f32 = 6.5;
const SUBTITLE_SIZE: f32 = 4.0;
pub const INFO_SIZE: f32 = 3.3;
const SUBTITLE_GAP: f32 = 1.5;
const INFO_GAP: f32 = 2.5;
const NOTES_LINE_H: f32 = INFO_SIZE * 1.35;
/// Space between the last header line and the rule below it.
const RULE_GAP: f32 = 2.5;
/// Space between the rule and the first row.
const AFTER_RULE: f32 = 1.0;

/// Header items starting at `top`; returns them and the y where rows start.
pub fn header(project: &Project, filter: PartFilter, top: f32) -> (Vec<Item>, f32) {
    let m = &project.meta;
    let mut items = Vec::new();
    let center = A4_WIDTH / 2.0;
    let mut y = top + TITLE_SIZE;
    items.push(centered(center, y, TITLE_SIZE, FontId::Bold, display_title(m)));
    if !m.subtitle.trim().is_empty() {
        y += SUBTITLE_SIZE + SUBTITLE_GAP;
        items.push(centered(center, y, SUBTITLE_SIZE, FontId::Regular, m.subtitle.trim()));
    }

    // Info line: tempo, key and part on the left, artist on the right.
    let mut facts: Vec<String> = Vec::new();
    if !m.key.trim().is_empty() {
        facts.push(format!("Key: {}", m.key.trim()));
    }
    if let PartFilter::Only(k) = filter {
        facts.push(format!("Part: {}", k.name()));
    }
    let artist = m.artist.trim();
    if m.tempo.is_some() || !facts.is_empty() || !artist.is_empty() {
        y += INFO_SIZE + INFO_GAP;
        let mut x = MARGIN_X;
        if let Some(bpm) = m.tempo {
            let mark = tempo_mark(x, y, bpm, INFO_SIZE);
            x = bbox_of(&mark).map_or(x, |b| b.x1) + 3.0;
            items.extend(mark);
        }
        if !facts.is_empty() {
            items.push(text(x, y, INFO_SIZE, FontId::Regular, facts.join("   ")));
        }
        if !artist.is_empty() {
            items.push(right_aligned(A4_WIDTH - MARGIN_X, y, INFO_SIZE, FontId::Italic, artist));
        }
    }
    for line in wrap(&m.notes, FontId::Regular, INFO_SIZE, CONTENT_W) {
        y += NOTES_LINE_H;
        items.push(text(MARGIN_X, y, INFO_SIZE, FontId::Regular, line));
    }
    y += RULE_GAP;
    items.push(hrule(MARGIN_X, A4_WIDTH - MARGIN_X, y, 0.35, Shade::BLACK));
    (items, y + AFTER_RULE)
}
