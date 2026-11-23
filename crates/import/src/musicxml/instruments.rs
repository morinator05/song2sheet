//! Which kind of staff a part is, and which drum each unpitched note is.

use std::collections::HashMap;

use model::{DrumHit, StaffKind};
use roxmltree::Node;

use super::child_number;

/// Part name keywords (lower case) for each staff kind, checked in order.
const KIND_KEYWORDS: [(StaffKind, &[&str]); 5] = [
    (StaffKind::Drums, &["drum", "percussion", "kit"]),
    (StaffKind::Bass, &["bass"]),
    (StaffKind::Guitar, &["guitar", "gtr"]),
    (StaffKind::Keys, &["piano", "keys", "keyboard", "organ", "synth"]),
    (StaffKind::Vocals, &["voice", "vocal", "voc", "sing"]),
];

/// MusicXML numbers MIDI keys from 1, General MIDI from 0.
const MIDI_UNPITCHED_OFFSET: u8 = 1;

/// Guesses the staff kind from the part name; percussion parts are drums.
pub fn kind(part_name: &str, percussion: bool) -> StaffKind {
    if percussion {
        return StaffKind::Drums;
    }
    let name = part_name.to_lowercase();
    KIND_KEYWORDS
        .iter()
        .find(|(_, words)| words.iter().any(|w| name.contains(w)))
        .map_or(StaffKind::Other, |(k, _)| *k)
}

/// Instrument id → General MIDI percussion key, from a `<score-part>`.
pub fn drum_map(score_part: Node) -> HashMap<String, u8> {
    score_part
        .children()
        .filter(|c| c.has_tag_name("midi-instrument"))
        .filter_map(|m| {
            let key: u8 = child_number(m, "midi-unpitched")?;
            Some((m.attribute("id")?.to_string(), key.checked_sub(MIDI_UNPITCHED_OFFSET)?))
        })
        .collect()
}

/// Drum for an unpitched note shown at `step`/`octave` (the standard drum
/// staff positions), when the instrument is unknown.
pub fn drum_at(step: &str, octave: u8, x_head: bool) -> Option<DrumHit> {
    Some(match (step, octave, x_head) {
        ("D", 4, _) => DrumHit::HiHatPedal,
        ("E" | "F", 4, _) => DrumHit::Kick,
        ("A", 4, _) => DrumHit::TomFloor,
        ("C", 5, true) => DrumHit::SideStick,
        ("C", 5, false) => DrumHit::Snare,
        ("D", 5, _) => DrumHit::TomMid,
        ("E", 5, _) => DrumHit::TomHigh,
        ("F", 5, _) => DrumHit::Ride,
        ("G", 5, _) => DrumHit::HiHatClosed,
        ("A", 5, _) => DrumHit::Crash,
        ("B", 5, _) => DrumHit::China,
        _ => return None,
    })
}
