//! Keyboard shortcuts of the section editor: the single source for key
//! handling, toolbar hints and the legend.
//!
//! Durations use MuseScore's numbers (5 = quarter, 4 = eighth …). On
//! guitar and bass staves the digits type frets instead (Guitar Pro
//! style) and `+`/`-` change the duration.

use eframe::egui::{Event, Key, KeyboardShortcut, Modifiers};
use model::{DrumHit, NoteValue, StaffKind};

use super::commands::Command;

/// Duration keys with the toolbar label of each value.
pub const DURATION_KEYS: [(Key, NoteValue, &str); 6] = [
    (Key::Num7, NoteValue::Whole, "1"),
    (Key::Num6, NoteValue::Half, "1/2"),
    (Key::Num5, NoteValue::Quarter, "1/4"),
    (Key::Num4, NoteValue::Eighth, "1/8"),
    (Key::Num3, NoteValue::Sixteenth, "1/16"),
    (Key::Num2, NoteValue::ThirtySecond, "1/32"),
];

/// Drum instrument keys.
pub const DRUM_KEYS: [(Key, DrumHit); 15] = [
    (Key::K, DrumHit::Kick),
    (Key::S, DrumHit::Snare),
    (Key::X, DrumHit::SideStick),
    (Key::H, DrumHit::HiHatClosed),
    (Key::O, DrumHit::HiHatOpen),
    (Key::P, DrumHit::HiHatPedal),
    (Key::R, DrumHit::Ride),
    (Key::B, DrumHit::RideBell),
    (Key::C, DrumHit::Crash),
    (Key::Q, DrumHit::China),
    (Key::E, DrumHit::Splash),
    (Key::T, DrumHit::TomHigh),
    (Key::M, DrumHit::TomMid),
    (Key::F, DrumHit::TomFloor),
    (Key::W, DrumHit::Cowbell),
];

pub const PREV_BEAT: Key = Key::ArrowLeft;
pub const NEXT_BEAT: Key = Key::ArrowRight;
/// Up/down: previous/next staff, or string on tab staves.
pub const UP: Key = Key::ArrowUp;
pub const DOWN: Key = Key::ArrowDown;
pub const DOT: Key = Key::Period;
pub const REST: Key = Key::Num0;
pub const ACCENT: Key = Key::A;
pub const VOICE: Key = Key::V;
pub const SLASH: Key = Key::N;
pub const LONGER: Key = Key::Plus;
/// `=` is `+` without Shift on many layouts.
pub const LONGER_ALT: Key = Key::Equals;
pub const SHORTER: Key = Key::Minus;
/// Keys that also make a rest, besides `REST`.
pub const CLEAR_KEYS: [Key; 2] = [Key::Delete, Key::Backspace];

pub const CHORD_SYMBOL: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::K);
pub const REPEAT_BAR: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);
pub const TRIPLET: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Num3);
pub const STAFF_UP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::ArrowUp);
pub const STAFF_DOWN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::ArrowDown);

/// What the user wants from a key press, besides plain commands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Edit(Command),
    /// Type a chord symbol for the selected beat.
    ChordSymbol,
}

/// Whether digits type frets on this staff.
pub fn is_fretted(kind: StaffKind) -> bool {
    matches!(kind, StaffKind::Guitar | StaffKind::Bass)
}

/// Maps one key press on a staff of `kind` to an action.
pub fn action(key: Key, mods: Modifiers, kind: StaffKind) -> Option<Action> {
    use Command::*;
    let edit = |c| Some(Action::Edit(c));
    if mods.command {
        let is = |s: KeyboardShortcut| s.logical_key == key;
        return if is(CHORD_SYMBOL) {
            Some(Action::ChordSymbol)
        } else if is(REPEAT_BAR) {
            edit(RepeatBar)
        } else if is(TRIPLET) {
            edit(ToggleTriplet)
        } else if is(STAFF_UP) {
            edit(StaffUp)
        } else if is(STAFF_DOWN) {
            edit(StaffDown)
        } else {
            None
        };
    }
    if mods.ctrl || mods.alt {
        return None; // e.g. Ctrl on macOS: not ours
    }
    if let Some(d) = key_digit(key)
        && is_fretted(kind)
    {
        return edit(FretDigit(d));
    }
    if let Some((_, value, _)) = DURATION_KEYS.iter().find(|(k, _, _)| *k == key) {
        return edit(Value(*value));
    }
    match key {
        PREV_BEAT => edit(Left),
        NEXT_BEAT => edit(Right),
        UP if is_fretted(kind) => edit(StringUp),
        DOWN if is_fretted(kind) => edit(StringDown),
        UP => edit(StaffUp),
        DOWN => edit(StaffDown),
        DOT => edit(ToggleDot),
        LONGER | LONGER_ALT => edit(Longer),
        SHORTER => edit(Shorter),
        REST => edit(Clear),
        _ if CLEAR_KEYS.contains(&key) => edit(Clear),
        ACCENT => edit(Accent),
        VOICE if kind == StaffKind::Drums => edit(ToggleVoice),
        SLASH if kind != StaffKind::Drums => edit(Slash),
        _ if kind == StaffKind::Drums => {
            DRUM_KEYS.iter().find(|(k, _)| *k == key).and_then(|(_, d)| edit(Drum(*d)))
        }
        _ => None,
    }
}

/// Commands that make sense when a held key auto-repeats; toggles don't.
fn repeats(action: Action) -> bool {
    use Command::*;
    matches!(
        action,
        Action::Edit(
            Left | Right | StringUp | StringDown | StaffUp | StaffDown | Longer | Shorter | Clear
        )
    )
}

/// All key presses in `events`, as actions for a staff of `kind`.
pub fn actions(events: &[Event], kind: StaffKind) -> Vec<Action> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Key { key, pressed: true, modifiers, repeat, .. } => {
                action(*key, *modifiers, kind).filter(|a| !*repeat || repeats(*a))
            }
            _ => None,
        })
        .collect()
}

fn key_digit(key: Key) -> Option<u8> {
    use Key::*;
    [Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9]
        .iter()
        .position(|k| *k == key)
        .map(|d| d as u8)
}

/// Human-readable name of a key, e.g. "." for the period.
pub fn key_label(key: Key) -> &'static str {
    key.symbol_or_name()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Modifiers = Modifiers::NONE;

    #[test]
    fn digits_mean_durations_on_drums_and_frets_on_guitar() {
        assert_eq!(
            action(Key::Num5, NONE, StaffKind::Drums),
            Some(Action::Edit(Command::Value(NoteValue::Quarter)))
        );
        assert_eq!(
            action(Key::Num5, NONE, StaffKind::Guitar),
            Some(Action::Edit(Command::FretDigit(5)))
        );
        assert_eq!(action(REST, NONE, StaffKind::Keys), Some(Action::Edit(Command::Clear)));
    }

    #[test]
    fn drum_letters_only_on_drum_staves() {
        assert_eq!(
            action(Key::H, NONE, StaffKind::Drums),
            Some(Action::Edit(Command::Drum(DrumHit::HiHatClosed)))
        );
        assert_eq!(action(Key::H, NONE, StaffKind::Guitar), None);
    }

    #[test]
    fn drum_keys_are_unique_and_free() {
        let reserved = [ACCENT, VOICE, SLASH, DOT, REST, LONGER, SHORTER];
        for (i, (k, _)) in DRUM_KEYS.iter().enumerate() {
            assert!(!reserved.contains(k), "{k:?} is reserved");
            assert!(DRUM_KEYS[i + 1..].iter().all(|(o, _)| o != k), "{k:?} twice");
        }
    }

    #[test]
    fn command_shortcuts() {
        assert_eq!(
            action(Key::K, Modifiers::COMMAND, StaffKind::Guitar),
            Some(Action::ChordSymbol)
        );
        assert_eq!(
            action(Key::R, Modifiers::COMMAND, StaffKind::Drums),
            Some(Action::Edit(Command::RepeatBar))
        );
        // Ctrl+H must not toggle the hi-hat where Ctrl is not the command key.
        assert_eq!(action(Key::H, Modifiers::CTRL, StaffKind::Drums), None);
    }

    #[test]
    fn held_keys_repeat_only_navigation() {
        let held = |key| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: NONE,
        };
        assert!(actions(&[held(Key::H)], StaffKind::Drums).is_empty());
        assert_eq!(actions(&[held(Key::ArrowRight)], StaffKind::Drums).len(), 1);
    }
}
