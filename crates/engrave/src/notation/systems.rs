//! Horizontal layout: measuring bars and packing them into systems.

use std::collections::BTreeSet;
use std::ops::Range;

use model::{NoteValue, Segment, Staff};

use super::metrics::*;
use super::staff::{engrave_staff, time_sig_width};
use crate::display::{Item, bbox_of};

/// Space after a 32nd note …
const GAP_32ND: f32 = 1.3 * SP;
/// … plus this much for every doubling of the duration.
const GAP_PER_DOUBLING: f32 = 0.9 * SP;
/// Extra space after a dotted note.
const DOT_GAP: f32 = 0.8 * SP;

/// Horizontal space after an onset that lasts `ticks` until the next onset.
fn gap(ticks: u32) -> f32 {
    let shortest = NoteValue::ThirtySecond.ticks() as f32;
    let doublings = (ticks as f32 / shortest).max(0.5).log2();
    GAP_32ND + GAP_PER_DOUBLING * doublings
}

/// Onset columns of one bar, shared by all staves of a system so that
/// simultaneous beats line up vertically.
#[derive(Clone, Debug)]
pub struct Columns {
    onsets: Vec<u32>,
    offsets: Vec<f32>,
    width: f32,
}

impl Columns {
    fn new(staves: &[&Staff], bar: usize, bar_ticks: u32) -> Self {
        let mut onsets = BTreeSet::from([0u32]);
        let mut dotted = BTreeSet::new();
        let mut end = bar_ticks;
        for voice in staves.iter().filter_map(|s| s.bars.get(bar)).flat_map(|b| &b.voices) {
            let mut t = 0;
            for beat in voice {
                onsets.insert(t);
                if beat.duration.dots > 0 {
                    dotted.insert(t);
                }
                t += beat.duration.ticks();
            }
            end = end.max(t);
        }
        let onsets: Vec<u32> = onsets.into_iter().collect();
        let mut offsets = Vec::with_capacity(onsets.len());
        let mut x = PAD_L;
        for (i, &t) in onsets.iter().enumerate() {
            offsets.push(x);
            let next = onsets.get(i + 1).copied().unwrap_or(end);
            x += gap(next.saturating_sub(t));
            if dotted.contains(&t) {
                x += DOT_GAP;
            }
        }
        Self { onsets, offsets, width: x + PAD_R }
    }

    fn offset(&self, tick: u32) -> f32 {
        match self.onsets.binary_search(&tick) {
            Ok(i) => self.offsets[i],
            Err(i) => self.offsets[i.saturating_sub(1)],
        }
    }
}

/// A bar placed in a system.
#[derive(Clone, Debug)]
pub struct SysBar<'a> {
    pub index: usize,
    pub x: f32,
    scale: f32,
    cols: &'a Columns,
}

impl SysBar<'_> {
    pub fn width(&self) -> f32 {
        self.cols.width * self.scale
    }

    pub fn end(&self) -> f32 {
        self.x + self.width()
    }

    /// x of the beat starting at `tick` within this bar.
    pub fn x_of(&self, tick: u32) -> f32 {
        self.x + self.cols.offset(tick) * self.scale
    }
}

/// One line of music.
#[derive(Clone, Debug)]
pub struct System<'a> {
    pub bars: Vec<SysBar<'a>>,
    pub first: bool,
    pub last: bool,
    pub end_x: f32,
}

impl System<'_> {
    pub fn is_final_bar(&self, bar: &SysBar) -> bool {
        self.last && self.bars.last().is_some_and(|b| b.index == bar.index)
    }
}

/// Result of engraving a segment: items relative to (0, 0) = top-left of
/// the notation area.
#[derive(Clone, Debug, Default)]
pub struct Block {
    pub items: Vec<Item>,
    pub height: f32,
    /// One label per staff drawn, for the caller to place left of it.
    pub staff_labels: Vec<StaffLabel>,
}

#[derive(Clone, Debug)]
pub struct StaffLabel {
    /// Vertical centre of the staff.
    pub y: f32,
    pub text: String,
}

/// Engraves `staves` (all from `seg`) into systems that fit `width`.
/// `repeat` draws repeat barlines around the whole segment.
pub fn engrave_segment(seg: &Segment, staves: &[&Staff], width: f32, repeat: bool) -> Block {
    let bar_count = staves.iter().map(|s| s.bars.len()).max().unwrap_or(0);
    if bar_count == 0 {
        return Block::default();
    }
    let bar_ticks = seg.time_sig.bar_ticks();
    let mut cols: Vec<Columns> =
        (0..bar_count).map(|i| Columns::new(staves, i, bar_ticks)).collect();
    if repeat {
        // Room for the closing repeat sign inside the last bar.
        if let Some(last) = cols.last_mut() {
            last.width += REPEAT_W;
        }
    }
    let first_prefix = CLEF_W + time_sig_width(seg.time_sig) + if repeat { REPEAT_W } else { 0.0 };
    let prefix = |first: bool| if first { first_prefix } else { CLEF_W };
    let systems = place_systems(&cols, width, prefix);
    let styles: Vec<Style> = staves.iter().map(|s| Style::of(s)).collect();

    let mut block = Block::default();
    let mut y = 0.0;
    for sys in &systems {
        for (staff, &style) in staves.iter().zip(&styles) {
            let mut items = engrave_staff(staff, style, seg, sys, repeat);
            let (top, bottom) = bbox_of(&items).map_or((0.0, 0.0), |b| (b.y0, b.y1));
            let dy = y - top.min(0.0);
            items.iter_mut().for_each(|it| it.translate(0.0, dy));
            block.items.extend(items);
            let label = StaffLabel {
                y: dy + style.height() / 2.0,
                text: staff.display_label().to_string(),
            };
            block.staff_labels.push(label);
            y = dy + bottom.max(style.height()) + STAFF_GAP;
        }
        y += SYSTEM_GAP - STAFF_GAP;
    }
    block.height = (y - SYSTEM_GAP).max(0.0);
    block
}

/// Splits bars into systems and assigns x positions and stretch factors.
fn place_systems(cols: &[Columns], width: f32, prefix: impl Fn(bool) -> f32) -> Vec<System<'_>> {
    let ranges = pack(cols, width, &prefix);
    let n = ranges.len();
    let mut systems = Vec::with_capacity(n);
    let mut prev_scale = f32::MAX;
    for (si, range) in ranges.into_iter().enumerate() {
        let (first, last) = (si == 0, si + 1 == n);
        let start = prefix(first);
        let natural: f32 = cols[range.clone()].iter().map(|c| c.width).sum();
        let mut scale = ((width - start) / natural).min(MAX_BAR_W * range.len() as f32 / natural);
        if last && n > 1 {
            // Don't stretch a short last system more than the others.
            scale = scale.min(prev_scale);
        }
        prev_scale = scale;
        let mut x = start;
        let bars = range
            .map(|index| {
                let bar = SysBar { index, x, scale, cols: &cols[index] };
                x = bar.end();
                bar
            })
            .collect();
        systems.push(System { bars, first, last, end_x: x });
    }
    systems
}

/// Finds the number of systems greedily (bars may shrink to `MIN_SCALE`),
/// then spreads the bars evenly over that many systems if they still fit.
fn pack(cols: &[Columns], width: f32, prefix: &impl Fn(bool) -> f32) -> Vec<Range<usize>> {
    let fits = |range: Range<usize>, first: bool| {
        prefix(first) + cols[range].iter().map(|c| c.width * MIN_SCALE).sum::<f32>() <= width
    };
    let count = cols.len();
    let mut greedy: Vec<Range<usize>> = Vec::new();
    let mut start = 0;
    for i in 1..=count {
        if i == count || !fits(start..i + 1, greedy.is_empty()) {
            greedy.push(start..i);
            start = i;
        }
    }
    let n = greedy.len();
    let balanced: Vec<Range<usize>> =
        (0..n).map(|k| (k * count).div_ceil(n)..((k + 1) * count).div_ceil(n)).collect();
    let balanced_fits =
        balanced.iter().enumerate().all(|(k, r)| !r.is_empty() && fits(r.clone(), k == 0));
    if balanced_fits { balanced } else { greedy }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cols(widths: &[f32]) -> Vec<Columns> {
        widths
            .iter()
            .map(|&width| Columns { onsets: vec![0], offsets: vec![PAD_L], width })
            .collect()
    }

    fn sizes(ranges: &[Range<usize>]) -> Vec<usize> {
        ranges.iter().map(|r| r.len()).collect()
    }

    #[test]
    fn spreads_bars_evenly_over_systems() {
        // 5 bars of 40 fit 3 per line greedily (3+2); balanced is also 3+2.
        assert_eq!(sizes(&pack(&cols(&[40.0; 5]), 100.0, &|_| 0.0)), [3, 2]);
        // 4 bars: greedy 3+1, balanced 2+2.
        assert_eq!(sizes(&pack(&cols(&[40.0; 4]), 100.0, &|_| 0.0)), [2, 2]);
    }

    #[test]
    fn bar_wider_than_the_line_gets_its_own_system() {
        assert_eq!(sizes(&pack(&cols(&[500.0, 10.0]), 100.0, &|_| 0.0)), [1, 1]);
    }

    #[test]
    fn bars_shrink_before_wrapping() {
        // Natural width 4 × 30 = 120 > 100, but 4 × 30 × MIN_SCALE fits.
        assert_eq!(sizes(&pack(&cols(&[30.0; 4]), 100.0, &|_| 0.0)), [4]);
    }

    #[test]
    fn longer_notes_get_more_space() {
        assert!(gap(960) > gap(480));
        assert!(gap(480) > gap(240));
        assert!(gap(1) > 0.0);
    }
}
