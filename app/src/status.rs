//! The message line at the bottom of the window: the outcome of the last
//! action. Confirmations and warnings fade out; errors stay until
//! replaced. Screens [`post`] messages; the app picks them up each frame.

use eframe::egui;

use crate::screens::widgets::{Icon, icons};
use crate::theme;

/// Seconds a confirmation stays visible.
const SHOW_SECS: f64 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Success,
    /// Something was refused or changed more than asked; fades like a
    /// confirmation.
    Warning,
    Error,
}

/// Leaves a message for the status line (from any screen). Of several
/// messages in one frame the most serious one is shown (the last of equals).
pub fn post(ctx: &egui::Context, kind: Kind, text: impl Into<String>) {
    let text = text.into();
    ctx.data_mut(|d| {
        let slot = d.get_temp_mut_or_default::<Option<(String, Kind)>>(mailbox());
        if slot.as_ref().is_none_or(|(_, k)| severity(kind) >= severity(*k)) {
            *slot = Some((text, kind));
        }
    });
}

fn severity(kind: Kind) -> u8 {
    match kind {
        Kind::Success => 0,
        Kind::Warning => 1,
        Kind::Error => 2,
    }
}

fn mailbox() -> egui::Id {
    egui::Id::unique("status_mailbox")
}

#[derive(Clone, Debug, Default)]
pub struct Status {
    message: Option<(String, Kind, f64)>,
}

impl Status {
    pub fn success(&mut self, ctx: &egui::Context, text: impl Into<String>) {
        self.set(ctx, text.into(), Kind::Success);
    }

    pub fn warn(&mut self, ctx: &egui::Context, text: impl Into<String>) {
        self.set(ctx, text.into(), Kind::Warning);
    }

    pub fn error(&mut self, ctx: &egui::Context, text: impl Into<String>) {
        self.set(ctx, text.into(), Kind::Error);
    }

    /// Shows `result`: its message as a confirmation or an error.
    pub fn report(&mut self, ctx: &egui::Context, result: Result<String, String>) {
        match result {
            Ok(m) => self.success(ctx, m),
            Err(e) => self.error(ctx, e),
        }
    }

    /// Takes over a message a screen [`post`]ed.
    pub fn collect(&mut self, ctx: &egui::Context) {
        let posted = ctx.data_mut(|d| d.remove_temp::<Option<(String, Kind)>>(mailbox())).flatten();
        if let Some((text, kind)) = posted {
            self.set(ctx, text, kind);
        }
    }

    pub fn clear(&mut self) {
        self.message = None;
    }

    fn set(&mut self, ctx: &egui::Context, text: String, kind: Kind) {
        self.message = Some((text, kind, ctx.input(|i| i.time)));
    }

    /// The visible message, if any.
    fn current(&self, now: f64) -> Option<(&str, Kind)> {
        let (text, kind, at) = self.message.as_ref()?;
        let fresh = *kind == Kind::Error || now - at < SHOW_SECS;
        fresh.then_some((text.as_str(), *kind))
    }

    /// Draws the message (or nothing) and schedules its fading.
    pub fn show(&self, ui: &mut egui::Ui) {
        let now = ui.input(|i| i.time);
        let Some((text, kind)) = self.current(now) else { return };
        let p = theme::palette(ui.ctx());
        let (icon, color) = match kind {
            Kind::Success => (Icon::Success, p.success),
            Kind::Warning => (Icon::Warning, p.warn),
            Kind::Error => (Icon::Error, p.error),
        };
        icons::label(ui, icon, text, color);
        if kind != Kind::Error {
            ui.ctx().request_repaint_after_secs(SHOW_SECS as f32);
        }
    }
}

/// The file name of `path`, for messages (the full path goes in tooltips).
pub fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmations_fade_but_errors_stay() {
        let ok = Status { message: Some(("Saved".into(), Kind::Success, 0.0)) };
        assert!(ok.current(1.0).is_some());
        assert!(ok.current(SHOW_SECS + 1.0).is_none());
        let err = Status { message: Some(("Failed".into(), Kind::Error, 0.0)) };
        assert!(err.current(SHOW_SECS * 10.0).is_some());
    }
}
