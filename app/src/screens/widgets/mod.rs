//! Widgets, sizes and labels shared by several screens.

pub mod buttons;
pub mod icons;
pub mod layout;

use eframe::egui::{self, Key};

pub use crate::theme::metrics::BLOCK_GAP;
pub use buttons::{
    combo, danger, ghost, icon_button, menu, primary, primary_enabled, secondary, toggle,
};
pub use icons::Icon;
pub use layout::{caption, card, card_frame, clickable, group_label, header};

/// Width of single-line name fields (sections, sheet row labels).
pub const NAME_FIELD_W: f32 = 160.0;
/// Width of short text fields (staff labels, chord symbols).
pub const SHORT_FIELD_W: f32 = 70.0;

/// Placeholder of section name fields.
pub const SECTION_NAME_HINT: &str = "e.g. Verse";

/// The instruments as combo box options.
pub fn staff_kind_options() -> Vec<(model::StaffKind, &'static str)> {
    model::StaffKind::ALL.iter().map(|k| (*k, k.name())).collect()
}

/// "1 row", "3 rows": `n` with `noun`, plural with an "s".
pub fn count(n: usize, noun: &str) -> String {
    if n == 1 { format!("1 {noun}") } else { format!("{n} {noun}s") }
}

/// Label written next to every tempo field.
pub const TEMPO_LABEL: &str = "Tempo";
pub const DEFAULT_BPM: u32 = 120;

/// Tab labels and headings of the three centre screens.
pub const SHEET_TITLE: &str = "Sheet";
pub const EDITOR_TITLE: &str = "Section editor";
pub const PICKER_TITLE: &str = "Import";
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
        if ui.checkbox(&mut on, "").on_hover_text("Show a tempo").changed() {
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

/// A modal yes/no question about discarding something; the confirm
/// button is red. Returns `Some(true)` when confirmed (button or
/// Enter), `Some(false)` when cancelled (button, Esc or a click outside)
/// and `None` while still open.
pub fn confirm(
    ctx: &egui::Context,
    id: egui::Id,
    question: &str,
    confirm_label: &str,
) -> Option<bool> {
    let mut choice = None;
    // Enter counts only once the dialog was already open: the key press
    // that opened it must not also confirm it.
    let seen = id.with("seen");
    let was_open = ctx.data(|d| d.get_temp::<bool>(seen)).unwrap_or(false);
    ctx.data_mut(|d| d.insert_temp(seen, true));
    let modal = egui::Modal::new(id).show(ctx, |ui| {
        ui.label(question);
        ui.add_space(BLOCK_GAP);
        ui.horizontal(|ui| {
            if buttons::danger_filled(ui, confirm_label).clicked() {
                choice = Some(true);
            }
            if secondary(ui, "Cancel").clicked() {
                choice = Some(false);
            }
        });
        if was_open && ui.input(|i| i.key_pressed(Key::Enter)) {
            choice = Some(true);
        }
    });
    if choice.is_none() && modal.should_close() {
        choice = Some(false);
    }
    if choice.is_some() {
        ctx.data_mut(|d| d.remove::<bool>(seen));
    }
    choice
}
