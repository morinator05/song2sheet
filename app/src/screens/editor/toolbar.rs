//! Clickable equivalents of the editor's keyboard shortcuts, showing the
//! selected beat's duration. Hints come from the key tables in `keys`.

use eframe::egui;
use model::{Segment, StaffKind};

use super::commands::Command;
use super::cursor::Cursor;
use super::keys::{self, DRUM_KEYS, DURATION_KEYS, is_fretted, key_label};
use crate::screens::widgets::{group_label, secondary, toggle};

/// Shows the toolbar; returns the command of a clicked button.
pub fn show(ui: &mut egui::Ui, seg: &Segment, cursor: &Cursor) -> Option<Command> {
    let beat = cursor.beat(seg)?;
    let staff = seg.staves.get(cursor.staff)?;
    let kind = staff.kind;
    let d = beat.duration;
    let key_hint = |key| format!("Key {}", key_label(key));
    let ctx = ui.ctx().clone();
    let shortcut_hint = |s| ctx.format_shortcut(&s);
    let mut clicked = None;
    // A toggle shows its state; an action (`on == None`) is a plain button.
    // `cmd` is `None` for a button that does nothing when clicked.
    let mut button =
        |ui: &mut egui::Ui, on: Option<bool>, text: &str, hint: &str, cmd: Option<Command>| {
            let r = match on {
                Some(on) => toggle(ui, on, text),
                None => secondary(ui, text),
            };
            if r.on_hover_text(hint).clicked() {
                clicked = cmd.or(clicked);
            }
        };
    ui.horizontal_wrapped(|ui| {
        group_label(ui, "Duration");
        for (key, value, label) in DURATION_KEYS {
            let hint = if is_fretted(kind) {
                format!("{} / {}", key_label(keys::LONGER), key_label(keys::SHORTER))
            } else {
                key_hint(key)
            };
            button(ui, Some(d.value == value), label, &hint, Some(Command::Value(value)));
        }
        button(ui, Some(d.dots > 0), "Dot", &key_hint(keys::DOT), Some(Command::ToggleDot));
        let triplet = d.valid_tuplet().is_some();
        button(
            ui,
            Some(triplet),
            "Triplet",
            &shortcut_hint(keys::TRIPLET),
            Some(Command::ToggleTriplet),
        );
        ui.separator();
        group_label(ui, "Beat");
        button(ui, Some(beat.accent), "Accent", &key_hint(keys::ACCENT), Some(Command::Accent));
        if kind != StaffKind::Drums && !staff.has_tab() {
            button(ui, Some(beat.slash), "Slash", &key_hint(keys::SLASH), Some(Command::Slash));
        }
        // On fretted staves 0 types fret 0, so only Delete makes a rest.
        let rest_hint = if is_fretted(kind) {
            keys::CLEAR_KEYS.map(key_label).join(" / ")
        } else {
            key_hint(keys::REST)
        };
        button(ui, None, "Rest", &rest_hint, Some(Command::Clear));
        let repeat_hint =
            format!("Copy this bar into the next ({})", shortcut_hint(keys::REPEAT_BAR));
        button(ui, None, "Repeat bar", &repeat_hint, Some(Command::RepeatBar));
    });
    if kind == StaffKind::Drums {
        ui.horizontal_wrapped(|ui| {
            group_label(ui, "Voice");
            let hint = format!("{}: switch voice", key_hint(keys::VOICE));
            for (voice, label) in [(0, "Hands ↑"), (1, "Feet ↓")] {
                let on = cursor.voice == voice;
                // Clicking the active voice keeps it.
                button(ui, Some(on), label, &hint, (!on).then_some(Command::ToggleVoice));
            }
            ui.separator();
            group_label(ui, "Drums");
            for (key, hit) in DRUM_KEYS {
                let on = super::drums::has_hit(seg, cursor, hit);
                button(ui, Some(on), hit.name(), &key_hint(key), Some(Command::Drum(hit)));
            }
        });
    }
    clicked
}

/// Lines of the keyboard legend, built from the same tables.
pub fn legend(ctx: &egui::Context) -> Vec<String> {
    let s = |sc| ctx.format_shortcut(&sc);
    let durations: Vec<String> =
        DURATION_KEYS.iter().map(|(k, _, label)| format!("{} = {label}", key_label(*k))).collect();
    let drums: Vec<String> =
        DRUM_KEYS.iter().map(|(k, d)| format!("{} {}", key_label(*k), d.name())).collect();
    vec![
        format!(
            "{}/{} select beat · {}/{} staff (tab: string) · {}/{} staff",
            key_label(keys::PREV_BEAT),
            key_label(keys::NEXT_BEAT),
            key_label(keys::UP),
            key_label(keys::DOWN),
            s(keys::STAFF_UP),
            s(keys::STAFF_DOWN)
        ),
        format!(
            "Durations: {} · {} dot · {} triplet · {} (or {})/{} longer/shorter",
            durations.join(", "),
            key_label(keys::DOT),
            s(keys::TRIPLET),
            key_label(keys::LONGER),
            key_label(keys::LONGER_ALT),
            key_label(keys::SHORTER),
        ),
        format!(
            "{} / {} rest · {} accent · {} chord symbol · {} repeat bar into the next",
            key_label(keys::REST),
            keys::CLEAR_KEYS.map(key_label).join(" / "),
            key_label(keys::ACCENT),
            s(keys::CHORD_SYMBOL),
            s(keys::REPEAT_BAR),
        ),
        format!(
            "Guitar/bass: digits type frets on the selected string (two quick digits up to {}) · {} rhythm slash",
            super::commands::MAX_FRET,
            key_label(keys::SLASH),
        ),
        format!(
            "Drums: {} hands/feet voice (kick and hi-hat pedal always go to the feet) · {}",
            key_label(keys::VOICE),
            drums.join(" · ")
        ),
    ]
}
