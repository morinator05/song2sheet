//! Beam grouping, flat beams, flags and tuplet numbers for one voice.

use model::NoteValue;

use super::events::Event;
use super::metrics::*;
use crate::display::Item;
use crate::fonts::{self, FontId};
use crate::smufl::*;

/// Groups beamable events (indices into `evs`) by beat group. Rests and
/// notes crossing a group boundary break beams. Single-element groups get
/// flags instead of beams.
pub fn beam_groups(evs: &[Event], group_ticks: u32) -> Vec<Vec<usize>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    let mut cur_group = None;
    for (i, ev) in evs.iter().enumerate() {
        if ev.stem.is_none() || ev.value().beams() == 0 {
            groups.extend((!cur.is_empty()).then(|| std::mem::take(&mut cur)));
            cur_group = None;
            continue;
        }
        let g = ev.onset / group_ticks;
        let g_end = (ev.onset + ev.beat.duration.ticks()).saturating_sub(1) / group_ticks;
        let crosses = g != g_end;
        if (cur_group != Some(g) || crosses) && !cur.is_empty() {
            groups.push(std::mem::take(&mut cur));
        }
        cur.push(i);
        cur_group = Some(g);
        if crosses {
            groups.push(std::mem::take(&mut cur));
            cur_group = None;
        }
    }
    groups.extend((!cur.is_empty()).then_some(cur));
    groups
}

/// Extends stems so every beamed group has one flat beam line.
pub fn align_stems(evs: &mut [Event], groups: &[Vec<usize>]) {
    for g in groups.iter().filter(|g| g.len() > 1) {
        let up = evs[g[0]].up;
        let tips = g.iter().filter_map(|&i| evs[i].stem.map(|s| s.tip));
        let beam_y = if up { tips.fold(f32::MAX, f32::min) } else { tips.fold(f32::MIN, f32::max) };
        for &i in g {
            if let Some(s) = &mut evs[i].stem {
                s.tip = beam_y;
            }
        }
    }
}

/// Draws flags for single notes and beams for groups.
pub fn draw_beams(items: &mut Vec<Item>, evs: &[Event], groups: &[Vec<usize>]) {
    for g in groups {
        if let [single] = g[..] {
            let ev = &evs[single];
            if let (Some(s), Some(f)) = (ev.stem, flag_glyph(ev.value(), ev.up)) {
                items.push(music(s.x - STEM_W / 2.0, s.tip, f));
            }
        } else {
            draw_group(items, evs, g);
        }
    }
}

fn draw_group(items: &mut Vec<Item>, evs: &[Event], group: &[usize]) {
    let up = evs[group[0]].up;
    let stems: Vec<(f32, u8)> =
        group.iter().filter_map(|&i| evs[i].stem.map(|s| (s.x, evs[i].value().beams()))).collect();
    let Some(beam_y) = evs[group[0]].stem.map(|s| s.tip) else { return };
    let level_y = |k: u8| {
        if up { beam_y + k as f32 * BEAM_STEP } else { beam_y - BEAM_H - k as f32 * BEAM_STEP }
    };
    let max_beams = stems.iter().map(|s| s.1).max().unwrap_or(1);
    for level in 0..max_beams {
        for (x0, x1) in beam_segments(&stems, level) {
            items.push(rect(x0 - STEM_W / 2.0, level_y(level), x1 - x0 + STEM_W, BEAM_H));
        }
    }
}

/// Horizontal extents of beam `level` (0 = outermost) across stems given as
/// (x, beam count). A lone note gets a short stub pointing into the group.
fn beam_segments(stems: &[(f32, u8)], level: u8) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < stems.len() {
        if stems[i].1 <= level {
            i += 1;
            continue;
        }
        let start = i;
        while i + 1 < stems.len() && stems[i + 1].1 > level {
            i += 1;
        }
        out.push(match (start == i, i + 1 == stems.len()) {
            (false, _) => (stems[start].0, stems[i].0),
            (true, true) => (stems[i].0 - BEAM_STUB, stems[i].0),
            (true, false) => (stems[i].0, stems[i].0 + BEAM_STUB),
        });
        i += 1;
    }
    out
}

/// Draws the tuplet number outside each tuplet group. A group ends at a
/// beam-group boundary (normally the beat) or where the tuplet changes.
pub fn draw_tuplets(items: &mut Vec<Item>, evs: &[Event], group_ticks: u32) {
    let mut i = 0;
    while i < evs.len() {
        let Some(tuplet) = evs[i].beat.duration.valid_tuplet() else {
            i += 1;
            continue;
        };
        let start = i;
        while i < evs.len() && evs[i].beat.duration.valid_tuplet() == Some(tuplet) {
            let end = evs[i].onset + evs[i].beat.duration.ticks();
            i += 1;
            if end.is_multiple_of(group_ticks) {
                break;
            }
        }
        draw_tuplet_number(items, &evs[start..i], tuplet.actual);
    }
}

fn draw_tuplet_number(items: &mut Vec<Item>, group: &[Event], actual: u8) {
    let size = MUSIC * TUPLET_SCALE;
    let (first, last) = (&group[0], &group[group.len() - 1]);
    let edges = group.iter().map(|e| e.articulations().1);
    let y = if first.up {
        edges.fold(0.0, f32::min) - TUPLET_GAP_ABOVE
    } else {
        edges.fold(STAFF5_H, f32::max) + TUPLET_GAP_BELOW
    };
    let text = digits(TUPLET_0, actual as u32);
    let w = fonts::text_width(FontId::Music, size, &text);
    let x = (first.x + last.x + last.head_w) / 2.0 - w / 2.0;
    items.push(Item::Text { x, y, size, font: FontId::Music, text });
}

fn flag_glyph(v: NoteValue, up: bool) -> Option<char> {
    Some(match (v, up) {
        (NoteValue::Eighth, true) => FLAG_8TH_UP,
        (NoteValue::Eighth, false) => FLAG_8TH_DOWN,
        (NoteValue::Sixteenth, true) => FLAG_16TH_UP,
        (NoteValue::Sixteenth, false) => FLAG_16TH_DOWN,
        (NoteValue::ThirtySecond, true) => FLAG_32ND_UP,
        (NoteValue::ThirtySecond, false) => FLAG_32ND_DOWN,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{Beat, Duration};

    /// Slash-notation events for `beats`, with onsets computed in order.
    fn events(beats: &[Beat]) -> Vec<Event<'_>> {
        let mut onset = 0;
        beats
            .iter()
            .map(|b| {
                let ev = Event::new(b, onset, onset as f32 / 100.0, true, Style::Slash);
                onset += b.duration.ticks();
                ev
            })
            .collect()
    }

    fn note(v: NoteValue) -> Beat {
        Beat::slash(Duration::new(v))
    }

    fn tuplet_count(beats: &[Beat], group_ticks: u32) -> usize {
        let mut items = Vec::new();
        draw_tuplets(&mut items, &events(beats), group_ticks);
        items.len()
    }

    #[test]
    fn eighths_beam_per_beat_and_rests_break_beams() {
        use NoteValue::Eighth;
        let beats = [
            note(Eighth),
            note(Eighth),
            note(Eighth),
            Beat::rest(Duration::new(Eighth)),
            note(Eighth),
        ];
        assert_eq!(beam_groups(&events(&beats), 960), vec![vec![0, 1], vec![2], vec![4]]);
    }

    #[test]
    fn note_across_the_beat_gets_its_own_flag() {
        use NoteValue::*;
        let beats = [note(Eighth), Beat::slash(Duration::dotted(Eighth)), note(Sixteenth)];
        // The dotted 8th starts at 480 and ends at 1200, crossing beat 2.
        assert_eq!(beam_groups(&events(&beats), 960), vec![vec![0], vec![1], vec![2]]);
    }

    #[test]
    fn compound_meter_beams_three_eighths() {
        let beats: Vec<Beat> = (0..6).map(|_| note(NoteValue::Eighth)).collect();
        assert_eq!(beam_groups(&events(&beats), 1440), vec![vec![0, 1, 2], vec![3, 4, 5]]);
    }

    #[test]
    fn mixed_triplet_in_one_beat_has_one_number() {
        // Triplet 8th, 16th, 16th, 8th = one beat.
        let t = |v| Beat::slash(Duration::triplet(v));
        let beats = [
            t(NoteValue::Eighth),
            t(NoteValue::Sixteenth),
            t(NoteValue::Sixteenth),
            t(NoteValue::Eighth),
        ];
        assert_eq!(tuplet_count(&beats, 960), 1);
    }

    #[test]
    fn two_beats_of_triplets_have_two_numbers() {
        let beats: Vec<Beat> =
            (0..6).map(|_| Beat::slash(Duration::triplet(NoteValue::Eighth))).collect();
        assert_eq!(tuplet_count(&beats, 960), 2);
    }

    #[test]
    fn invalid_tuplet_draws_nothing() {
        let mut d = Duration::new(NoteValue::Eighth);
        d.tuplet = Some(model::Tuplet { actual: 3, normal: 0 });
        assert_eq!(tuplet_count(&[Beat::slash(d)], 960), 0);
    }

    #[test]
    fn dotted_eighth_sixteenth_gets_stub() {
        // Dotted 8th (1 beam) + 16th (2 beams): second level is a left stub.
        let stems = [(0.0, 1), (5.0, 2)];
        assert_eq!(beam_segments(&stems, 0), vec![(0.0, 5.0)]);
        assert_eq!(beam_segments(&stems, 1), vec![(5.0 - BEAM_STUB, 5.0)]);
    }

    #[test]
    fn sixteenths_share_both_levels() {
        let stems = [(0.0, 2), (3.0, 2), (6.0, 2)];
        assert_eq!(beam_segments(&stems, 1), vec![(0.0, 6.0)]);
    }
}
