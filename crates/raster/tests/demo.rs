//! Renders the demo fixture to PNG (target/test-output/) for visual checks.

use model::{PartFilter, Project, StaffKind};

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
            let pixmap = raster::render(page, 6.0);
            assert_eq!(pixmap.width(), 1260);
            pixmap.save_png(out.join(format!("demo-{name}-{}.png", i + 1))).unwrap();
        }
    }
}
