//! Rasterizes an [`engrave::Page`] with tiny-skia (used for the on-screen
//! preview and for visual checks in tests).

use engrave::{FontId, Item, Page, Shade, fonts};
use tiny_skia::{
    Color, FillRule, LineCap, Paint, Path, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

/// Renders `page` at `px_per_mm` onto a white pixmap.
pub fn render(page: &Page, px_per_mm: f32) -> Pixmap {
    let w = (page.width * px_per_mm).ceil().max(1.0) as u32;
    let h = (page.height * px_per_mm).ceil().max(1.0) as u32;
    let mut pm = Pixmap::new(w, h).expect("non-zero pixmap size");
    pm.fill(Color::WHITE);
    let base = Transform::from_scale(px_per_mm, px_per_mm);
    for item in &page.items {
        draw_item(&mut pm, item, base);
    }
    pm
}

fn paint(shade: Shade) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(shade.0, shade.0, shade.0, 255);
    p.anti_alias = true;
    p
}

fn draw_item(pm: &mut Pixmap, item: &Item, base: Transform) {
    match item {
        Item::Line { x1, y1, x2, y2, width, shade } => {
            let mut pb = PathBuilder::new();
            pb.move_to(*x1, *y1);
            pb.line_to(*x2, *y2);
            if let Some(path) = pb.finish() {
                let stroke = Stroke { width: *width, line_cap: LineCap::Butt, ..Stroke::default() };
                pm.stroke_path(&path, &paint(*shade), &stroke, base, None);
            }
        }
        Item::Rect { x, y, w, h, shade } => {
            if let Some(r) = Rect::from_xywh(*x, *y, *w, *h) {
                pm.fill_rect(r, &paint(*shade), base, None);
            }
        }
        Item::Text { x, y, size, font, text } => draw_text(pm, *x, *y, *size, *font, text, base),
    }
}

struct Outline(PathBuilder);

impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.0.close();
    }
}

fn glyph_path(font: FontId, gid: ttf_parser::GlyphId) -> Option<Path> {
    let mut b = Outline(PathBuilder::new());
    fonts::face(font).outline_glyph(gid, &mut b)?;
    b.0.finish()
}

fn draw_text(
    pm: &mut Pixmap,
    x: f32,
    y: f32,
    size: f32,
    font: FontId,
    text: &str,
    base: Transform,
) {
    let upem = fonts::face(font).units_per_em() as f32;
    let scale = size / upem;
    let paint = paint(Shade::BLACK);
    let mut pen = x;
    for glyph in fonts::glyph_run(font, text) {
        if let Some(path) = glyph_path(font, glyph.id) {
            // Font units are y-up; flip and move to the pen position.
            let t = base.pre_concat(Transform::from_row(scale, 0.0, 0.0, -scale, pen, y));
            pm.fill_path(&path, &paint, FillRule::Winding, t, None);
        }
        pen += glyph.advance * size;
    }
}
