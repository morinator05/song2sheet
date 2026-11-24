//! Structure: panel headers, cards and captions.

use eframe::egui::{self, CornerRadius, Frame, InnerResponse, Label, RichText, Stroke};

use crate::theme::{self, metrics};

/// A panel or screen title with its actions aligned right.
pub fn header<R>(ui: &mut egui::Ui, title: &str, actions: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let r = ui
        .horizontal(|ui| {
            ui.add(Label::new(RichText::new(title).heading()).extend());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), actions).inner
        })
        .inner;
    ui.add_space(metrics::SPACE_XS);
    r
}

/// A group of related controls on a slightly lifted background.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
    let p = theme::palette(ui.ctx());
    card_frame(&p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui)
    })
}

/// [`card`]'s frame, e.g. to change its fill for a highlighted card.
pub fn card_frame(p: &theme::Palette) -> Frame {
    Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(metrics::HAIRLINE, p.border))
        .corner_radius(CornerRadius::same(metrics::CARD_RADIUS))
        .inner_margin(metrics::CARD_MARGIN)
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
