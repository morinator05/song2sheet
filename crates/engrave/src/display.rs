//! Renderer-independent page description. Units are millimetres, origin is
//! the top-left page corner, y grows downwards. Text is positioned at its
//! baseline (left end).

use crate::fonts::{self, FontId};

pub const A4_WIDTH: f32 = 210.0;
pub const A4_HEIGHT: f32 = 297.0;

/// Assumed ascent and descent of text fonts, in em.
const TEXT_ASCENT: f32 = 0.75;
const TEXT_DESCENT: f32 = 0.25;

/// Gray level: 0 = black, 255 = white.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade(pub u8);

impl Shade {
    pub const BLACK: Shade = Shade(0);
    pub const WHITE: Shade = Shade(255);
    pub const LIGHT: Shade = Shade(180);
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Line { x1: f32, y1: f32, x2: f32, y2: f32, width: f32, shade: Shade },
    Rect { x: f32, y: f32, w: f32, h: f32, shade: Shade },
    Text { x: f32, y: f32, size: f32, font: FontId, text: String },
}

impl Item {
    pub fn translate(&mut self, dx: f32, dy: f32) {
        match self {
            Item::Line { x1, y1, x2, y2, .. } => {
                *x1 += dx;
                *x2 += dx;
                *y1 += dy;
                *y2 += dy;
            }
            Item::Rect { x, y, .. } | Item::Text { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
        }
    }

    /// Approximate bounding box. Music glyphs use their real outlines;
    /// other text assumes ascent 0.75 em and descent 0.25 em.
    pub fn bbox(&self) -> BBox {
        match self {
            Item::Line { x1, y1, x2, y2, width, .. } => {
                let h = width / 2.0;
                BBox {
                    x0: x1.min(*x2) - h,
                    y0: y1.min(*y2) - h,
                    x1: x1.max(*x2) + h,
                    y1: y1.max(*y2) + h,
                }
            }
            Item::Rect { x, y, w, h, .. } => BBox { x0: *x, y0: *y, x1: x + w, y1: y + h },
            Item::Text { x, y, size, font: FontId::Music, text } => {
                let mut pen = *x;
                let mut bbox: Option<BBox> = None;
                for c in text.chars() {
                    if let Some(g) = fonts::glyph_bbox(FontId::Music, c) {
                        // Glyph boxes are in em units with y up.
                        let b = BBox {
                            x0: pen + g.x0 * size,
                            y0: y - g.y1 * size,
                            x1: pen + g.x1 * size,
                            y1: y - g.y0 * size,
                        };
                        bbox = Some(bbox.map_or(b, |a| a.union(b)));
                    }
                    pen += fonts::text_width(FontId::Music, *size, &c.to_string());
                }
                bbox.unwrap_or(BBox { x0: *x, y0: *y, x1: *x, y1: *y })
            }
            Item::Text { x, y, size, font, text } => BBox {
                x0: *x,
                y0: y - TEXT_ASCENT * size,
                x1: x + fonts::text_width(*font, *size, text),
                y1: y + TEXT_DESCENT * size,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    pub width: f32,
    pub height: f32,
    pub items: Vec<Item>,
}

impl Page {
    pub fn a4() -> Self {
        Self { width: A4_WIDTH, height: A4_HEIGHT, items: Vec::new() }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DisplayList {
    pub pages: Vec<Page>,
}

/// Axis-aligned bounding box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl BBox {
    pub fn union(self, o: BBox) -> BBox {
        BBox {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }
}

/// Union bounding box of items; `None` if empty.
pub fn bbox_of(items: &[Item]) -> Option<BBox> {
    items.iter().map(Item::bbox).reduce(BBox::union)
}
