//! Widgets, sizes and labels shared by several screens.

pub mod buttons;
pub mod icons;
pub mod layout;

use eframe::egui;

pub use crate::theme::metrics::BLOCK_GAP;
pub use buttons::{combo, danger, ghost, icon_button, menu, primary_enabled, secondary, toggle};
pub use icons::Icon;
pub use layout::{
    caption, card, card_frame, clickable, empty_state, group_label, header, open_card_frame,
};

pub use crate::theme::metrics::{NAME_FIELD_W, SHORT_FIELD_W};

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

/// Width (points) to show a rendered image at when `available` points are
/// free, and the pixel width to render it with.
pub fn image_widths(ui: &egui::Ui, available: f32) -> (f32, f32) {
    let width = available.max(MIN_IMAGE_W);
    let px = (width * ui.pixels_per_point() / RENDER_WIDTH_STEP_PX).ceil() * RENDER_WIDTH_STEP_PX;
    (width, px)
}

/// Checkbox (labelled `label`, may be empty) + BPM field for an optional
/// tempo.
pub fn optional_bpm(ui: &mut egui::Ui, label: &str, tempo: &mut Option<u32>) {
    ui.horizontal(|ui| {
        let mut on = tempo.is_some();
        if ui.checkbox(&mut on, label).on_hover_text("Print a tempo").changed() {
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

/// Tooltip of a control with a shortcut: "What it does (Shortcut)".
pub fn hint(what: &str, shortcut: impl std::fmt::Display) -> String {
    format!("{what} ({shortcut})")
}

/// Whether a dialog or a menu (popup, open combo box) is open; keys then
/// belong to it, not to the screen behind.
pub fn dialog_or_menu_open(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.top_modal_layer().is_some()) || egui::Popup::is_any_open(ctx)
}

/// Whether plain keys may drive a screen: no dialog, menu or text field
/// has them.
pub fn keys_free(ctx: &egui::Context) -> bool {
    !dialog_or_menu_open(ctx) && !ctx.text_edit_focused()
}

/// A modal yes/no question about discarding something; the confirm
/// button is red. Returns `Some(true)` when confirmed (only by clicking it:
/// a habitual Enter must not destroy work), `Some(false)` when cancelled
/// (button, Esc or a click outside) and `None` while still open.
pub fn confirm(
    ctx: &egui::Context,
    id: egui::Id,
    question: &str,
    confirm_label: &str,
) -> Option<bool> {
    let mut choice = None;
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
    });
    if choice.is_none() && modal.should_close() {
        choice = Some(false);
    }
    choice
}
