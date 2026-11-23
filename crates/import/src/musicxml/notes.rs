//! Reading one MusicXML `<note>`: its duration and what it adds to a beat.

use std::collections::HashMap;

use model::{Beat, DrumHit, Duration, NoteValue, TabNote, Tuplet};
use roxmltree::Node;

use super::{child, child_number, child_text, instruments};

/// MusicXML note types. Values shorter than a 32nd become 32nds and the
/// breve becomes a whole note: a song sheet doesn't need them.
const NOTE_TYPES: [(&str, NoteValue); 8] = [
    ("breve", NoteValue::Whole),
    ("whole", NoteValue::Whole),
    ("half", NoteValue::Half),
    ("quarter", NoteValue::Quarter),
    ("eighth", NoteValue::Eighth),
    ("16th", NoteValue::Sixteenth),
    ("32nd", NoteValue::ThirtySecond),
    ("64th", NoteValue::ThirtySecond),
];

/// Notehead MusicXML uses for cymbals and other "x" notes.
const X_NOTEHEAD: &str = "x";

/// Duration written as type, dots and tuplet; `None` if the note has no
/// type (e.g. a whole-bar rest).
pub fn duration(note: Node) -> Option<Duration> {
    let kind = child_text(note, "type")?;
    let value = NOTE_TYPES.iter().find(|(t, _)| *t == kind)?.1;
    let dots = note.children().filter(|c| c.has_tag_name("dot")).count() as u8;
    let tuplet = child(note, "time-modification").and_then(|tm| {
        let t = Tuplet {
            actual: child_number(tm, "actual-notes")?,
            normal: child_number(tm, "normal-notes")?,
        };
        t.is_valid().then_some(t)
    });
    Some(Duration { value, dots, tuplet })
}

/// Adds the sound of `note` (drum, fret or rhythm slash, accent) to `beat`.
/// `drums` maps instrument ids to General MIDI keys.
pub fn add_to_beat(beat: &mut Beat, note: Node, drums: &HashMap<String, u8>) {
    if let Some(unpitched) = child(note, "unpitched") {
        let by_instrument = child(note, "instrument")
            .and_then(|i| i.attribute("id"))
            .and_then(|id| drums.get(id))
            .and_then(|&key| DrumHit::from_gm(key));
        let by_position = || {
            let x_head = child_text(note, "notehead") == Some(X_NOTEHEAD);
            instruments::drum_at(
                child_text(unpitched, "display-step")?,
                child_number(unpitched, "display-octave")?,
                x_head,
            )
        };
        if let Some(hit) = by_instrument.or_else(by_position)
            && !beat.drums.contains(&hit)
        {
            beat.drums.push(hit);
        }
    } else if child(note, "pitch").is_some() {
        let technical = notations(note).find_map(|n| child(n, "technical"));
        let fret = technical.and_then(|t| {
            Some(TabNote { string: child_number(t, "string")?, fret: child_number(t, "fret")? })
        });
        match fret {
            Some(tab) => {
                beat.tab.retain(|n| n.string != tab.string);
                beat.tab.push(tab);
            }
            None => beat.slash = true,
        }
    }
    let mut articulations = notations(note).filter_map(|n| child(n, "articulations"));
    if articulations.any(|a| child(a, "accent").is_some() || child(a, "strong-accent").is_some()) {
        beat.accent = true;
    }
}

/// All `<notations>` of a note (the spec allows several).
fn notations<'a, 'i>(note: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    note.children().filter(|c| c.has_tag_name("notations"))
}

/// Whether the note is percussion (unpitched).
pub fn is_unpitched(note: Node) -> bool {
    child(note, "unpitched").is_some()
}
