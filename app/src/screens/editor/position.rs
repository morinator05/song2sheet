//! The selected beat's position in musical terms, for the caption under
//! the notation: "Drums · bar 2 · count 3&".

use model::{Segment, StaffKind, TICKS_PER_WHOLE, TimeSig};

use super::cursor::Cursor;

/// Syllables of the sixteenths within a beat ("1 e & a").
const SUBDIVISIONS: [&str; 4] = ["", "e", "&", "a"];
/// Marks a time between those syllables (e.g. a triplet).
const BETWEEN: &str = "+";

/// The count of `tick` in a bar of `time_sig`: the beat number plus the
/// sixteenth syllable, e.g. "3&".
pub fn count(tick: u32, time_sig: TimeSig) -> String {
    let beat_ticks = TICKS_PER_WHOLE / u32::from(time_sig.den.max(1));
    let beat = tick / beat_ticks + 1;
    let rest = tick % beat_ticks;
    let step = beat_ticks / SUBDIVISIONS.len() as u32;
    let syllable = if step > 0 && rest.is_multiple_of(step) {
        SUBDIVISIONS[(rest / step) as usize]
    } else {
        BETWEEN
    };
    format!("{beat}{syllable}")
}

/// "Drums · bar 2 · count 3& · feet", with the string on tab staves.
pub fn caption(seg: &Segment, cursor: &Cursor) -> String {
    let Some(staff) = seg.staves.get(cursor.staff) else { return String::new() };
    let name = if staff.label.trim().is_empty() { staff.kind.name() } else { staff.label.trim() };
    let mut text = format!(
        "{name} · bar {} · count {}",
        cursor.bar + 1,
        count(cursor.tick(seg), seg.time_sig)
    );
    if staff.kind == StaffKind::Drums {
        text.push_str(if cursor.voice == 0 { " · hands" } else { " · feet" });
    } else if staff.has_tab() {
        text.push_str(&format!(" · string {}", cursor.string));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::TICKS_PER_QUARTER;

    const FOUR_FOUR: TimeSig = TimeSig { num: 4, den: 4 };

    #[test]
    fn counts_beats_and_sixteenths() {
        assert_eq!(count(0, FOUR_FOUR), "1");
        assert_eq!(count(TICKS_PER_QUARTER / 2, FOUR_FOUR), "1&");
        assert_eq!(count(2 * TICKS_PER_QUARTER + TICKS_PER_QUARTER / 4, FOUR_FOUR), "3e");
        assert_eq!(count(3 * TICKS_PER_QUARTER + 3 * TICKS_PER_QUARTER / 4, FOUR_FOUR), "4a");
        assert_eq!(count(TICKS_PER_QUARTER / 3, FOUR_FOUR), "1+", "triplet");
    }

    #[test]
    fn eighth_note_meters_count_eighths() {
        let six_eight = TimeSig { num: 6, den: 8 };
        assert_eq!(count(5 * TICKS_PER_QUARTER / 2, six_eight), "6");
    }
}
