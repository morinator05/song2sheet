//! A single section on its own, for the app's section editor and import
//! overview.

use model::{Segment, Staff};

use super::row::{NOTATION_W, has_staff_labels, label_strip_w};
use crate::display::Page;
use crate::notation::{self, BeatBox, Spacing, StaffLabel, SystemSpan};

const MARGIN: f32 = 2.0;
/// Minimum view height, so an empty section still shows something.
const MIN_HEIGHT: f32 = 10.0;

/// A section engraved as wide as its staves are on the sheet, so the app
/// shows the same line breaks as the printout of a full score without
/// repeat signs (repeats and a part filter can change the breaks slightly).
/// The staves get more room between them than on paper, and their names
/// are left to the app to draw.
pub struct SegmentView {
    /// Page sized to the content.
    pub page: Page,
    /// Where each beat is, in page coordinates (mm).
    pub beats: Vec<BeatBox>,
    /// Staff names to draw left of the page, in page coordinates (none for
    /// a single staff, as on the sheet).
    pub staff_labels: Vec<StaffLabel>,
    /// Where each system is, top to bottom, in page coordinates.
    pub systems: Vec<SystemSpan>,
}

pub fn layout_segment(seg: &Segment) -> SegmentView {
    let staves: Vec<&Staff> = seg.staves.iter().collect();
    let width = NOTATION_W - label_strip_w(staves.len());
    let mut block = notation::engrave_segment(seg, &staves, width, false, Spacing::SCREEN);
    block.items.iter_mut().for_each(|it| it.translate(MARGIN, MARGIN));
    block.beats.iter_mut().for_each(|b| b.translate(MARGIN, MARGIN));
    block.systems.iter_mut().for_each(|s| s.translate(MARGIN));
    block.staff_labels.iter_mut().for_each(|l| l.y += MARGIN);
    if !has_staff_labels(staves.len()) {
        block.staff_labels.clear();
    }
    let bottom = if block.height > 0.0 { MARGIN + block.height } else { 0.0 };
    let page = Page {
        width: width + 2.0 * MARGIN,
        height: bottom.max(MIN_HEIGHT) + MARGIN,
        items: block.items,
    };
    SegmentView { page, beats: block.beats, staff_labels: block.staff_labels, systems: block.systems }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FontId, Item};
    use model::{Bar, Beat, Duration, NoteValue, StaffKind};

    /// `staff_count` drum staves of `bar_count` bars of eighth notes.
    fn segment(staff_count: usize, bar_count: usize) -> Segment {
        let eighths = vec![Beat::rest(Duration::new(NoteValue::Eighth)); 8];
        let mut staff = Staff::new(StaffKind::Drums);
        staff.bars = vec![Bar::single(eighths); bar_count];
        Segment { staves: vec![staff; staff_count], ..Default::default() }
    }

    #[test]
    fn every_beat_gets_a_box() {
        let mut staff = Staff::new(StaffKind::Drums);
        let beats = vec![Beat::rest(Duration::new(NoteValue::Half)); 2];
        staff.bars = vec![
            Bar { voices: vec![beats.clone(), beats] },
            Bar::single(vec![Beat::rest(Duration::new(NoteValue::Whole))]),
        ];
        let seg = Segment { staves: vec![staff], ..Default::default() };
        let view = layout_segment(&seg);
        assert_eq!(view.beats.len(), 5);
        assert!(view.beats.iter().all(|b| b.x0 < b.x1 && b.y0 < b.y1));
        assert!(view.beats.iter().all(|b| b.x1 <= view.page.width && b.y1 <= view.page.height));
    }

    #[test]
    fn screen_spacing_keeps_the_sheet_line_breaks() {
        let seg = segment(3, 12);
        let staves: Vec<&Staff> = seg.staves.iter().collect();
        let width = NOTATION_W - label_strip_w(staves.len());
        let print = notation::engrave_segment(&seg, &staves, width, false, Spacing::PRINT);
        let view = layout_segment(&seg);
        assert!(view.systems.len() > 1, "the section must wrap for this test");
        let bars = |s: &[SystemSpan]| s.iter().map(|s| s.bars.clone()).collect::<Vec<_>>();
        assert_eq!(bars(&view.systems), bars(&print.systems));
        assert_eq!(view.beats.len(), print.beats.len());
        for (screen, sheet) in view.beats.iter().zip(&print.beats) {
            assert!((screen.x0 - MARGIN - sheet.x0).abs() < 1e-3, "{screen:?} vs {sheet:?}");
        }
        let gap = |s: &[SystemSpan]| s[1].top - s[0].bottom;
        assert!(gap(&view.systems) > gap(&print.systems), "more room between systems");
    }

    #[test]
    fn staff_names_are_left_to_the_app() {
        let view = layout_segment(&segment(3, 12));
        assert_eq!(view.staff_labels.len(), 3 * view.systems.len());
        let engraved = |i: &Item| matches!(i, Item::Text { font: FontId::Italic, .. });
        assert!(!view.page.items.iter().any(engraved), "no engraved staff names");
        assert!(layout_segment(&segment(1, 1)).staff_labels.is_empty(), "one staff needs no name");
    }
}
