//! MusicXML (partwise) import: parts become staves, bars keep their
//! rhythm, drums, chord symbols, tab and accents. Pitched notes without tab
//! become rhythm slashes (a song sheet shows rhythm, not melody).

mod harmony;
mod instruments;
mod measure;
mod notes;

use std::collections::HashMap;
use std::io::Read;

use roxmltree::{Document, Node, ParsingOptions};

use crate::{BarInfo, ImportError, Part, Score};

/// Path of the container file that names the score inside an `.mxl`.
const MXL_CONTAINER: &str = "META-INF/container.xml";
/// Largest file read from inside an `.mxl` (guards against zip bombs).
const MAX_UNZIPPED_BYTES: u64 = 64 * 1024 * 1024;
const UTF16_LE_BOM: [u8; 2] = [0xFF, 0xFE];
const UTF16_BE_BOM: [u8; 2] = [0xFE, 0xFF];

/// Text of an XML file: UTF-8, or UTF-16 with a byte order mark.
pub fn decode(bytes: &[u8]) -> Result<String, ImportError> {
    let utf16 = |big_endian: bool| {
        let (pairs, _) = bytes[2..].as_chunks::<2>();
        let units: Vec<u16> = pairs
            .iter()
            .map(|&p| if big_endian { u16::from_be_bytes(p) } else { u16::from_le_bytes(p) })
            .collect();
        String::from_utf16(&units).map_err(format_error)
    };
    match bytes.get(..2) {
        Some(b) if b == UTF16_LE_BOM => utf16(false),
        Some(b) if b == UTF16_BE_BOM => utf16(true),
        _ => String::from_utf8(bytes.to_vec())
            .map_err(|_| ImportError::Format("the file is neither UTF-8 nor UTF-16".into())),
    }
}

fn format_error(e: impl std::fmt::Display) -> ImportError {
    ImportError::Format(e.to_string())
}

/// First child element called `name`.
fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children().find(|c| c.has_tag_name(name))
}

/// Trimmed text of the first child element called `name`.
fn child_text<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    child(node, name).and_then(|c| c.text()).map(str::trim)
}

fn child_number<T: std::str::FromStr>(node: Node, name: &str) -> Option<T> {
    child_text(node, name)?.parse().ok()
}

/// Parses an uncompressed MusicXML document.
pub fn parse(xml: &str) -> Result<Score, ImportError> {
    // MusicXML files start with a `<!DOCTYPE>`. roxmltree only reads its
    // internal subset (with entity-expansion limits) and never loads
    // external entities.
    let options = ParsingOptions { allow_dtd: true, ..ParsingOptions::default() };
    let doc = Document::parse_with_options(xml, options).map_err(format_error)?;
    let root = doc.root_element();
    if !root.has_tag_name("score-partwise") {
        return Err(ImportError::Format(format!(
            "expected <score-partwise>, found <{}> (only partwise scores are supported)",
            root.tag_name().name()
        )));
    }
    let title = child(root, "work")
        .and_then(|w| child_text(w, "work-title"))
        .or_else(|| child_text(root, "movement-title"))
        .unwrap_or_default()
        .to_string();
    let composer = child(root, "identification")
        .and_then(|id| {
            id.children()
                .find(|c| c.has_tag_name("creator") && c.attribute("type") == Some("composer"))
        })
        .and_then(|c| c.text())
        .unwrap_or_default()
        .trim()
        .to_string();

    let part_list: HashMap<&str, Node> = child(root, "part-list")
        .map(|pl| {
            pl.children()
                .filter(|c| c.has_tag_name("score-part"))
                .filter_map(|c| Some((c.attribute("id")?, c)))
                .collect()
        })
        .unwrap_or_default();

    let mut parts = Vec::new();
    let mut bars = Vec::new();
    for part in root.children().filter(|c| c.has_tag_name("part")) {
        let info = part.attribute("id").and_then(|id| part_list.get(id)).copied();
        let name = info.and_then(|i| child_text(i, "part-name")).unwrap_or_default().to_string();
        let drums = info.map(instruments::drum_map).unwrap_or_default();
        let parsed = measure::parse_part(part, &drums);
        let mut staff = parsed.staff;
        staff.kind = instruments::kind(&name, parsed.percussion);
        // No label: the short default (e.g. "Gtr") fits the sheet's label
        // column, full part names usually don't.
        if bars.is_empty() {
            bars = parsed.bars;
        }
        parts.push(Part { name, staff });
    }
    if parts.is_empty() {
        return Err(ImportError::Format("the score has no parts".into()));
    }
    even_out_bars(&mut parts, &mut bars);
    Ok(Score { title, composer, parts, bars })
}

/// Gives every part as many bars as the longest one (parts may differ in
/// broken files), padding with rests and repeating the last bar's info.
fn even_out_bars(parts: &mut [Part], bars: &mut Vec<BarInfo>) {
    let count = parts.iter().map(|p| p.staff.bars.len()).max().unwrap_or(0).max(bars.len());
    let last = bars.last().cloned().unwrap_or_default();
    bars.resize(count, BarInfo { number: String::new(), ..last });
    for part in parts {
        let start = part.staff.bars.len();
        let padding =
            bars[start..].iter().map(|b| model::edit::empty_bar(1, b.time_sig.bar_ticks()));
        part.staff.bars.extend(padding);
    }
}

/// Parses a compressed `.mxl` file (a zip with the score inside).
pub fn parse_mxl(bytes: &[u8]) -> Result<Score, ImportError> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(format_error)?;
    let mut read = |name: &str| -> Result<String, ImportError> {
        let mut bytes = Vec::new();
        let entry = zip.by_name(name).map_err(format_error)?;
        entry.take(MAX_UNZIPPED_BYTES).read_to_end(&mut bytes).map_err(ImportError::Io)?;
        decode(&bytes)
    };
    let container = read(MXL_CONTAINER)?;
    let doc = Document::parse(&container).map_err(format_error)?;
    let score_path = doc
        .descendants()
        .find(|n| n.has_tag_name("rootfile"))
        .and_then(|n| n.attribute("full-path"))
        .ok_or_else(|| ImportError::Format(format!("{MXL_CONTAINER} names no score")))?
        .to_string();
    parse(&read(&score_path)?)
}

#[cfg(test)]
mod tests;
