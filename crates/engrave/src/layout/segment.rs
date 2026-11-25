//! A single section on its own, for the section editor.

use model::{Segment, Staff};

use super::row::{NOTATION_W, place_notation};
use crate::display::Page;
use crate::notation::BeatBox;

const MARGIN: f32 = 2.0;
/// Minimum view height, so an empty section still shows something.
const MIN_HEIGHT: f32 = 10.0;

/// A section engraved as wide as on the sheet, so the editor shows the
/// same line breaks as the printout of a full score without repeat signs
/// (repeats and a part filter can change the breaks slightly).
pub struct SegmentView {
    /// Page sized to the content.
    pub page: Page,
    /// Where each beat is, in page coordinates (mm).
    pub beats: Vec<BeatBox>,
}

pub fn layout_segment(seg: &Segment) -> SegmentView {
    let staves: Vec<&Staff> = seg.staves.iter().collect();
    let placed = place_notation(seg, &staves, false, MARGIN, MARGIN, MARGIN + NOTATION_W);
    let page = Page {
        width: NOTATION_W + 2.0 * MARGIN,
        height: placed.bottom.max(MIN_HEIGHT) + MARGIN,
        items: placed.items,
    };
    SegmentView { page, beats: placed.beats }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{Bar, Beat, Duration, NoteValue, StaffKind};

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
}
