//! Turns a [`model::Project`] into a renderer-independent [`DisplayList`].
//!
//! Both the PDF exporter and the on-screen preview draw the same display
//! list, so the preview is exactly what gets printed.

pub mod display;
pub mod fonts;
mod layout;
mod notation;
mod smufl;

pub use display::{A4_HEIGHT, A4_WIDTH, BBox, DisplayList, Item, Page, Shade};
pub use fonts::FontId;
pub use layout::{SegmentView, has_staff_labels, layout_segment, layout_sheet};
pub use notation::{BeatBox, StaffLabel, SystemSpan};
