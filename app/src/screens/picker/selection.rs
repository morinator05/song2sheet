//! Pure parts of the picker: overview chunks and the bar selection.

use std::ops::Range;

use import::BarInfo;

/// Most bars shown in one overview image (keeps textures small).
pub const MAX_CHUNK_BARS: usize = 8;

/// Splits the bars into overview chunks of at most [`MAX_CHUNK_BARS`],
/// starting a new chunk wherever the time signature changes (a section
/// has a single time signature).
pub fn chunks(bars: &[BarInfo]) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    for (i, bar) in bars.iter().enumerate() {
        match out.last_mut() {
            Some(c) if c.len() < MAX_CHUNK_BARS && bars[c.start].time_sig == bar.time_sig => {
                c.end = i + 1
            }
            _ => out.push(i..i + 1),
        }
    }
    out
}

/// "Bars 5–8" (or "Bar 5") with the bar numbers printed in the score.
pub fn bar_range_label(bars: &[BarInfo], range: &Range<usize>) -> String {
    let label = |i: usize| bars.get(i).map_or_else(|| (i + 1).to_string(), |b| b.label(i));
    let last = range.end.saturating_sub(1);
    if range.len() <= 1 {
        format!("Bar {}", label(range.start))
    } else {
        format!("Bars {}–{}", label(range.start), label(last))
    }
}

/// Selected bars: a click starts at a bar, Shift+click extends from there.
#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    anchor: usize,
    pub bars: Range<usize>,
}

impl Selection {
    pub fn new(bar: usize) -> Self {
        Self { anchor: bar, bars: bar..bar + 1 }
    }

    /// Extends from the anchor to `bar` (in either direction).
    pub fn extend_to(&mut self, bar: usize) {
        self.bars = self.anchor.min(bar)..self.anchor.max(bar) + 1;
    }

    /// The selected bars inside `chunk`, relative to its start.
    pub fn within(&self, chunk: &Range<usize>) -> Range<usize> {
        let start = self.bars.start.clamp(chunk.start, chunk.end);
        let end = self.bars.end.clamp(chunk.start, chunk.end);
        start - chunk.start..end - chunk.start
    }

    /// Whether the selection spans several time signatures.
    pub fn mixes_meters(&self, bars: &[BarInfo]) -> bool {
        let sigs = bars.get(self.bars.clone()).unwrap_or_default();
        sigs.windows(2).any(|w| w[0].time_sig != w[1].time_sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::TimeSig;

    fn bars(sigs: &[(u8, u8)]) -> Vec<BarInfo> {
        sigs.iter()
            .map(|&(num, den)| BarInfo { time_sig: TimeSig { num, den }, ..Default::default() })
            .collect()
    }

    #[test]
    fn chunks_split_at_size_and_meter_changes() {
        let mut b = bars(&[(4, 4); 10]);
        assert_eq!(chunks(&b), vec![0..8, 8..10]);
        b[3].time_sig = TimeSig { num: 7, den: 8 };
        assert_eq!(chunks(&b), vec![0..3, 3..4, 4..10]);
        assert!(chunks(&[]).is_empty());
    }

    #[test]
    fn labels_use_score_numbers() {
        let mut b = bars(&[(4, 4); 3]);
        assert_eq!(bar_range_label(&b, &(0..3)), "Bars 1–3");
        b[0].number = "0".into();
        assert_eq!(bar_range_label(&b, &(0..1)), "Bar 0");
    }

    #[test]
    fn selection_extends_both_ways() {
        let mut s = Selection::new(5);
        s.extend_to(2);
        assert_eq!(s.bars, 2..6);
        s.extend_to(7);
        assert_eq!(s.bars, 5..8);
    }

    #[test]
    fn selection_within_a_chunk_is_relative() {
        let mut s = Selection::new(6);
        s.extend_to(9);
        assert_eq!(s.within(&(0..8)), 6..8);
        assert_eq!(s.within(&(8..16)), 0..2);
        assert!(s.within(&(16..20)).is_empty());
    }

    #[test]
    fn detects_mixed_meters() {
        let b = bars(&[(4, 4), (4, 4), (7, 8)]);
        let mut s = Selection::new(0);
        s.extend_to(1);
        assert!(!s.mixes_meters(&b));
        s.extend_to(2);
        assert!(s.mixes_meters(&b));
    }
}
