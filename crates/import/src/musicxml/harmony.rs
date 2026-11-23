//! Chord symbol text from MusicXML `<harmony>`.

use roxmltree::Node;

use super::{child, child_text};

/// Suffixes for MusicXML chord kinds (used when the file gives no text).
const KIND_SUFFIXES: [(&str, &str); 26] = [
    ("major", ""),
    ("minor", "m"),
    ("augmented", "aug"),
    ("diminished", "dim"),
    ("dominant", "7"),
    ("major-seventh", "maj7"),
    ("minor-seventh", "m7"),
    ("diminished-seventh", "dim7"),
    ("augmented-seventh", "aug7"),
    ("half-diminished", "m7b5"),
    ("major-minor", "m(maj7)"),
    ("major-sixth", "6"),
    ("minor-sixth", "m6"),
    ("dominant-ninth", "9"),
    ("dominant-11th", "11"),
    ("dominant-13th", "13"),
    ("major-11th", "maj11"),
    ("major-13th", "maj13"),
    ("minor-11th", "m11"),
    ("minor-13th", "m13"),
    ("major-ninth", "maj9"),
    ("minor-ninth", "m9"),
    ("suspended-second", "sus2"),
    ("suspended-fourth", "sus4"),
    ("power", "5"),
    ("none", ""),
];

const SHARP: &str = "#";
/// Between chord and bass note, as in "C/E".
const BASS_SEPARATOR: char = '/';
const FLAT: &str = "b";

/// "#", "b" or "" for a MusicXML alteration like "-1".
fn accidental(alter: Option<&str>) -> &'static str {
    match alter.and_then(|a| a.parse::<f32>().ok()).map(|a| a.round() as i32) {
        Some(1) => SHARP,
        Some(-1) => FLAT,
        _ => "",
    }
}

/// "A", "Bb", "F#" from a step and an alteration.
fn note_name(step: &str, alter: Option<&str>) -> String {
    format!("{step}{}", accidental(alter))
}

/// Chord degree changes like "add9", "b9", "no3".
fn degrees(harmony: Node) -> String {
    harmony
        .children()
        .filter(|c| c.has_tag_name("degree"))
        .filter_map(|d| {
            let value = child_text(d, "degree-value")?;
            let alter = accidental(child_text(d, "degree-alter"));
            Some(match child_text(d, "degree-type") {
                Some("add") => format!("add{alter}{value}"),
                Some("subtract") => format!("no{value}"),
                _ => format!("{alter}{value}"),
            })
        })
        .collect()
}

/// Chord symbol like "Bbmaj7/F", or `None` if the element has no root.
pub fn chord_text(harmony: Node) -> Option<String> {
    let root = child(harmony, "root")?;
    let mut text = note_name(child_text(root, "root-step")?, child_text(root, "root-alter"));
    if let Some(kind) = child(harmony, "kind") {
        let value = kind.text().unwrap_or_default().trim();
        let suffix = kind
            .attribute("text")
            .map(str::to_string)
            .or_else(|| KIND_SUFFIXES.iter().find(|(k, _)| *k == value).map(|(_, s)| s.to_string()))
            .unwrap_or_default();
        text.push_str(&suffix);
    }
    text.push_str(&degrees(harmony));
    if let Some(bass) = child(harmony, "bass")
        && let Some(step) = child_text(bass, "bass-step")
    {
        text.push(BASS_SEPARATOR);
        text.push_str(&note_name(step, child_text(bass, "bass-alter")));
    }
    Some(text)
}
