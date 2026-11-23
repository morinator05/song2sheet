//! Clickable equivalents of the editor's keyboard shortcuts, showing the
//! selected beat's duration. Hints come from the key tables in `keys`.

use eframe::egui;
use model::{Segment, StaffKind};

use super::commands::Command;
use super::cursor::Cursor;
use super::keys::{self, DRUM_KEYS, DURATION_KEYS, is_fretted, key_label};

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
    let mut button = |ui: &mut egui::Ui, selected: bool, text: &str, hint: &str, cmd: Command| {
        if ui.selectable_label(selected, text).on_hover_text(hint).clicked() {
            clicked = Some(cmd);
        }
    };
    ui.horizontal_wrapped(|ui| {
        for (key, value, label) in DURATION_KEYS {
            let hint = if is_fretted(kind) {
                format!("{} / {}", key_label(keys::LONGER), key_label(keys::SHORTER))
            } else {
                key_hint(key)
            };
            button(ui, d.value == value, label, &hint, Command::Value(value));
        }
        button(ui, d.dots > 0, "dot", &key_hint(keys::DOT), Command::ToggleDot);
        button(
            ui,
            d.valid_tuplet().is_some(),
            "triplet",
            &shortcut_hint(keys::TRIPLET),
            Command::ToggleTriplet,
        );
        ui.separator();
        button(ui, beat.accent, "accent", &key_hint(keys::ACCENT), Command::Accent);
        button(ui, false, "rest", &key_hint(keys::REST), Command::Clear);
        button(ui, false, "repeat bar", &shortcut_hint(keys::REPEAT_BAR), Command::RepeatBar);
        if kind == StaffKind::Drums {
            ui.separator();
            let hint = key_hint(keys::VOICE);
            button(ui, cursor.voice == 0, "hands ↑", &hint, Command::ToggleVoice);
            button(ui, cursor.voice == 1, "feet ↓", &hint, Command::ToggleVoice);
        } else if !staff.has_tab() {
            ui.separator();
            button(ui, beat.slash, "slash", &key_hint(keys::SLASH), Command::Slash);
        }
    });
    if kind == StaffKind::Drums {
        ui.horizontal_wrapped(|ui| {
            for (key, hit) in DRUM_KEYS {
                button(
                    ui,
                    beat.drums.contains(&hit),
                    hit.name(),
                    &key_hint(key),
                    Command::Drum(hit),
                );
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
        format!("Drums: {} hands/feet voice · {}", key_label(keys::VOICE), drums.join(" · ")),
    ]
}
