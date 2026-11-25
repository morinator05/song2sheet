//! Engraving of a segment's staves into one or more single-line systems.
//!
//! Deliberately minimal: percussion staves, rhythm-slash staves (with chord
//! symbols) and tab staves with rhythm stems; flat beams; no pitched notation.
//!
//! Pipeline: [`systems`] measures bars and packs them into systems,
//! [`staff`] draws one staff of one system, [`events`] draws individual
//! beats and [`beams`] connects them.

mod beams;
mod events;
mod metrics;
mod staff;
mod systems;

pub use metrics::Spacing;
pub use systems::{BeatBox, StaffLabel, SystemSpan, engrave_segment};
