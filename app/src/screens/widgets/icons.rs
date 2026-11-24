//! Icon code points of the embedded Phosphor font (regular weight).
//! They render in any text, since the font is a fallback of every family.

pub const ADD: &str = "\u{e3d4}";
pub const REMOVE: &str = "\u{e32a}";
pub const EDIT: &str = "\u{e3b4}";
pub const DUPLICATE: &str = "\u{e1ca}";
pub const DELETE: &str = "\u{e4a6}";
pub const DRAG: &str = "\u{eae2}";
pub const NEW_FILE: &str = "\u{e236}";
pub const OPEN: &str = "\u{e256}";
pub const RECENT: &str = "\u{e1a0}";
pub const SAVE: &str = "\u{e248}";
pub const EXPORT: &str = "\u{e702}";
pub const UNDO: &str = "\u{e038}";
pub const REDO: &str = "\u{e036}";
pub const BACK: &str = "\u{e058}";
pub const KEYBOARD: &str = "\u{e2d8}";
pub const SCORE: &str = "\u{e340}";
pub const ADD_TO_SHEET: &str = "\u{e2f8}";
pub const WARNING: &str = "\u{e4e0}";
pub const SUCCESS: &str = "\u{e184}";
pub const ERROR: &str = "\u{e4f8}";

/// `icon` and `text` as one button or label text.
pub fn with(icon: &str, text: &str) -> String {
    format!("{icon}  {text}")
}
