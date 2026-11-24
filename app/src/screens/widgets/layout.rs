//! Structure: panel headers, cards and captions.

use eframe::egui::{self, CornerRadius, Frame, InnerResponse, Label, RichText, Stroke};

use crate::theme::{self, metrics};

/// A panel or screen title with its actions aligned right. The actions
/// keep their size; a title too long for the rest is cut with "…" and
/// shown whole in its tooltip. Put captions under the header, not in it.
pub fn header<R>(ui: &mut egui::Ui, title: &str, actions: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let height = ui.spacing().interact_size.y;
    let size = egui::vec2(ui.available_width(), height);
    let r = ui
        .allocate_ui_with_layout(size, egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let r = actions(ui);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(Label::new(RichText::new(title).heading()).truncate());
            });
            r
        })
        .inner;
    ui.add_space(metrics::SPACE_XS);
    r
}

/// What a screen or list shows when it has nothing yet: a centred line,
/// optional buttons (`actions`), and a weak hint under them.
pub fn empty_state<R>(
    ui: &mut egui::Ui,
    text: &str,
    hint: Option<&str>,
    actions: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.add_space(metrics::BLOCK_GAP);
    ui.vertical_centered(|ui| {
        ui.label(text);
        ui.add_space(metrics::SPACE_S);
        let r = actions(ui);
        if let Some(hint) = hint {
            ui.add_space(metrics::SPACE_S);
            caption(ui, hint);
        }
        r
    })
    .inner
}

/// A group of related controls on a slightly lifted background.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
    let p = theme::palette(ui.ctx());
    card_frame(&p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui)
    })
}

/// `add`'s contents with clicks on their background reported (widgets
/// inside still get their own clicks), e.g. double-click to open.
pub fn clickable<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
    ui.scope_builder(egui::UiBuilder::new().sense(egui::Sense::click()), add)
}

/// [`card`]'s frame, e.g. to change its fill for a highlighted card.
pub fn card_frame(p: &theme::Palette) -> Frame {
    Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(metrics::HAIRLINE, p.border))
        .corner_radius(CornerRadius::same(metrics::CARD_RADIUS))
        .inner_margin(metrics::CARD_MARGIN)
}

/// [`card_frame`] of the item open in the editor: accent tint and stroke.
pub fn open_card_frame(p: &theme::Palette) -> Frame {
    card_frame(p)
        .fill(theme::mix(p.card, p.accent, theme::OPEN_TINT))
        .stroke(Stroke::new(metrics::HAIRLINE, p.accent))
}

/// Small text in the weak colour: hints, captions, counts.
pub fn caption(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let weak = theme::palette(ui.ctx()).weak_text;
    ui.label(RichText::new(text.into()).small().color(weak))
}

/// A short bold label naming the group of controls after it.
pub fn group_label(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let weak = theme::palette(ui.ctx()).weak_text;
    ui.add(Label::new(RichText::new(text).small().strong().color(weak)).extend())
}
