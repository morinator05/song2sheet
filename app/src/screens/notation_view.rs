//! An engraved section with a highlight on top (the editor's cursor or the
//! picker's bar selection). Clicking reports the beat under the pointer.

use std::ops::Range;

use eframe::egui::{self, ColorImage, Rect, Sense, TextureHandle, TextureOptions, pos2, vec2};
use engrave::BeatBox;
use model::Segment;

use super::editor::Cursor;
use super::widgets::image_widths;

/// How far above/below the staff lines a click or highlight reaches (mm).
const REACH: f32 = 3.0;
/// Opacity of the highlight.
const HIGHLIGHT_ALPHA: f32 = 0.35;
const HIGHLIGHT_ROUNDING: f32 = 2.0;
/// Half height (mm) of the mark on the selected tab string.
const STRING_MARK_HALF_H: f32 = 0.6;
/// Opacity of the string mark (stronger than the beat highlight).
const STRING_MARK_ALPHA: f32 = 0.8;

/// What to highlight.
#[derive(Clone, Copy, Debug)]
pub enum Highlight<'a> {
    /// The editor's selected beat (and tab string).
    Cursor(&'a Cursor),
    /// Whole bars, on every staff.
    Bars(&'a Range<usize>),
}

#[derive(Default)]
pub struct NotationView {
    /// What the texture was rendered from: section and pixel width.
    rendered: Option<(Segment, f32)>,
    texture: Option<TextureHandle>,
    beats: Vec<BeatBox>,
    page_width: f32,
}

impl NotationView {
    /// Shows `seg` with `highlight`. Returns the beat the user clicked.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        seg: &Segment,
        highlight: Highlight,
    ) -> Option<BeatBox> {
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

        let painter = ui.painter_at(response.rect);
        let accent = crate::theme::palette(ui.ctx()).accent;
        let fill = accent.gamma_multiply(HIGHLIGHT_ALPHA);
        for a in highlight_areas(&self.beats, highlight) {
            painter.rect_filled(
                to_screen(a.x0, a.y0 - REACH, a.x1, a.y1 + REACH),
                HIGHLIGHT_ROUNDING,
                fill,
            );
        }
        if let Highlight::Cursor(cursor) = highlight
            && let Some(b) = self.beats.iter().find(|b| is_at(b, cursor))
            && let Some(y) = string_y(seg, cursor, b)
        {
            let mark = to_screen(b.x0, y - STRING_MARK_HALF_H, b.x1, y + STRING_MARK_HALF_H);
            painter.rect_filled(
                mark,
                HIGHLIGHT_ROUNDING / 2.0,
                accent.gamma_multiply(STRING_MARK_ALPHA),
            );
        }

        let click = response.clicked().then(|| response.interact_pointer_pos()).flatten()?;
        // Typing should go to the notation again, not to a focused widget.
        if let Some(focused) = ui.memory(|m| m.focused()) {
            ui.memory_mut(|m| m.surrender_focus(focused));
        }
        let mm = (click - origin) / scale;
        let voice = if let Highlight::Cursor(c) = highlight { c.voice } else { 0 };
        hit(&self.beats, pos2(mm.x, mm.y), voice)
    }

    fn render(&mut self, ctx: &egui::Context, seg: &Segment, px_width: f32) {
        let view = engrave::layout_segment(seg);
        let pixmap = raster::render(&view.page, px_width / view.page.width);
        let size = [pixmap.width() as usize, pixmap.height() as usize];
        let image = ColorImage::from_rgba_premultiplied(size, pixmap.data());
        self.texture = Some(ctx.load_texture("notation-view", image, TextureOptions::LINEAR));
        self.beats = view.beats;
        self.page_width = view.page.width;
        self.rendered = Some((seg.clone(), px_width));
    }
}

/// Areas to fill: the cursor's beat, or one box per selected bar and staff.
fn highlight_areas(beats: &[BeatBox], highlight: Highlight) -> Vec<BeatBox> {
    match highlight {
        Highlight::Cursor(c) => beats.iter().filter(|b| is_at(b, c)).copied().collect(),
        Highlight::Bars(bars) => {
            let mut areas: Vec<BeatBox> = Vec::new();
            for b in beats.iter().filter(|b| bars.contains(&b.bar)) {
                match areas.iter_mut().find(|a| a.bar == b.bar && a.staff == b.staff) {
                    Some(a) => {
                        a.x0 = a.x0.min(b.x0);
                        a.x1 = a.x1.max(b.x1);
                    }
                    None => areas.push(*b),
                }
            }
            areas
        }
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

    #[test]
    fn bar_highlight_covers_each_selected_bar_once_per_staff() {
        let beats = [
            beat(0, 0, 0.0, 5.0),
            beat(0, 1, 5.0, 10.0),
            beat(1, 0, 0.0, 10.0),
            BeatBox { bar: 1, ..beat(0, 0, 10.0, 20.0) },
        ];
        let areas = highlight_areas(&beats, Highlight::Bars(&(0..1)));
        assert_eq!(areas.len(), 1);
        assert_eq!((areas[0].x0, areas[0].x1), (0.0, 10.0));
    }
}
