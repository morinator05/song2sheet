//! Song sheet data model and the `.s2s.json` file format.
//!
//! A [`Project`] is one song: metadata, a pool of reusable [`Segment`]s (short
//! excerpts of notation) and the [`sheet`](Project::sheet), an ordered list of
//! [`SheetEntry`]s that reference segments by id. The same segment can appear
//! in the sheet many times (e.g. a verse played three times is stored once).

mod drums;
pub mod edit;
mod project;
mod rhythm;
mod staff;

pub use drums::DrumHit;
pub use project::*;
pub use rhythm::*;
pub use staff::*;

/// Current file format version, written into every saved project.
pub const FORMAT_VERSION: u32 = 1;

/// File extension used for saved projects.
pub const FILE_EXTENSION: &str = "s2s.json";

/// Ticks per quarter note. All durations are expressed in ticks.
pub const TICKS_PER_QUARTER: u32 = 960;

/// Ticks per whole note.
pub const TICKS_PER_WHOLE: u32 = TICKS_PER_QUARTER * 4;

/// Application name, used in window titles and as the PDF creator.
pub const APP_NAME: &str = "song2sheet";

/// Display name for a song without a title.
pub const UNTITLED: &str = "Untitled";

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}
