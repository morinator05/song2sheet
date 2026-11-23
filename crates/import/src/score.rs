//! An imported score and extracting sections from it.

use std::ops::Range;

use model::{Segment, Staff, TimeSig, edit};

/// A whole imported score: one staff per part, all with the same bars.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Score {
    pub title: String,
    pub composer: String,
    pub parts: Vec<Part>,
    /// Time signature, tempo and number of every bar.
    pub bars: Vec<BarInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub name: String,
    /// Kind is guessed from the instrument; bars match [`Score::bars`].
    pub staff: Staff,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BarInfo {
    pub time_sig: TimeSig,
    /// Tempo in effect at the start of the bar.
    pub tempo: Option<u32>,
    /// Bar number as printed in the source score (may differ from the
    /// position, e.g. with a pickup bar); empty if unknown.
    pub number: String,
}

impl BarInfo {
    /// The bar's number for display: as in the score, else its position.
    pub fn label(&self, index: usize) -> String {
        if self.number.is_empty() { (index + 1).to_string() } else { self.number.clone() }
    }
}

/// A section made of `bars` of the parts with the given indices. Takes the
/// tempo and time signature of the first bar.
pub fn extract(score: &Score, bars: Range<usize>, parts: &[usize], name: &str) -> Segment {
    let bars = bars.start.min(score.bars.len())..bars.end.min(score.bars.len());
    let info = score.bars.get(bars.start).cloned().unwrap_or_default();
    let staves = parts
        .iter()
        .filter_map(|&i| score.parts.get(i))
        .map(|p| Staff {
            kind: p.staff.kind,
            label: p.staff.label.clone(),
            tuning: p.staff.tuning.clone(),
            bars: p.staff.bars.get(bars.clone()).unwrap_or_default().to_vec(),
        })
        .collect();
    let mut seg = Segment {
        name: name.to_string(),
        tempo: info.tempo,
        time_sig: info.time_sig,
        staves,
        ..Default::default()
    };
    edit::normalize(&mut seg);
    seg
}
