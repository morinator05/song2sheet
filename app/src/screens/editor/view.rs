//! The engraved section with the editing cursor drawn on top. Clicking a
//! beat selects it.

use eframe::egui::{
    self, Color32, ColorImage, Rect, Sense, TextureHandle, TextureOptions, pos2, vec2,
};
use engrave::BeatBox;
use model::Segment;

use super::cursor::Cursor;
use crate::screens::widgets::image_widths;

/// How far above/below the staff lines a click or the cursor reaches (mm).
const REACH: f32 = 3.0;
/// Opacity of the selected-beat highlight.
const CURSOR_ALPHA: f32 = 0.35;
const CURSOR_ROUNDING: f32 = 2.0;
/// Half height (mm) of the mark on the selected tab string.
const STRING_MARK_HALF_H: f32 = 0.6;
const STRING_MARK_COLOR: Color32 = Color32::from_rgba_premultiplied(110, 30, 30, 110);

#[derive(Default)]
pub struct NotationView {
    /// What the texture was rendered from: section and pixel width.
    rendered: Option<(Segment, f32)>,
    texture: Option<TextureHandle>,
    beats: Vec<BeatBox>,
    page_width: f32,
}

impl NotationView {
    /// Shows `seg` with `cursor`. Returns the beat the user clicked.
    pub fn show(&mut self, ui: &mut egui::Ui, seg: &Segment, cursor: &Cursor) -> Option<BeatBox> {
        let (width, px_width) = image_widths(ui);
        let fresh = self.rendered.as_ref().is_some_and(|(s, w)| s == seg && *w == px_width);
        if !fresh {
            self.render(ui.ctx(), seg, px_width);
        }
        let tex = self.texture.as_ref()?;
        let size = tex.size_vec2();
        let image = egui::Image::new((tex.id(), vec2(width, width * size.y / size.x)))
            .sense(Sense::click());
        let response = ui.add(image);
        let scale = width / self.page_width; // points per mm
        let origin = response.rect.min;
        let to_screen = |x0: f32, y0: f32, x1: f32, y1: f32| {
            Rect::from_min_max(origin + vec2(x0, y0) * scale, origin + vec2(x1, y1) * scale)
        };

        if let Some(b) = self.beats.iter().find(|b| is_at(b, cursor)) {
            let painter = ui.painter_at(response.rect);
            let highlight = ui.visuals().selection.bg_fill.gamma_multiply(CURSOR_ALPHA);
            painter.rect_filled(
                to_screen(b.x0, b.y0 - REACH, b.x1, b.y1 + REACH),
                CURSOR_ROUNDING,
                highlight,
            );
            if let Some(y) = string_y(seg, cursor, b) {
                let mark = to_screen(b.x0, y - STRING_MARK_HALF_H, b.x1, y + STRING_MARK_HALF_H);
                painter.rect_filled(mark, CURSOR_ROUNDING / 2.0, STRING_MARK_COLOR);
            }
        }

        let click = response.clicked().then(|| response.interact_pointer_pos()).flatten()?;
        // Typing should go to the editor again, not to a focused widget.
        if let Some(focused) = ui.memory(|m| m.focused()) {
            ui.memory_mut(|m| m.surrender_focus(focused));
        }
        let mm = (click - origin) / scale;
        hit(&self.beats, pos2(mm.x, mm.y), cursor.voice)
    }

    fn render(&mut self, ctx: &egui::Context, seg: &Segment, px_width: f32) {
        let view = engrave::layout_segment(seg);
        let pixmap = raster::render(&view.page, px_width / view.page.width);
        let size = [pixmap.width() as usize, pixmap.height() as usize];
        let image = ColorImage::from_rgba_premultiplied(size, pixmap.data());
        self.texture = Some(ctx.load_texture("section-editor", image, TextureOptions::LINEAR));
        self.beats = view.beats;
        self.page_width = view.page.width;
        self.rendered = Some((seg.clone(), px_width));
    }
}

/// The beat under `p` (mm). Where voices overlap, `voice` wins.
fn hit(beats: &[BeatBox], p: egui::Pos2, voice: usize) -> Option<BeatBox> {
    let under =
        |b: &&BeatBox| (b.x0..b.x1).contains(&p.x) && (b.y0 - REACH..b.y1 + REACH).contains(&p.y);
    let mut hits = beats.iter().filter(under);
    let first = hits.clone().next().copied();
    hits.find(|b| b.voice == voice).copied().or(first)
}

fn is_at(b: &BeatBox, c: &Cursor) -> bool {
    (b.staff, b.bar, b.voice, b.beat) == (c.staff, c.bar, c.voice, c.beat)
}

/// y (mm) of the selected tab string, if the staff is drawn as tab.
fn string_y(seg: &Segment, cursor: &Cursor, b: &BeatBox) -> Option<f32> {
    let staff = seg.staves.get(cursor.staff)?;
    if !staff.has_tab() {
        return None;
    }
    let gaps = staff.tuning().len().saturating_sub(1).max(1) as f32;
    Some(b.y0 + (cursor.string as f32 - 1.0) * (b.y1 - b.y0) / gaps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn beat(voice: usize, beat: usize, x0: f32, x1: f32) -> BeatBox {
        BeatBox { staff: 0, bar: 0, voice, beat, x0, x1, y0: 10.0, y1: 16.0 }
    }

    #[test]
    fn hit_finds_the_beat_under_the_pointer() {
        let beats = [beat(0, 0, 0.0, 5.0), beat(0, 1, 5.0, 10.0)];
        assert_eq!(hit(&beats, pos2(6.0, 12.0), 0).map(|b| b.beat), Some(1));
        assert_eq!(hit(&beats, pos2(6.0, 12.0 + 20.0), 0), None, "far below the staff");
    }

    #[test]
    fn overlapping_voices_prefer_the_cursor_voice() {
        let beats = [beat(0, 0, 0.0, 10.0), beat(1, 0, 0.0, 10.0)];
        assert_eq!(hit(&beats, pos2(1.0, 12.0), 1).map(|b| b.voice), Some(1));
        assert_eq!(hit(&beats, pos2(1.0, 12.0), 0).map(|b| b.voice), Some(0));
    }
}
