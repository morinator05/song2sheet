//! The button hierarchy of `docs/DESIGN.md`: primary, secondary, ghost,
//! danger and toggle. Button text never wraps or truncates.

use eframe::egui::{self, Button, Response, RichText, TextStyle, TextWrapMode};

use crate::theme;

/// The one main action of an area, filled with the accent colour.
pub fn primary(ui: &mut egui::Ui, text: &str) -> Response {
    let p = theme::palette(ui.ctx());
    let button = Button::new(RichText::new(text).color(p.on_accent)).fill(p.accent);
    ui.add(button.wrap_mode(TextWrapMode::Extend))
}

/// [`primary`] that can be disabled; `why` explains a disabled button.
pub fn primary_enabled(ui: &mut egui::Ui, enabled: bool, text: &str, why: &str) -> Response {
    let r = ui.add_enabled_ui(enabled, |ui| primary(ui, text)).inner;
    if enabled { r } else { r.on_disabled_hover_text(why) }
}

/// An ordinary button.
pub fn secondary(ui: &mut egui::Ui, text: &str) -> Response {
    ui.add(Button::new(text).wrap_mode(TextWrapMode::Extend))
}

/// A frameless button for row tools; `tooltip` says what it does.
pub fn ghost(ui: &mut egui::Ui, text: &str, tooltip: &str) -> Response {
    let button = Button::new(text).frame_when_inactive(false).wrap_mode(TextWrapMode::Extend);
    ui.add(button).on_hover_text(tooltip)
}

/// A [`ghost`] button for destructive actions: turns red under the pointer.
pub fn danger(ui: &mut egui::Ui, text: &str, tooltip: &str) -> Response {
    let error = theme::palette(ui.ctx()).error;
    ui.scope(|ui| {
        let w = &mut ui.visuals_mut().widgets;
        w.hovered.fg_stroke.color = error;
        w.active.fg_stroke.color = error;
        w.hovered.bg_stroke.color = error;
        w.active.bg_stroke.color = error;
        ghost(ui, text, tooltip)
    })
    .inner
}

/// Text on a red button.
const ON_DANGER: egui::Color32 = egui::Color32::WHITE;

/// A red button confirming that something is destroyed.
pub fn danger_filled(ui: &mut egui::Ui, text: &str) -> Response {
    let error = theme::palette(ui.ctx()).error;
    let button = Button::new(RichText::new(text).color(ON_DANGER)).fill(error);
    ui.add(button.wrap_mode(TextWrapMode::Extend))
}

/// A button that shows an on/off state in the accent colour.
pub fn toggle(ui: &mut egui::Ui, on: bool, text: &str) -> Response {
    ui.add(Button::new(text).selected(on).wrap_mode(TextWrapMode::Extend))
}

/// Width a combo box needs so that none of `options` is cut off.
pub fn combo_width<'a>(ui: &egui::Ui, options: impl IntoIterator<Item = &'a str>) -> f32 {
    let font = TextStyle::Button.resolve(ui.style());
    let color = ui.visuals().text_color();
    let widest = ui.ctx().fonts_mut(|f| {
        options
            .into_iter()
            .map(|o| f.layout_no_wrap(o.to_string(), font.clone(), color).size().x)
            .fold(0.0, f32::max)
    });
    let s = &ui.style().spacing;
    widest + s.icon_width + s.icon_spacing + 2.0 * s.button_padding.x
}

/// A combo box sized for its longest option.
pub fn combo<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: impl egui::AsIdSalt,
    value: &mut T,
    options: &[(T, impl AsRef<str>)],
) -> Response {
    let selected = options.iter().find(|(v, _)| v == value).map_or("", |(_, n)| n.as_ref());
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected)
        .width(combo_width(ui, options.iter().map(|(_, n)| n.as_ref())))
        .wrap_mode(TextWrapMode::Extend)
        .show_ui(ui, |ui| {
            for (v, name) in options {
                ui.selectable_value(value, *v, name.as_ref());
            }
        })
        .response
}
