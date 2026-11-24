//! App-wide keyboard shortcuts (the editor's own keys are in
//! `screens/editor/keys.rs`). Shortcuts with Shift come before the same
//! key without it, since egui's matching ignores extra Shift presses.

use eframe::egui::{Key, KeyboardShortcut, Modifiers};

const fn cmd(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND, key)
}
const fn cmd_shift(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), key)
}
const fn alt(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::ALT, key)
}

pub const NEW: KeyboardShortcut = cmd(Key::N);
pub const OPEN: KeyboardShortcut = cmd(Key::O);
pub const SAVE_AS: KeyboardShortcut = cmd_shift(Key::S);
pub const SAVE: KeyboardShortcut = cmd(Key::S);
pub const EXPORT: KeyboardShortcut = cmd(Key::E);
pub const QUIT: KeyboardShortcut = cmd(Key::Q);
pub const REDO: KeyboardShortcut = cmd_shift(Key::Z);
pub const REDO_ALT: KeyboardShortcut = cmd(Key::Y);
pub const UNDO: KeyboardShortcut = cmd(Key::Z);
/// Switch the centre screen. Alt, since Ctrl+3 is the editor's triplet.
pub const SHOW_SHEET: KeyboardShortcut = alt(Key::Num1);
pub const SHOW_EDITOR: KeyboardShortcut = alt(Key::Num2);
pub const SHOW_PICKER: KeyboardShortcut = alt(Key::Num3);
/// Sheet: move the row being edited up or down.
pub const MOVE_ROW_UP: KeyboardShortcut = alt(Key::ArrowUp);
pub const MOVE_ROW_DOWN: KeyboardShortcut = alt(Key::ArrowDown);
/// Import screen: focus the new section's name field.
pub const FOCUS_NAME: KeyboardShortcut = cmd(Key::L);
/// Back to the sheet from the editor or the import screen.
pub const BACK: Key = Key::Escape;

/// Every app-wide shortcut with what it does, for the shortcuts popup.
pub const LEGEND: [(KeyboardShortcut, &str); 14] = [
    (NEW, "New song"),
    (OPEN, "Open (a score on the Import screen)"),
    (SAVE, "Save"),
    (SAVE_AS, "Save as"),
    (EXPORT, "Export PDF"),
    (UNDO, "Undo"),
    (REDO, "Redo"),
    (SHOW_SHEET, "Show the sheet"),
    (SHOW_EDITOR, "Show the section editor"),
    (SHOW_PICKER, "Show the import screen"),
    (MOVE_ROW_UP, "Sheet: move the row being edited up"),
    (MOVE_ROW_DOWN, "Sheet: move the row being edited down"),
    (FOCUS_NAME, "Import: go to the section name"),
    (QUIT, "Quit"),
];
