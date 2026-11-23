//! Staves, bars and beats.

use serde::{Deserialize, Serialize};

use crate::{DrumHit, Duration};

/// Which player a staff is for. Used for the part filter on export and to
/// pick a notation style.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StaffKind {
    #[default]
    Drums,
    Guitar,
    Bass,
    Keys,
    Vocals,
    Other,
}

impl StaffKind {
    pub const ALL: [StaffKind; 6] = [
        StaffKind::Drums,
        StaffKind::Guitar,
        StaffKind::Bass,
        StaffKind::Keys,
        StaffKind::Vocals,
        StaffKind::Other,
    ];

    pub fn name(self) -> &'static str {
        match self {
            StaffKind::Drums => "Drums",
            StaffKind::Guitar => "Guitar",
            StaffKind::Bass => "Bass",
            StaffKind::Keys => "Keys",
            StaffKind::Vocals => "Vocals",
            StaffKind::Other => "Other",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            StaffKind::Drums => "Dr",
            StaffKind::Guitar => "Gtr",
            StaffKind::Bass => "Bass",
            StaffKind::Keys => "Keys",
            StaffKind::Vocals => "Voc",
            StaffKind::Other => "",
        }
    }

    /// Standard tuning (highest string first) for tab staves of this kind.
    pub fn default_tuning(self) -> Vec<Pitch> {
        let midi: &[u8] = match self {
            StaffKind::Bass => &[43, 38, 33, 28],
            _ => &[64, 59, 55, 50, 45, 40],
        };
        midi.iter().map(|&m| Pitch(m)).collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Staff {
    pub kind: StaffKind,
    /// Optional display label; empty uses [`StaffKind::short_name`].
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// String tuning for tab, highest string first. Empty = default for kind.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tuning: Vec<Pitch>,
    pub bars: Vec<Bar>,
}

impl Staff {
    pub fn new(kind: StaffKind) -> Self {
        Self { kind, ..Default::default() }
    }

    pub fn display_label(&self) -> &str {
        if self.label.is_empty() { self.kind.short_name() } else { &self.label }
    }

    pub fn tuning(&self) -> Vec<Pitch> {
        if self.tuning.is_empty() { self.kind.default_tuning() } else { self.tuning.clone() }
    }

    /// Whether any beat carries tab (string/fret) data.
    pub fn has_tab(&self) -> bool {
        self.beats().any(|b| !b.tab.is_empty())
    }

    pub fn beats(&self) -> impl Iterator<Item = &Beat> {
        self.bars.iter().flat_map(|b| b.voices.iter().flatten())
    }
}

/// MIDI pitch number.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct Pitch(pub u8);

/// One bar. Each voice is a sequence of beats filling the bar. Drums use
/// voice 0 for hands (stems up) and voice 1 for feet (stems down); other
/// staves normally have a single voice.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Bar {
    pub voices: Vec<Vec<Beat>>,
}

impl Bar {
    pub fn single(beats: Vec<Beat>) -> Self {
        Self { voices: vec![beats] }
    }
}

/// A rhythmic event: a rest, or one or more simultaneous notes.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Beat {
    pub duration: Duration,
    /// Drum hits (drum staves).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub drums: Vec<DrumHit>,
    /// Tab notes (guitar/bass staves).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tab: Vec<TabNote>,
    /// Chord symbol shown above this beat, e.g. "Am7".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chord: Option<String>,
    /// A rhythm-only note (slash notehead) on non-drum staves without tab.
    #[serde(skip_serializing_if = "crate::is_false")]
    pub slash: bool,
    #[serde(skip_serializing_if = "crate::is_false")]
    pub accent: bool,
}

impl Beat {
    pub fn rest(duration: Duration) -> Self {
        Self { duration, ..Default::default() }
    }

    pub fn drums(duration: Duration, hits: &[DrumHit]) -> Self {
        Self { duration, drums: hits.to_vec(), ..Default::default() }
    }

    pub fn slash(duration: Duration) -> Self {
        Self { duration, slash: true, ..Default::default() }
    }

    pub fn with_chord(mut self, chord: &str) -> Self {
        self.chord = Some(chord.to_string());
        self
    }

    pub fn is_rest(&self) -> bool {
        self.drums.is_empty() && self.tab.is_empty() && !self.slash
    }
}

/// A fretted note. `string` is 1-based, 1 = highest string.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(default)]
pub struct TabNote {
    pub string: u8,
    pub fret: u8,
}
