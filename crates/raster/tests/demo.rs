//! Renders the demo fixture to PNG (target/test-output/) for visual checks.

use model::{PartFilter, Project, StaffKind};
use tiny_skia::{Color, Pixmap, PixmapPaint, Transform};

/// Grey level of the card the app draws section views on (dark theme).
const CARD_GREY: u8 = 48;
const PX_PER_MM: f32 = 6.0;

fn demo() -> Project {
    Project::from_json(include_str!("../../../tests/fixtures/demo.s2s.json")).unwrap()
}

#[test]
fn render_demo_pages() {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-output");
    std::fs::create_dir_all(&out).unwrap();
    let p = demo();
    for (name, filter) in [("all", PartFilter::All), ("drums", PartFilter::Only(StaffKind::Drums))]
    {
        let list = engrave::layout_sheet(&p, filter);
        assert!(!list.pages.is_empty());
        for (i, page) in list.pages.iter().enumerate() {
            let pixmap = raster::render(page, PX_PER_MM);
            assert_eq!(pixmap.width(), 1260);
            pixmap.save_png(out.join(format!("demo-{name}-{}.png", i + 1))).unwrap();
        }
    }
}

#[test]
fn render_section_editor_view() {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-output");
    std::fs::create_dir_all(&out).unwrap();
    let p = demo();
    let verse = p.segments.iter().find(|s| s.name == "Verse").unwrap();
    let view = engrave::layout_segment(verse);
    assert!(!view.beats.is_empty());
    on_card(&view.page).save_png(out.join("demo-section-verse.png")).unwrap();
}

#[test]
fn render_imported_musicxml() {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-output");
    std::fs::create_dir_all(&out).unwrap();
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/groove.musicxml");
    let score = import::load(&fixture).unwrap();
    let seg = import::extract(&score, 0..4, &[0, 1, 2], "Imported");
    let view = engrave::layout_segment(&seg);
    on_card(&view.page).save_png(out.join("import-groove.png")).unwrap();
}

/// A section view as the app shows it: the ink (untinted, so white) on a
/// dark card.
fn on_card(page: &engrave::Page) -> Pixmap {
    let ink = raster::render_ink(page, PX_PER_MM);
    let mut card = Pixmap::new(ink.width(), ink.height()).unwrap();
    card.fill(Color::from_rgba8(CARD_GREY, CARD_GREY, CARD_GREY, u8::MAX));
    card.draw_pixmap(0, 0, ink.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
    card
}
