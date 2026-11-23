use model::{PartFilter, Project};

#[test]
fn exports_demo_as_a4_pdf() {
    let p = Project::from_json(include_str!("../../../tests/fixtures/demo.s2s.json")).unwrap();
    let list = engrave::layout_sheet(&p, PartFilter::All);
    let bytes = pdf::export(&list, &p.meta.title).unwrap();
    assert!(bytes.starts_with(b"%PDF"));

    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-output");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("demo.pdf"), bytes).unwrap();
}
