//! Durations, tuplets and time signatures.

use serde::{Deserialize, Serialize};

use crate::{TICKS_PER_QUARTER, TICKS_PER_WHOLE};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct TimeSig {
    pub num: u8,
    pub den: u8,
}

impl Default for TimeSig {
    fn default() -> Self {
        Self { num: 4, den: 4 }
    }
}

impl TimeSig {
    /// Most beats per bar supported (and offered by the editor).
    pub const MAX_NUM: u8 = 16;
    /// Beat units supported (and offered by the editor). Other units can't
    /// always be filled exactly with rests.
    pub const DENOMINATORS: [u8; 4] = [2, 4, 8, 16];

    pub fn is_supported(self) -> bool {
        (1..=Self::MAX_NUM).contains(&self.num) && Self::DENOMINATORS.contains(&self.den)
    }

    /// The nearest supported time signature (4/4 for unknown units).
    pub fn supported(self) -> TimeSig {
        let num = self.num.clamp(1, Self::MAX_NUM);
        let den =
            if Self::DENOMINATORS.contains(&self.den) { self.den } else { Self::default().den };
        TimeSig { num, den }
    }

    pub fn bar_ticks(self) -> u32 {
        TICKS_PER_WHOLE * self.num as u32 / self.den.max(1) as u32
    }

    /// Ticks of one beaming group: a dotted quarter in compound meters
    /// (3/8, 6/8, 9/8, 12/8), otherwise one beat.
    pub fn beam_group_ticks(self) -> u32 {
        let beat = TICKS_PER_WHOLE / self.den.max(1) as u32;
        if self.den == 8 && self.num.is_multiple_of(3) {
            beat * 3
        } else if self.den == 8 {
            // Other meters in /8 (5/8, 7/8): beam per quarter.
            beat * 2
        } else {
            beat
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub enum NoteValue {
    Whole,
    Half,
    #[default]
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
}

impl NoteValue {
    pub const ALL: [NoteValue; 6] = [
        NoteValue::Whole,
        NoteValue::Half,
        NoteValue::Quarter,
        NoteValue::Eighth,
        NoteValue::Sixteenth,
        NoteValue::ThirtySecond,
    ];

    pub fn ticks(self) -> u32 {
        match self {
            NoteValue::Whole => TICKS_PER_WHOLE,
            NoteValue::Half => TICKS_PER_QUARTER * 2,
            NoteValue::Quarter => TICKS_PER_QUARTER,
            NoteValue::Eighth => TICKS_PER_QUARTER / 2,
            NoteValue::Sixteenth => TICKS_PER_QUARTER / 4,
            NoteValue::ThirtySecond => TICKS_PER_QUARTER / 8,
        }
    }

    /// Number of beams (or flags) for this value.
    pub fn beams(self) -> u8 {
        match self {
            NoteValue::Eighth => 1,
            NoteValue::Sixteenth => 2,
            NoteValue::ThirtySecond => 3,
            _ => 0,
        }
    }

    pub fn has_stem(self) -> bool {
        self != NoteValue::Whole
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Tuplet {
    /// Notes played …
    pub actual: u8,
    /// … in the time of this many.
    pub normal: u8,
}

impl Tuplet {
    pub const TRIPLET: Tuplet = Tuplet { actual: 3, normal: 2 };

    /// Tuplets with a zero count are meaningless and treated as absent.
    pub fn is_valid(self) -> bool {
        self.actual > 0 && self.normal > 0
    }
}

impl Default for Tuplet {
    fn default() -> Self {
        Self::TRIPLET
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(default)]
pub struct Duration {
    pub value: NoteValue,
    #[serde(skip_serializing_if = "crate::is_zero")]
    pub dots: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuplet: Option<Tuplet>,
}

impl Duration {
    pub const fn new(value: NoteValue) -> Self {
        Self { value, dots: 0, tuplet: None }
    }

    pub const fn dotted(value: NoteValue) -> Self {
        Self { value, dots: 1, tuplet: None }
    }

    pub const fn triplet(value: NoteValue) -> Self {
        Self { value, dots: 0, tuplet: Some(Tuplet::TRIPLET) }
    }

    /// The tuplet, if any and valid.
    pub fn tuplet(self) -> Option<Tuplet> {
        self.tuplet.filter(|t| t.is_valid())
    }

    /// The tuplet, if any and valid.
    pub fn valid_tuplet(self) -> Option<Tuplet> {
        self.tuplet.filter(|t| t.is_valid())
    }

    pub fn ticks(self) -> u32 {
        let base = self.value.ticks();
        let mut t = base;
        let mut add = base;
        for _ in 0..self.dots {
            add /= 2;
            t += add;
        }
        match self.valid_tuplet() {
            Some(tp) => t * tp.normal as u32 / tp.actual as u32,
            None => t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_ticks() {
        assert_eq!(Duration::new(NoteValue::Quarter).ticks(), 960);
        assert_eq!(Duration::dotted(NoteValue::Eighth).ticks(), 720);
        assert_eq!(Duration::triplet(NoteValue::Eighth).ticks(), 320);
        assert_eq!(
            Duration { value: NoteValue::Half, dots: 2, tuplet: None }.ticks(),
            960 * 2 + 960 + 480
        );
    }

    #[test]
    fn time_sig_ticks() {
        assert_eq!(TimeSig { num: 4, den: 4 }.bar_ticks(), 3840);
        assert_eq!(TimeSig { num: 6, den: 8 }.bar_ticks(), 2880);
        assert_eq!(TimeSig { num: 6, den: 8 }.beam_group_ticks(), 1440);
        assert_eq!(TimeSig { num: 3, den: 4 }.beam_group_ticks(), 960);
        assert_eq!(TimeSig { num: 3, den: 8 }.beam_group_ticks(), 1440);
        assert_eq!(TimeSig { num: 7, den: 8 }.beam_group_ticks(), 960);
    }

    #[test]
    fn invalid_tuplet_is_ignored() {
        let d = Duration {
            tuplet: Some(Tuplet { actual: 3, normal: 0 }),
            ..Duration::new(NoteValue::Quarter)
        };
        assert_eq!(d.valid_tuplet(), None);
        assert_eq!(d.ticks(), 960);
    }
}
