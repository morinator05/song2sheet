//! Drum input: every hit goes to its own voice, hands (stems up) or feet
//! (stems down, see [`DrumHit::is_foot`]), whichever voice the cursor is in.

use model::{DrumHit, Segment, edit};

use super::cursor::{Cursor, ensure_voice};

const HANDS: usize = 0;
const FEET: usize = 1;

fn voice_of(hit: DrumHit) -> usize {
    if hit.is_foot() { FEET } else { HANDS }
}

/// The beat to toggle `hit` on: the selected one if it already has the hit
/// or is in the hit's voice, else the beat at the same time in the hit's
/// voice. A rest sounding across that time is split so that a beat starts
/// there. `None` when a note sounds across it (the hit then stays put).
pub fn target(seg: &mut Segment, cursor: &Cursor, hit: DrumHit) -> Option<Cursor> {
    let beat = cursor.beat(seg)?;
    let voice = voice_of(hit);
    if beat.drums.contains(&hit) || voice == cursor.voice {
        return Some(*cursor);
    }
    let (tick, duration) = (cursor.tick(seg), beat.duration);
    let mut at = Cursor { voice, ..*cursor };
    ensure_voice(seg, &at);
    let beats = at.voice_mut(seg)?;
    let i = edit::beat_at(beats, tick);
    let start = edit::onset(beats, i);
    at.beat = i;
    if start != tick {
        if !beats[i].is_rest() {
            return None;
        }
        // The new beat at `tick` gets the selected beat's duration if it fits.
        let end = start + beats[i].duration.ticks();
        let mut parts = edit::rests(tick - start, duration);
        at.beat = i + parts.len();
        parts.extend(edit::rests(end - tick, duration));
        beats.splice(i..=i, parts);
    }
    Some(at)
}

/// Whether `hit` sounds at the selected beat's time, in either voice.
pub fn has_hit(seg: &Segment, cursor: &Cursor, hit: DrumHit) -> bool {
    if cursor.beat(seg).is_some_and(|b| b.drums.contains(&hit)) {
        return true;
    }
    let tick = cursor.tick(seg);
    let other = Cursor { voice: voice_of(hit), ..*cursor };
    other.voice(seg).is_some_and(|beats| {
        let i = edit::beat_at(beats, tick);
        edit::onset(beats, i) == tick && beats[i].drums.contains(&hit)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::editor::commands::{Command, Input, add_staff, apply};
    use model::{Duration, NoteValue, StaffKind};

    fn eighths() -> (Segment, Cursor, Input) {
        let mut seg = Segment::default();
        add_staff(&mut seg, StaffKind::Drums);
        let (mut c, mut input) = (Cursor::default(), Input::default());
        apply(Command::Value(NoteValue::Eighth), &mut seg, &mut c, &mut input, 0.0).unwrap();
        (seg, c, input)
    }

    #[test]
    fn kick_typed_in_the_hands_voice_goes_to_the_feet() {
        let (mut seg, mut c, mut input) = eighths();
        apply(Command::Right, &mut seg, &mut c, &mut input, 0.0).unwrap();
        apply(Command::Drum(DrumHit::Kick), &mut seg, &mut c, &mut input, 0.0).unwrap();
        let bar = &seg.staves[0].bars[0];
        assert!(bar.voices[0].iter().all(|b| b.drums.is_empty()), "nothing in the hands");
        let feet = &bar.voices[1];
        let i = edit::beat_at(feet, Duration::new(NoteValue::Eighth).ticks());
        assert_eq!(feet[i].drums, [DrumHit::Kick]);
        assert_eq!(feet[i].duration, Duration::new(NoteValue::Eighth));
        assert_eq!(c.voice, 0, "the cursor stays in the hands voice");
        assert!(has_hit(&seg, &c, DrumHit::Kick));
        // Typing it again removes it.
        apply(Command::Drum(DrumHit::Kick), &mut seg, &mut c, &mut input, 0.0).unwrap();
        assert!(!has_hit(&seg, &c, DrumHit::Kick));
    }

    #[test]
    fn snare_typed_in_the_feet_voice_goes_to_the_hands() {
        let (mut seg, mut c, mut input) = eighths();
        apply(Command::ToggleVoice, &mut seg, &mut c, &mut input, 0.0).unwrap();
        apply(Command::Drum(DrumHit::Snare), &mut seg, &mut c, &mut input, 0.0).unwrap();
        assert_eq!(seg.staves[0].bars[0].voices[0][0].drums, [DrumHit::Snare]);
    }
}
