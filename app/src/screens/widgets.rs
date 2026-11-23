//! Widgets, sizes and labels shared by several screens.

use eframe::egui;

pub const ICON_ADD: &str = "+";
pub const ICON_REMOVE: &str = "−";
pub const ICON_EDIT: &str = "✏";
pub const ICON_DUPLICATE: &str = "⧉";
pub const ICON_DELETE: &str = "🗑";

/// Width of single-line name fields (sections, sheet row labels).
pub const NAME_FIELD_W: f32 = 140.0;
/// Width of short text fields (staff labels, chord symbols).
pub const SHORT_FIELD_W: f32 = 70.0;
/// Width of small combo boxes (part, time signature denominator).
pub const SMALL_COMBO_W: f32 = 80.0;
/// Vertical gap between blocks of a screen.
pub const BLOCK_GAP: f32 = 8.0;

/// Label written next to every tempo field.
pub const TEMPO_LABEL: &str = "Tempo";
pub const DEFAULT_BPM: u32 = 120;

/// Heading and toolbar label of the sheet screen and the section editor.
pub const SHEET_TITLE: &str = "Sheet";
pub const EDITOR_TITLE: &str = "Section editor";
pub const BPM_RANGE: std::ops::RangeInclusive<u32> = 20..=400;

/// Rendered images are made for widths rounded up to this many physical
/// pixels, so resizing the window doesn't re-render on every pixel.
const RENDER_WIDTH_STEP_PX: f32 = 100.0;
/// Smallest width (points) a rendered image is shown at.
const MIN_IMAGE_W: f32 = 100.0;

/// Width (points) to show a rendered image at, and the pixel width to
/// render it with.
pub fn image_widths(ui: &egui::Ui) -> (f32, f32) {
    let width = ui.available_width().max(MIN_IMAGE_W);
    let px = (width * ui.pixels_per_point() / RENDER_WIDTH_STEP_PX).ceil() * RENDER_WIDTH_STEP_PX;
    (width, px)
}

/// Checkbox + BPM field for an optional tempo.
pub fn optional_bpm(ui: &mut egui::Ui, tempo: &mut Option<u32>) {
    ui.horizontal(|ui| {
        let mut on = tempo.is_some();
        if ui.checkbox(&mut on, "").changed() {
            *tempo = on.then_some(DEFAULT_BPM);
        }
        if let Some(bpm) = tempo {
            // Don't rewrite out-of-range values from a file just by showing them.
            ui.add(
                egui::DragValue::new(bpm)
                    .range(BPM_RANGE)
                    .suffix(" bpm")
                    .clamp_existing_to_range(false),
            );
        }
    });
}

/// A modal yes/no question. Returns `Some(true)` when confirmed,
/// `Some(false)` when cancelled and `None` while still open.
pub fn confirm(
    ctx: &egui::Context,
    id: egui::Id,
    question: &str,
    confirm_label: &str,
) -> Option<bool> {
    let mut choice = None;
    egui::Modal::new(id).show(ctx, |ui| {
        ui.label(question);
        ui.horizontal(|ui| {
            if ui.button(confirm_label).clicked() {
                choice = Some(true);
            }
            if ui.button("Cancel").clicked() {
                choice = Some(false);
            }
        });
    });
    choice
}
