//! The editor cursor: which beat of which voice, bar and staff is selected.

use model::edit;
use model::{Beat, Segment};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub staff: usize,
    pub bar: usize,
    pub voice: usize,
    pub beat: usize,
    /// Selected tab string, 1 = highest.
    pub string: u8,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { staff: 0, bar: 0, voice: 0, beat: 0, string: 1 }
    }
}

impl Cursor {
    /// Pulls every index back into range for `seg` (which must be
    /// normalized, see [`model::edit::normalize`]).
    pub fn clamp(&mut self, seg: &Segment) {
        self.staff = self.staff.min(seg.staves.len().saturating_sub(1));
        let Some(staff) = seg.staves.get(self.staff) else { return };
        self.bar = self.bar.min(staff.bars.len().saturating_sub(1));
        let voices = staff.bars.get(self.bar).map_or(1, |b| b.voices.len());
        self.voice = self.voice.min(voices.saturating_sub(1));
        let beats = self.voice(seg).map_or(1, Vec::len);
        self.beat = self.beat.min(beats.saturating_sub(1));
        self.string = self.string.clamp(1, staff.tuning().len().max(1) as u8);
    }

    pub fn voice<'a>(&self, seg: &'a Segment) -> Option<&'a Vec<Beat>> {
        seg.staves.get(self.staff)?.bars.get(self.bar)?.voices.get(self.voice)
    }

    pub fn voice_mut<'a>(&self, seg: &'a mut Segment) -> Option<&'a mut Vec<Beat>> {
        seg.staves.get_mut(self.staff)?.bars.get_mut(self.bar)?.voices.get_mut(self.voice)
    }

    pub fn beat<'a>(&self, seg: &'a Segment) -> Option<&'a Beat> {
        self.voice(seg)?.get(self.beat)
    }

    pub fn beat_mut<'a>(&self, seg: &'a mut Segment) -> Option<&'a mut Beat> {
        self.voice_mut(seg)?.get_mut(self.beat)
    }

    /// Onset tick of the selected beat within its bar.
    pub fn tick(&self, seg: &Segment) -> u32 {
        self.voice(seg).map_or(0, |v| edit::onset(v, self.beat))
    }

    /// Next beat, continuing into the next bar.
    pub fn right(&mut self, seg: &mut Segment) {
        let len = self.voice(seg).map_or(0, Vec::len);
        if self.beat + 1 < len {
            self.beat += 1;
        } else if self.bar + 1 < seg.bar_count() {
            self.bar += 1;
            self.beat = 0;
            ensure_voice(seg, self);
        }
    }

    /// Previous beat, continuing into the previous bar.
    pub fn left(&mut self, seg: &mut Segment) {
        if self.beat > 0 {
            self.beat -= 1;
        } else if self.bar > 0 {
            self.bar -= 1;
            ensure_voice(seg, self);
            self.beat = self.voice(seg).map_or(0, |v| v.len().saturating_sub(1));
        }
    }

    /// Moves to `staff`/`voice`, selecting the beat sounding at the same
    /// time as the current one.
    pub fn switch(&mut self, seg: &mut Segment, staff: usize, voice: usize) {
        let tick = self.tick(seg);
        self.staff = staff.min(seg.staves.len().saturating_sub(1));
        self.voice = voice;
        ensure_voice(seg, self);
        self.beat = self.voice(seg).map_or(0, |v| beat_at(v, tick));
        self.clamp(seg);
    }
}

/// Index of the beat sounding at `tick`.
pub fn beat_at(voice: &[Beat], tick: u32) -> usize {
    let mut t = 0;
    for (i, b) in voice.iter().enumerate() {
        t += b.duration.ticks();
        if tick < t {
            return i;
        }
    }
    voice.len().saturating_sub(1)
}

/// Creates the cursor's voice (filled with rests) if the bar lacks it.
pub fn ensure_voice(seg: &mut Segment, cursor: &Cursor) {
    let bar_ticks = seg.time_sig.bar_ticks();
    if let Some(bar) = seg.staves.get_mut(cursor.staff).and_then(|s| s.bars.get_mut(cursor.bar)) {
        while bar.voices.len() <= cursor.voice {
            bar.voices.push(edit::rests(bar_ticks, edit::DEFAULT_REST));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{Duration, NoteValue, Staff, StaffKind};

    fn seg(bars: usize) -> Segment {
        let mut staff = Staff::new(StaffKind::Drums);
        staff.bars = (0..bars).map(|_| edit::empty_bar(1, 3840)).collect();
        let mut seg = Segment { staves: vec![staff.clone(), staff], ..Default::default() };
        // Bar 0 of staff 0: four quarter rests.
        seg.staves[0].bars[0].voices[0] = vec![Beat::rest(Duration::new(NoteValue::Quarter)); 4];
        seg
    }

    #[test]
    fn right_and_left_cross_barlines() {
        let mut s = seg(2);
        let mut c = Cursor { beat: 3, ..Default::default() };
        c.right(&mut s);
        assert_eq!((c.bar, c.beat), (1, 0));
        c.left(&mut s);
        assert_eq!((c.bar, c.beat), (0, 3));
    }

    #[test]
    fn right_stops_at_the_end() {
        let mut s = seg(1);
        let mut c = Cursor { beat: 3, ..Default::default() };
        c.right(&mut s);
        assert_eq!((c.bar, c.beat), (0, 3));
    }

    #[test]
    fn switching_voice_creates_it_and_keeps_the_time() {
        let mut s = seg(1);
        let mut c = Cursor { beat: 2, ..Default::default() };
        c.switch(&mut s, 0, 1);
        assert_eq!(s.staves[0].bars[0].voices.len(), 2);
        assert_eq!(c.voice, 1);
        // The new voice is one whole rest, sounding at the old beat's tick.
        assert_eq!(c.beat, 0);
        c.switch(&mut s, 0, 0);
        assert_eq!(c.beat, 0, "tick of the whole rest is 0");
    }

    #[test]
    fn beat_at_finds_the_sounding_beat() {
        let v = vec![Beat::rest(Duration::new(NoteValue::Quarter)); 4];
        assert_eq!(beat_at(&v, 0), 0);
        assert_eq!(beat_at(&v, 1000), 1);
        assert_eq!(beat_at(&v, 99999), 3);
    }
}
