//! One sheet row: left column (name, tempo, repeats), notation, comments.

use model::{PartFilter, Project, Segment, SheetEntry, Staff};

use super::text::{right_aligned, tempo_mark, text, wrap};
use crate::display::Item;
use crate::fonts::FontId;
use crate::notation::{self, StaffLabel};

/// Width of the left column with section name, tempo and repeat count.
const LEFT_COL: f32 = 30.0;
/// Extra indent for staff labels when a row shows more than one staff.
const STAFF_LABEL_W: f32 = 7.0;
const ROW_PAD: f32 = 2.0;
/// Space between the notation and the first comment line.
const COMMENT_GAP: f32 = 1.0;

const LABEL_SIZE: f32 = 3.6;
const LABEL_LINE_H: f32 = LABEL_SIZE * 1.2;
const TEMPO_SIZE: f32 = 3.0;
const TEMPO_LINE_H: f32 = TEMPO_SIZE * 1.6;
const REPEAT_LINE_H: f32 = TEMPO_SIZE * 1.5;
const COMMENT_SIZE: f32 = 2.9;
const COMMENT_LINE_H: f32 = COMMENT_SIZE * 1.3;
const STAFF_LABEL_SIZE: f32 = 2.5;

/// A laid-out row; items are relative to (content left, row top).
pub struct Row {
    pub items: Vec<Item>,
    pub height: f32,
}

/// Lays out one sheet entry across `width` (the full content width).
pub fn layout_row(
    project: &Project,
    entry: &SheetEntry,
    seg: &Segment,
    filter: PartFilter,
    width: f32,
) -> Row {
    let (mut items, left_bottom) = left_column(project, entry, seg);
    let (notes, notes_bottom) = notation(entry, seg, filter, width);
    items.extend(notes);

    let mut y = if notes_bottom > 0.0 { notes_bottom + COMMENT_GAP } else { ROW_PAD };
    for comment in comment_lines(entry, filter) {
        for line in wrap(&comment, FontId::Italic, COMMENT_SIZE, width - LEFT_COL) {
            y += COMMENT_LINE_H;
            items.push(text(LEFT_COL, y, COMMENT_SIZE, FontId::Italic, line));
        }
    }
    Row { items, height: left_bottom.max(notes_bottom).max(y) + ROW_PAD }
}

/// Section name (wrapped), tempo and "×N". Returns items and bottom y.
fn left_column(project: &Project, entry: &SheetEntry, seg: &Segment) -> (Vec<Item>, f32) {
    let mut items = Vec::new();
    let label = if entry.label.trim().is_empty() { seg.name.trim() } else { entry.label.trim() };
    let mut y = ROW_PAD;
    for (i, line) in wrap(label, FontId::Bold, LABEL_SIZE, LEFT_COL - 2.0).into_iter().enumerate() {
        y += if i == 0 { LABEL_SIZE } else { LABEL_LINE_H };
        items.push(text(0.0, y, LABEL_SIZE, FontId::Bold, line));
    }
    if let Some(bpm) = seg.tempo.or(project.meta.tempo) {
        y += TEMPO_LINE_H;
        items.extend(tempo_mark(0.0, y, bpm, TEMPO_SIZE));
    }
    if entry.repeat > 1 {
        y += REPEAT_LINE_H;
        items.push(text(0.0, y, TEMPO_SIZE, FontId::Bold, format!("×{}", entry.repeat)));
    }
    (items, y + 1.0)
}

/// The engraved staves shown by `filter`. Returns items and bottom y
/// (0 if nothing is shown).
fn notation(entry: &SheetEntry, seg: &Segment, filter: PartFilter, width: f32) -> (Vec<Item>, f32) {
    let staves: Vec<&Staff> = seg.staves.iter().filter(|s| filter.shows(s.kind)).collect();
    let labelled = staves.len() > 1;
    let x = LEFT_COL + if labelled { STAFF_LABEL_W } else { 0.0 };
    let y = ROW_PAD + 1.0;
    let block = notation::engrave_segment(seg, &staves, width - x, entry.repeat > 1);
    let mut items = block.items;
    items.iter_mut().for_each(|it| it.translate(x, y));
    if labelled {
        items.extend(staff_labels(&block.staff_labels, x - 1.5, y));
    }
    let bottom = if block.height > 0.0 { y + block.height } else { 0.0 };
    (items, bottom)
}

/// Small italic staff names, right-aligned at `right_x`.
fn staff_labels(labels: &[StaffLabel], right_x: f32, dy: f32) -> Vec<Item> {
    labels
        .iter()
        .map(|l| {
            let baseline = dy + l.y + STAFF_LABEL_SIZE * 0.35;
            right_aligned(right_x, baseline, STAFF_LABEL_SIZE, FontId::Italic, &l.text)
        })
        .collect()
}

/// The comments visible with `filter`: the general one, then part comments.
/// On a full score, part comments are prefixed with the part name.
fn comment_lines(entry: &SheetEntry, filter: PartFilter) -> Vec<String> {
    let general = Some(entry.comment.trim()).filter(|c| !c.is_empty()).map(str::to_string);
    let parts = entry
        .part_comments
        .iter()
        .filter(|pc| filter.shows(pc.kind) && !pc.text.trim().is_empty())
        .map(|pc| match filter {
            PartFilter::All => format!("{}: {}", pc.kind.name(), pc.text.trim()),
            PartFilter::Only(_) => pc.text.trim().to_string(),
        });
    general.into_iter().chain(parts).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{PartComment, SegmentId, StaffKind};

    fn entry() -> SheetEntry {
        let mut e = SheetEntry::new(SegmentId(1));
        e.comment = "loud".into();
        e.part_comments = vec![
            PartComment { kind: StaffKind::Drums, text: "ride".into() },
            PartComment { kind: StaffKind::Bass, text: "  ".into() },
        ];
        e
    }

    #[test]
    fn full_score_prefixes_part_comments() {
        assert_eq!(comment_lines(&entry(), PartFilter::All), ["loud", "Drums: ride"]);
    }

    #[test]
    fn part_sheet_shows_only_its_comments() {
        assert_eq!(comment_lines(&entry(), PartFilter::Only(StaffKind::Drums)), ["loud", "ride"]);
        assert_eq!(comment_lines(&entry(), PartFilter::Only(StaffKind::Guitar)), ["loud"]);
    }
}
