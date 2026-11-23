//! Writes an [`engrave::DisplayList`] as a PDF with embedded fonts.

use engrave::{DisplayList, FontId, Item, Shade};
use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Rect};
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LineCap, Stroke};
use krilla::surface::Surface;
use krilla::text::{Font, GlyphId, KrillaGlyph};

/// PDF points per millimetre.
const PT_PER_MM: f32 = 72.0 / 25.4;

#[derive(Debug)]
pub struct ExportError(String);

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PDF export failed: {}", self.0)
    }
}

impl std::error::Error for ExportError {}

/// Renders all pages of `list` into PDF bytes. `title` goes into the metadata.
pub fn export(list: &DisplayList, title: &str) -> Result<Vec<u8>, ExportError> {
    let fonts = load_fonts()?;
    let mut doc = Document::new();
    doc.set_metadata(Metadata::new().title(title.to_string()).creator("song2sheet".into()));
    for page in &list.pages {
        let settings = PageSettings::from_wh(page.width * PT_PER_MM, page.height * PT_PER_MM)
            .ok_or_else(|| ExportError("invalid page size".into()))?;
        let mut pdf_page = doc.start_page_with(settings);
        let mut surface = pdf_page.surface();
        for item in &page.items {
            draw_item(&mut surface, item, &fonts);
        }
        surface.finish();
        pdf_page.finish();
    }
    doc.finish().map_err(|e| ExportError(format!("{e:?}")))
}

/// Same glyphs and advances as the engraver and the preview use.
fn krilla_glyphs(font: FontId, text: &str) -> Vec<KrillaGlyph> {
    engrave::fonts::glyph_run(font, text)
        .into_iter()
        .map(|g| {
            KrillaGlyph::new(
                GlyphId::new(g.id.0 as u32),
                g.advance,
                0.0,
                0.0,
                0.0,
                g.text_range,
                None,
            )
        })
        .collect()
}

/// One krilla font per [`FontId`], indexed by [`FontId::index`].
fn load_fonts() -> Result<Vec<Font>, ExportError> {
    FontId::ALL
        .iter()
        .map(|f| {
            Font::new(f.data().into(), 0)
                .ok_or_else(|| ExportError(format!("cannot load font {f:?}")))
        })
        .collect()
}

fn color(shade: Shade) -> rgb::Color {
    rgb::Color::new(shade.0, shade.0, shade.0)
}

fn fill(shade: Shade) -> Fill {
    Fill { paint: color(shade).into(), opacity: NormalizedF32::ONE, rule: FillRule::NonZero }
}

fn draw_item(surface: &mut Surface, item: &Item, fonts: &[Font]) {
    let mm = |v: f32| v * PT_PER_MM;
    match item {
        Item::Line { x1, y1, x2, y2, width, shade } => {
            let mut pb = PathBuilder::new();
            pb.move_to(mm(*x1), mm(*y1));
            pb.line_to(mm(*x2), mm(*y2));
            if let Some(path) = pb.finish() {
                surface.set_fill(None);
                surface.set_stroke(Some(Stroke {
                    paint: color(*shade).into(),
                    width: mm(*width),
                    line_cap: LineCap::Butt,
                    ..Stroke::default()
                }));
                surface.draw_path(&path);
                surface.set_stroke(None);
            }
        }
        Item::Rect { x, y, w, h, shade } => {
            let mut pb = PathBuilder::new();
            if let Some(rect) = Rect::from_xywh(mm(*x), mm(*y), mm(*w), mm(*h)) {
                pb.push_rect(rect);
            }
            if let Some(path) = pb.finish() {
                surface.set_fill(Some(fill(*shade)));
                surface.draw_path(&path);
            }
        }
        Item::Text { x, y, size, font, text } => {
            surface.set_fill(Some(fill(Shade::BLACK)));
            surface.draw_glyphs(
                Point::from_xy(mm(*x), mm(*y)),
                &krilla_glyphs(*font, text),
                fonts[font.index()].clone(),
                text,
                mm(*size),
                false,
            );
        }
    }
}
