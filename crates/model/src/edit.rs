//! Editing operations on voices (sequences of beats filling one bar).
//!
//! A voice always fills its bar exactly: changing a beat's duration takes
//! time from (or gives time to) the following beats, which are filled up
//! with rests.

use crate::{Bar, Beat, Duration, NoteValue, Segment, Tuplet};

/// Rest value used to fill bars when no other duration is in play
/// (whole rests where they fit, so as few rests as possible).
pub const DEFAULT_REST: Duration = Duration::new(NoteValue::Whole);

/// Rests filling exactly `ticks`: as many rests of `preferred` as allow
/// an exact fill, then as few other rests as possible, longest first.
pub fn rests(ticks: u32, preferred: Duration) -> Vec<Beat> {
    let step = preferred.ticks();
    let max_preferred = ticks.checked_div(step).unwrap_or(0);
    for count in (0..=max_preferred).rev() {
        if let Some(rest) = exact_fill(ticks - count * step) {
            let mut out = vec![Beat::rest(preferred); count as usize];
            out.extend(rest.into_iter().map(Beat::rest));
            return out;
        }
    }
    // Only possible with unsupported meters (see `TimeSig::is_supported`).
    Vec::new()
}

/// Every duration a filler rest may have: plain, dotted and triplet values.
fn rest_values() -> Vec<Duration> {
    let mut v: Vec<Duration> = NoteValue::ALL
        .iter()
        .flat_map(|&value| {
            let plain = Duration::new(value);
            [plain, Duration::dotted(value), Duration { tuplet: Some(Tuplet::TRIPLET), ..plain }]
        })
        .collect();
    v.sort_by_key(|d| std::cmp::Reverse(d.ticks()));
    v
}

/// Rests summing to exactly `ticks` (longest first), if possible. Tries
/// the quick greedy fill first and searches for the fewest rests only when
/// greedy gets stuck.
fn exact_fill(ticks: u32) -> Option<Vec<Duration>> {
    let values = rest_values();
    let mut greedy = Vec::new();
    let mut left = ticks;
    while let Some(d) = values.iter().find(|d| d.ticks() <= left && d.ticks() > 0) {
        greedy.push(*d);
        left -= d.ticks();
    }
    if left == 0 {
        return Some(greedy);
    }
    let n = ticks as usize;
    // For every tick count up to `ticks`: fewest rests and the last one used.
    let mut count = vec![u32::MAX; n + 1];
    let mut last: Vec<Option<Duration>> = vec![None; n + 1];
    count[0] = 0;
    for t in 1..=n {
        for d in &values {
            let dt = d.ticks() as usize;
            if dt <= t && count[t - dt] != u32::MAX && count[t - dt] + 1 < count[t] {
                count[t] = count[t - dt] + 1;
                last[t] = Some(*d);
            }
        }
    }
    let mut out = Vec::new();
    let mut t = n;
    while t > 0 {
        let d = last[t]?;
        out.push(d);
        t -= d.ticks() as usize;
    }
    out.sort_by_key(|d| std::cmp::Reverse(d.ticks()));
    Some(out)
}

/// Onset tick of beat `index` within its voice.
pub fn onset(voice: &[Beat], index: usize) -> u32 {
    voice[..index.min(voice.len())].iter().map(|b| b.duration.ticks()).sum()
}

/// Makes `voice` fill exactly `bar_ticks`: beats beyond the bar are cut
/// (the one crossing the barline becomes rests up to it), missing time
/// is padded with rests.
pub fn fit(voice: &mut Vec<Beat>, bar_ticks: u32) {
    let mut t = 0;
    let mut keep = 0;
    for b in voice.iter() {
        let end = t + b.duration.ticks();
        if end > bar_ticks {
            break;
        }
        t = end;
        keep += 1;
    }
    voice.truncate(keep);
    voice.extend(rests(bar_ticks - t, DEFAULT_REST));
}

/// Changes the duration of beat `index`, keeping its content. Following
/// beats that would overlap are removed and the gap up to the next kept
/// beat is filled with rests of `duration`. Returns `false` (and changes
/// nothing) if the beat would cross the barline.
pub fn set_duration(
    voice: &mut Vec<Beat>,
    index: usize,
    duration: Duration,
    bar_ticks: u32,
) -> bool {
    if index >= voice.len() {
        return false;
    }
    let start = onset(voice, index);
    let end = start + duration.ticks();
    if end > bar_ticks || duration.ticks() == 0 {
        return false;
    }
    // Remove beats from `index` until the new beat's end is covered.
    let mut covered = start;
    let mut remove = 0;
    while index + remove < voice.len() && covered < end {
        covered += voice[index + remove].duration.ticks();
        remove += 1;
    }
    let gap = covered.saturating_sub(end);
    let fill = rests(gap, duration);
    if fill.iter().map(|b| b.duration.ticks()).sum::<u32>() != gap {
        return false;
    }
    let mut beat = voice[index].clone();
    beat.duration = duration;
    voice.splice(index..index + remove, std::iter::once(beat).chain(fill));
    true
}

/// Removes voices after the first that contain only rests. They are
/// invisible on the sheet and only appear while editing.
pub fn prune(project: &mut crate::Project) {
    for bar in project.segments.iter_mut().flat_map(|s| &mut s.staves).flat_map(|st| &mut st.bars) {
        while bar.voices.len() > 1 && bar.voices.last().is_some_and(|v| v.iter().all(Beat::is_rest))
        {
            bar.voices.pop();
        }
    }
}

/// Turns beat `index` into a rest of the same duration.
pub fn clear(voice: &mut [Beat], index: usize) {
    if let Some(b) = voice.get_mut(index) {
        *b = Beat::rest(b.duration);
    }
}

/// A bar with `voices` voices (at least one) of rests.
pub fn empty_bar(voices: usize, bar_ticks: u32) -> Bar {
    Bar { voices: (0..voices.max(1)).map(|_| rests(bar_ticks, DEFAULT_REST)).collect() }
}

/// Makes a segment editable: every staff has the same number of bars (at
/// least one), every bar at least one voice, and every voice fills its bar.
/// Returns whether anything changed.
pub fn normalize(seg: &mut Segment) -> bool {
    let before = seg.clone();
    seg.time_sig = seg.time_sig.supported();
    let bar_ticks = seg.time_sig.bar_ticks();
    let bars = seg.bar_count().max(1);
    for staff in &mut seg.staves {
        staff.bars.resize_with(bars, Bar::default);
        for bar in &mut staff.bars {
            if bar.voices.is_empty() {
                bar.voices.push(Vec::new());
            }
            for voice in &mut bar.voices {
                fit(voice, bar_ticks);
            }
        }
    }
    *seg != before
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DrumHit, TICKS_PER_QUARTER, TICKS_PER_WHOLE};

    const BAR: u32 = TICKS_PER_WHOLE;

    fn total(v: &[Beat]) -> u32 {
        v.iter().map(|b| b.duration.ticks()).sum()
    }

    fn eighth() -> Duration {
        Duration::new(NoteValue::Eighth)
    }

    #[test]
    fn rests_prefer_the_given_value() {
        let r = rests(BAR, eighth());
        assert_eq!(r.len(), 8);
        assert!(r.iter().all(|b| b.is_rest() && b.duration == eighth()));
    }

    #[test]
    fn every_duration_at_every_position_keeps_the_bar_full() {
        for d in rest_values() {
            for index in 0..8 {
                let mut v = rests(BAR, eighth());
                set_duration(&mut v, index, d, BAR);
                assert_eq!(total(&v), BAR, "{d:?} at {index}");
            }
        }
    }

    #[test]
    fn dotted_32nd_at_the_start_fits() {
        let mut v = rests(BAR, DEFAULT_REST);
        assert!(set_duration(&mut v, 0, Duration::dotted(NoteValue::ThirtySecond), BAR));
        assert_eq!(total(&v), BAR);
    }

    #[test]
    fn normalize_replaces_unsupported_meters() {
        use crate::{Staff, StaffKind, TimeSig};
        let mut staff = Staff::new(StaffKind::Drums);
        staff.bars.push(Bar::default());
        let mut seg = Segment {
            time_sig: TimeSig { num: 40, den: 5 },
            staves: vec![staff],
            ..Default::default()
        };
        normalize(&mut seg);
        assert!(seg.time_sig.is_supported());
        let bar_ticks = seg.time_sig.bar_ticks();
        assert_eq!(total(&seg.staves[0].bars[0].voices[0]), bar_ticks);
    }

    #[test]
    fn prune_drops_rest_only_extra_voices() {
        use crate::{Project, Segment, Staff, StaffKind};
        let mut staff = Staff::new(StaffKind::Drums);
        staff.bars.push(empty_bar(2, BAR));
        let mut p = Project::default();
        p.add_segment(Segment { staves: vec![staff], ..Default::default() });
        prune(&mut p);
        assert_eq!(p.segments[0].staves[0].bars[0].voices.len(), 1);
    }

    #[test]
    fn rests_fill_remainders_exactly() {
        for ticks in [0, 320, 480 + 320, 1000 - 40, BAR] {
            assert_eq!(
                total(&rests(ticks, Duration::dotted(NoteValue::Quarter))),
                ticks,
                "{ticks}"
            );
        }
    }

    #[test]
    fn shortening_a_beat_fills_with_current_duration() {
        let mut v = rests(BAR, Duration::new(NoteValue::Whole));
        v[0].drums.push(DrumHit::HiHatClosed);
        assert!(set_duration(&mut v, 0, eighth(), BAR));
        assert_eq!(v.len(), 8);
        assert_eq!(v[0].drums, [DrumHit::HiHatClosed]);
        assert_eq!(total(&v), BAR);
    }

    #[test]
    fn lengthening_a_beat_swallows_following_beats() {
        let mut v = rests(BAR, eighth());
        assert!(set_duration(&mut v, 1, Duration::new(NoteValue::Half), BAR));
        // 8th, half (480..2400), then 1440 ticks of rests.
        assert_eq!(onset(&v, 2), 2400);
        assert_eq!(total(&v), BAR);
    }

    #[test]
    fn cannot_cross_the_barline() {
        let mut v = rests(BAR, eighth());
        let before = v.clone();
        assert!(!set_duration(&mut v, 7, Duration::new(NoteValue::Quarter), BAR));
        assert_eq!(v, before);
    }

    #[test]
    fn triplets_fill_a_beat() {
        let mut v = rests(BAR, Duration::new(NoteValue::Quarter));
        assert!(set_duration(&mut v, 0, Duration::triplet(NoteValue::Eighth), BAR));
        assert_eq!(onset(&v, 3), TICKS_PER_QUARTER);
        assert_eq!(total(&v), BAR);
    }

    #[test]
    fn normalize_pads_staves_and_voices() {
        use crate::{Staff, StaffKind};
        let mut a = Staff::new(StaffKind::Drums);
        a.bars = vec![Bar::default(), Bar::default()];
        let b = Staff::new(StaffKind::Guitar);
        let mut seg = Segment { staves: vec![a, b], ..Default::default() };
        assert!(normalize(&mut seg));
        for staff in &seg.staves {
            assert_eq!(staff.bars.len(), 2);
            assert!(staff.bars.iter().all(|bar| total(&bar.voices[0]) == BAR));
        }
        assert!(!normalize(&mut seg), "already normal");
    }

    #[test]
    fn fit_cuts_and_pads() {
        let mut v = rests(BAR, eighth());
        fit(&mut v, 3 * TICKS_PER_QUARTER);
        assert_eq!(total(&v), 3 * TICKS_PER_QUARTER);
        fit(&mut v, BAR);
        assert_eq!(total(&v), BAR);
        let mut half = vec![Beat::rest(Duration::new(NoteValue::Whole))];
        fit(&mut half, 2 * TICKS_PER_QUARTER);
        assert_eq!(total(&half), 2 * TICKS_PER_QUARTER);
    }
}
