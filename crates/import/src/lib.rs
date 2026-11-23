//! Importing scores from other programs, and cutting sections out of them.
//!
//! Supported: MusicXML (`.musicxml`, `.xml`, compressed `.mxl`) — the export
//! format of MuseScore and most notation programs.

mod musicxml;
mod score;

pub use score::{BarInfo, Part, Score, extract};

use std::path::Path;

/// Compressed MusicXML.
const MXL_EXTENSION: &str = "mxl";
/// Plain MusicXML.
const XML_EXTENSIONS: [&str; 2] = ["musicxml", "xml"];
/// File extensions [`load`] understands.
pub const EXTENSIONS: [&str; 3] = [XML_EXTENSIONS[0], XML_EXTENSIONS[1], MXL_EXTENSION];

#[derive(Debug)]
pub enum ImportError {
    Io(std::io::Error),
    /// The file is not valid XML or not a MusicXML score.
    Format(String),
    /// The extension is not one of [`EXTENSIONS`].
    UnknownType,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Io(e) => write!(f, "cannot read file: {e}"),
            ImportError::Format(e) => write!(f, "not a readable MusicXML score: {e}"),
            ImportError::UnknownType => {
                write!(
                    f,
                    "unsupported file type (expected {})",
                    EXTENSIONS.map(|e| format!(".{e}")).join(", ")
                )
            }
        }
    }
}

impl std::error::Error for ImportError {}

/// Reads a score file, choosing the format by extension.
pub fn load(path: &Path) -> Result<Score, ImportError> {
    let ext = path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    let bytes = std::fs::read(path).map_err(ImportError::Io)?;
    match ext.as_deref() {
        Some(MXL_EXTENSION) => musicxml::parse_mxl(&bytes),
        Some(e) if XML_EXTENSIONS.contains(&e) => musicxml::parse(&musicxml::decode(&bytes)?),
        _ => Err(ImportError::UnknownType),
    }
}
