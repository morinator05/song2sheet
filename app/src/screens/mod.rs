//! UI screens. Each `show` function edits the project in place.

pub mod arranger;
pub mod editor;
mod notation_view;
pub mod picker;
pub mod song_panel;
pub mod widgets;

use model::SegmentId;

/// Where a screen asks the app to go next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Nav {
    /// Open this section in the editor.
    Edit(SegmentId),
    Import,
    Sheet,
}
