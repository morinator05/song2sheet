//! Rasterizes an [`engrave::Page`] with tiny-skia: on white paper for the
//! sheet preview (and visual checks in tests), or as a white ink mask that
//! the app tints to its text colour for notation shown on its own.

use engrave::{FontId, Item, Page, Shade, fonts};
use tiny_skia::{
    BlendMode, Color, FillRule, LineCap, Paint, Path, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

/// What the page is drawn on.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    /// Shades of grey on white paper.
    Paper,
    /// White ink, as opaque as the shade is dark, on a transparent
    /// background. Paper-white items (knockouts) erase the ink under them.
    Ink,
}

/// Renders `page` at `px_per_mm` onto a white pixmap.
pub fn render(page: &Page, px_per_mm: f32) -> Pixmap {
    draw_page(page, px_per_mm, Mode::Paper)
}

/// Renders `page` at `px_per_mm` as white ink on a transparent pixmap, to
/// be tinted when it is shown.
pub fn render_ink(page: &Page, px_per_mm: f32) -> Pixmap {
    draw_page(page, px_per_mm, Mode::Ink)
}

fn draw_page(page: &Page, px_per_mm: f32, mode: Mode) -> Pixmap {
    let w = (page.width * px_per_mm).ceil().max(1.0) as u32;
    let h = (page.height * px_per_mm).ceil().max(1.0) as u32;
    let mut pm = Pixmap::new(w, h).expect("non-zero pixmap size");
    if mode == Mode::Paper {
        pm.fill(Color::WHITE);
    }
    let base = Transform::from_scale(px_per_mm, px_per_mm);
    for item in &page.items {
        draw_item(&mut pm, item, base, mode);
    }
    pm
}

fn paint(shade: Shade, mode: Mode) -> Paint<'static> {
    let mut p = Paint::default();
    match mode {
        Mode::Paper => p.set_color_rgba8(shade.0, shade.0, shade.0, u8::MAX),
        Mode::Ink if shade == Shade::WHITE => p.blend_mode = BlendMode::Clear,
        Mode::Ink => p.set_color_rgba8(u8::MAX, u8::MAX, u8::MAX, u8::MAX - shade.0),
    }
    p.anti_alias = true;
    p
}

fn draw_item(pm: &mut Pixmap, item: &Item, base: Transform, mode: Mode) {
    match item {
        Item::Line { x1, y1, x2, y2, width, shade } => {
            let mut pb = PathBuilder::new();
            pb.move_to(*x1, *y1);
            pb.line_to(*x2, *y2);
            if let Some(path) = pb.finish() {
                let stroke = Stroke { width: *width, line_cap: LineCap::Butt, ..Stroke::default() };
                pm.stroke_path(&path, &paint(*shade, mode), &stroke, base, None);
            }
        }
        Item::Rect { x, y, w, h, shade } => {
            if let Some(r) = Rect::from_xywh(*x, *y, *w, *h) {
                pm.fill_rect(r, &paint(*shade, mode), base, None);
            }
        }
        Item::Text { x, y, size, font, text } => {
            draw_text(pm, (*x, *y), *size, *font, text, base, &paint(Shade::BLACK, mode))
        }
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

/// Draws `text` with its baseline starting at `at`.
fn draw_text(
    pm: &mut Pixmap,
    at: (f32, f32),
    size: f32,
    font: FontId,
    text: &str,
    base: Transform,
    paint: &Paint,
) {
    let upem = fonts::face(font).units_per_em() as f32;
    let scale = size / upem;
    let (mut pen, y) = at;
    for glyph in fonts::glyph_run(font, text) {
        if let Some(path) = glyph_path(font, glyph.id) {
            // Font units are y-up; flip and move to the pen position.
            let t = base.pre_concat(Transform::from_row(scale, 0.0, 0.0, -scale, pen, y));
            pm.fill_path(&path, paint, FillRule::Winding, t, None);
        }
        pen += glyph.advance * size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 10 × 10 mm page: black top half with a knockout in it.
    fn page() -> Page {
        let items = vec![
            Item::Rect { x: 0.0, y: 0.0, w: 10.0, h: 5.0, shade: Shade::BLACK },
            Item::Rect { x: 2.0, y: 1.0, w: 2.0, h: 2.0, shade: Shade::WHITE },
        ];
        Page { width: 10.0, height: 10.0, items }
    }

    #[test]
    fn ink_is_white_on_a_transparent_background() {
        let pm = render_ink(&page(), 1.0);
        let px = |x, y| pm.pixel(x, y).expect("inside the pixmap");
        assert_eq!((px(8, 2).red(), px(8, 2).alpha()), (u8::MAX, u8::MAX), "ink");
        assert_eq!(px(8, 8).alpha(), 0, "background");
        assert_eq!(px(3, 2).alpha(), 0, "the knockout erases the ink under it");
    }

    #[test]
    fn paper_is_white_with_black_ink() {
        let pm = render(&page(), 1.0);
        let px = |x, y| pm.pixel(x, y).expect("inside the pixmap");
        assert_eq!(px(8, 2).red(), 0, "ink");
        assert_eq!((px(8, 8).red(), px(8, 8).alpha()), (u8::MAX, u8::MAX), "paper");
        assert_eq!(px(3, 2).red(), u8::MAX, "knockout");
    }
}
