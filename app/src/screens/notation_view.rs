//! An engraved section drawn straight onto the card it sits in: ink in the
//! text colour, staff names in a gutter on the left, a hairline between
//! systems and a highlight under the ink (the editor's cursor or the
//! picker's bar selection). Clicking reports the beat under the pointer.

use std::ops::Range;

use eframe::egui::{
    self, Align2, Color32, ColorImage, FontId, Rect, Sense, Stroke, StrokeKind, TextStyle,
    TextureHandle, TextureOptions, pos2, vec2,
};
use engrave::{BeatBox, StaffLabel, SystemSpan};
use model::Segment;

use super::editor::Cursor;
use super::widgets::image_widths;
use crate::theme::{
    self,
    metrics::{
        HAIRLINE, NOTATION_HIGHLIGHT_ALPHA as HIGHLIGHT_ALPHA,
        NOTATION_HIGHLIGHT_RADIUS as HIGHLIGHT_ROUNDING, SPACE_S,
    },
};

/// How far above/below the staff lines a click or highlight reaches (mm).
const REACH: f32 = 3.0;
/// Half height (mm) of the solid accent mark on the selected tab string.
const STRING_MARK_HALF_H: f32 = 0.6;

/// What to highlight.
#[derive(Clone, Copy, Debug)]
pub enum Highlight<'a> {
    /// The editor's selected beat (and tab string).
    Cursor(&'a Cursor),
    /// Whole bars, across all staves.
    Bars(&'a Range<usize>),
}

#[derive(Default)]
pub struct NotationView {
    /// What the texture was rendered from: section and pixel width.
    rendered: Option<(Segment, f32)>,
    /// The ink as a white mask, tinted when drawn.
    texture: Option<TextureHandle>,
    beats: Vec<BeatBox>,
    labels: Vec<StaffLabel>,
    systems: Vec<SystemSpan>,
    page_width: f32,
    /// Screen area of the last drawn highlight.
    highlight: Option<Rect>,
}

/// An area of the page, in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Area {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl NotationView {
    /// Shows `seg` with `highlight`. Returns the beat the user clicked.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        seg: &Segment,
        highlight: Highlight,
    ) -> Option<BeatBox> {
        let font = TextStyle::Small.resolve(ui.style());
        let gutter = gutter_width(ui, seg, &font);
        let (width, px_width) = image_widths(ui, ui.available_width() - gutter);
        let fresh = self.rendered.as_ref().is_some_and(|(s, w)| s == seg && *w == px_width);
        if !fresh {
            self.render(ui.ctx(), seg, px_width);
        }
        let tex = self.texture.as_ref()?;
        let size = tex.size_vec2();
        let image_size = vec2(width, width * size.y / size.x);
        let (rect, response) =
            ui.allocate_exact_size(vec2(gutter + width, image_size.y), Sense::click());
        let image_rect = Rect::from_min_size(rect.min + vec2(gutter, 0.0), image_size);
        let scale = width / self.page_width; // points per mm
        let origin = image_rect.min;
        let to_screen = |a: Area| {
            Rect::from_min_max(origin + vec2(a.x0, a.y0) * scale, origin + vec2(a.x1, a.y1) * scale)
        };

        let painter = ui.painter_at(rect);
        let palette = theme::palette(ui.ctx());
        // The highlight goes under the ink, so the notes stay readable: an
        // accent tint with an accent outline, like the section open in the
        // editor elsewhere.
        self.highlight = None;
        let fill = palette.accent.gamma_multiply(HIGHLIGHT_ALPHA);
        let outline = Stroke::new(HAIRLINE, palette.accent);
        for a in highlight_areas(&self.beats, &self.systems, highlight) {
            // Clipped to the image: past the outer staves there may be no
            // page left for the full reach.
            let r = to_screen(a).intersect(image_rect);
            painter.rect(r, HIGHLIGHT_ROUNDING, fill, outline, StrokeKind::Inside);
            self.highlight = Some(self.highlight.map_or(r, |h| h.union(r)));
        }
        if let Some(mark) = self.string_mark(seg, highlight) {
            painter.rect_filled(to_screen(mark), HIGHLIGHT_ROUNDING / 2.0, palette.accent);
        }
        egui::Image::new((tex.id(), image_size)).tint(palette.text).paint_at(ui, image_rect);
        for pair in self.systems.windows(2) {
            let y = origin.y + (pair[0].bottom + pair[1].top) / 2.0 * scale;
            painter.hline(rect.x_range(), y, Stroke::new(HAIRLINE, palette.border));
        }
        let cursor_staff = if let Highlight::Cursor(c) = highlight { Some(c.staff) } else { None };
        for label in &self.labels {
            let color =
                if Some(label.staff) == cursor_staff { palette.text } else { palette.weak_text };
            let pos = pos2(image_rect.left() - SPACE_S, origin.y + label.y * scale);
            painter.text(pos, Align2::RIGHT_CENTER, &label.text, font.clone(), color);
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

    /// Where the highlight was last drawn, to scroll it into view.
    pub fn highlight_rect(&self) -> Option<Rect> {
        self.highlight
    }

    fn render(&mut self, ctx: &egui::Context, seg: &Segment, px_width: f32) {
        let view = engrave::layout_segment(seg);
        let pixmap = raster::render_ink(&view.page, px_width / view.page.width);
        let size = [pixmap.width() as usize, pixmap.height() as usize];
        let image = ColorImage::from_rgba_premultiplied(size, pixmap.data());
        self.texture = Some(ctx.load_texture("notation-view", image, TextureOptions::LINEAR));
        self.beats = view.beats;
        self.labels = view.staff_labels;
        self.systems = view.systems;
        self.page_width = view.page.width;
        self.rendered = Some((seg.clone(), px_width));
    }

    /// The mark on the cursor's tab string, if its staff is drawn as tab.
    fn string_mark(&self, seg: &Segment, highlight: Highlight) -> Option<Area> {
        let Highlight::Cursor(cursor) = highlight else { return None };
        let b = self.beats.iter().find(|b| is_at(b, cursor))?;
        let y = string_y(seg, cursor, b)?;
        Some(Area { x0: b.x0, y0: y - STRING_MARK_HALF_H, x1: b.x1, y1: y + STRING_MARK_HALF_H })
    }
}

/// Width of the staff names left of the notation: none for a single staff,
/// as on the sheet.
fn gutter_width(ui: &egui::Ui, seg: &Segment, font: &FontId) -> f32 {
    if !engrave::has_staff_labels(seg.staves.len()) {
        return 0.0;
    }
    let measure = |text: &str| {
        ui.painter().layout_no_wrap(text.to_string(), font.clone(), Color32::PLACEHOLDER).size().x
    };
    seg.staves.iter().map(|s| measure(s.display_label())).fold(0.0, f32::max) + SPACE_S
}

/// Areas to fill: the cursor's beat, or one column per system over the
/// selected bars of all its staves.
fn highlight_areas(beats: &[BeatBox], systems: &[SystemSpan], highlight: Highlight) -> Vec<Area> {
    match highlight {
        Highlight::Cursor(c) => beats
            .iter()
            .filter(|b| is_at(b, c))
            .map(|b| Area { x0: b.x0, y0: b.y0 - REACH, x1: b.x1, y1: b.y1 + REACH })
            .collect(),
        Highlight::Bars(bars) => systems
            .iter()
            .filter_map(|s| {
                let picked =
                    beats.iter().filter(|b| bars.contains(&b.bar) && s.bars.contains(&b.bar));
                let (x0, x1) = picked.fold(None, |span: Option<(f32, f32)>, b| {
                    Some(span.map_or((b.x0, b.x1), |(x0, x1)| (x0.min(b.x0), x1.max(b.x1))))
                })?;
                Some(Area { x0, y0: s.y0 - REACH, x1, y1: s.y1 + REACH })
            })
            .collect(),
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
    fn bar_highlight_is_one_column_per_system() {
        // Two staves; bars 0 and 1 on the first system, bar 2 on the second.
        // Each bar is one 10 mm beat starting at `x0`.
        let at = |staff, bar, x0: f32, y0: f32| BeatBox {
            staff,
            bar,
            y0,
            y1: y0 + 6.0,
            ..beat(0, 0, x0, x0 + 10.0)
        };
        let beats = [
            at(0, 0, 0.0, 10.0),
            at(1, 0, 0.0, 20.0),
            at(0, 1, 10.0, 10.0),
            at(1, 1, 10.0, 20.0),
            at(0, 2, 0.0, 40.0),
            at(1, 2, 0.0, 50.0),
        ];
        let systems = [
            SystemSpan { bars: 0..2, top: 8.0, y0: 10.0, y1: 26.0, bottom: 28.0 },
            SystemSpan { bars: 2..3, top: 38.0, y0: 40.0, y1: 56.0, bottom: 58.0 },
        ];
        let areas = highlight_areas(&beats, &systems, Highlight::Bars(&(1..3)));
        assert_eq!(
            areas,
            [
                Area { x0: 10.0, y0: 10.0 - REACH, x1: 20.0, y1: 26.0 + REACH },
                Area { x0: 0.0, y0: 40.0 - REACH, x1: 10.0, y1: 56.0 + REACH },
            ]
        );
    }
}
