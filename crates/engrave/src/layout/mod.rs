//! Song sheet page layout: header, one row per sheet entry, A4 pagination.

mod header;
mod row;
mod segment;
mod text;

pub use segment::{SegmentView, layout_segment};

use model::{PartFilter, Project};

use crate::display::{A4_HEIGHT, A4_WIDTH, DisplayList, Page, Shade};
use crate::fonts::FontId;
use header::{INFO_SIZE, header};
use row::layout_row;
use text::{display_title, hrule, right_aligned, text};

const MARGIN_X: f32 = 14.0;
const MARGIN_TOP: f32 = 12.0;
const MARGIN_BOTTOM: f32 = 12.0;
const FOOTER_H: f32 = 5.0;
const CONTENT_W: f32 = A4_WIDTH - 2.0 * MARGIN_X;
/// Running header on pages 2+: title line, then the rule.
const CONT_RULE_Y: f32 = MARGIN_TOP + 5.0;
const CONT_BODY_Y: f32 = CONT_RULE_Y + 2.0;
/// Rule under the header, and the thin rules between rows / under the
/// running header.
const HEADER_RULE_W: f32 = 0.35;
const THIN_RULE_W: f32 = 0.2;

/// Lays out the whole sheet on A4 pages. Rows are never split across
/// pages; a row taller than a whole page (very unlikely for a song sheet)
/// runs past the bottom margin.
pub fn layout_sheet(project: &Project, filter: PartFilter) -> DisplayList {
    let (head, body_top) = header(project, filter, MARGIN_TOP);
    let mut first = Page::a4();
    first.items.extend(head);
    let mut pages = vec![first];
    let bottom_limit = A4_HEIGHT - MARGIN_BOTTOM - FOOTER_H;
    let mut y = body_top;
    let mut rows_on_page = 0;

    for entry in &project.sheet {
        let Some(seg) = project.segment(entry.segment) else { continue };
        let row = layout_row(project, entry, seg, filter, CONTENT_W);
        if y + row.height > bottom_limit && rows_on_page > 0 {
            pages.push(continuation_page(project));
            y = CONT_BODY_Y;
            rows_on_page = 0;
        }
        let page = pages.last_mut().expect("at least one page");
        page.items.extend(row.items.into_iter().map(|mut it| {
            it.translate(MARGIN_X, y);
            it
        }));
        y += row.height;
        rows_on_page += 1;
        page.items.push(hrule(MARGIN_X, A4_WIDTH - MARGIN_X, y, THIN_RULE_W, Shade::LIGHT));
    }
    number_pages(&mut pages);
    DisplayList { pages }
}

/// Adds "n / total" footers when there is more than one page.
fn number_pages(pages: &mut [Page]) {
    let total = pages.len();
    if total < 2 {
        return;
    }
    for (i, page) in pages.iter_mut().enumerate() {
        let s = format!("{} / {total}", i + 1);
        page.items.push(right_aligned(
            A4_WIDTH - MARGIN_X,
            A4_HEIGHT - MARGIN_BOTTOM,
            INFO_SIZE,
            FontId::Regular,
            &s,
        ));
    }
}

fn continuation_page(project: &Project) -> Page {
    let mut page = Page::a4();
    let title = display_title(&project.meta);
    page.items.push(text(MARGIN_X, MARGIN_TOP + INFO_SIZE, INFO_SIZE, FontId::Italic, title));
    page.items.push(hrule(MARGIN_X, A4_WIDTH - MARGIN_X, CONT_RULE_Y, THIN_RULE_W, Shade::BLACK));
    page
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::Item;
    use model::{Bar, Beat, Duration, NoteValue, Segment, SheetEntry, Staff, StaffKind};

    fn project_with_rows(n: usize) -> Project {
        let mut p = Project::default();
        let mut staff = Staff::new(StaffKind::Guitar);
        staff
            .bars
            .push(Bar::single(vec![Beat::slash(Duration::new(NoteValue::Whole)).with_chord("Am")]));
        let id = p.add_segment(Segment {
            name: "Verse".into(),
            staves: vec![staff],
            ..Default::default()
        });
        p.sheet = (0..n).map(|_| SheetEntry::new(id)).collect();
        p
    }

    fn has_text(page: &Page, s: &str) -> bool {
        page.items.iter().any(|it| matches!(it, Item::Text { text, .. } if text == s))
    }

    #[test]
    fn short_sheet_is_one_unnumbered_page() {
        let list = layout_sheet(&project_with_rows(3), PartFilter::All);
        assert_eq!(list.pages.len(), 1);
        assert!(!has_text(&list.pages[0], "1 / 1"));
    }

    #[test]
    fn long_sheet_flows_onto_numbered_pages() {
        let list = layout_sheet(&project_with_rows(40), PartFilter::All);
        assert!(list.pages.len() >= 2);
        let total = list.pages.len();
        assert!(has_text(&list.pages[0], &format!("1 / {total}")));
        assert!(has_text(&list.pages[1], model::UNTITLED), "continuation header");
    }

    #[test]
    fn dangling_segment_reference_is_skipped() {
        let mut p = project_with_rows(1);
        p.sheet.push(SheetEntry::new(model::SegmentId(999)));
        assert_eq!(layout_sheet(&p, PartFilter::All).pages.len(), 1);
    }
}
