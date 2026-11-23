//! The project (one song) and its sheet.

use serde::{Deserialize, Serialize};

use crate::{FORMAT_VERSION, Staff, StaffKind, TimeSig};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Project {
    pub version: u32,
    pub meta: Meta,
    pub segments: Vec<Segment>,
    pub sheet: Vec<SheetEntry>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: FORMAT_VERSION,
            meta: Meta::default(),
            segments: Vec::new(),
            sheet: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    Json(serde_json::Error),
    UnsupportedVersion(u32),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Json(e) => write!(f, "invalid song sheet file: {e}"),
            LoadError::UnsupportedVersion(v) => write!(
                f,
                "file format version {v} is newer than this app supports ({FORMAT_VERSION})"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

impl Project {
    pub fn from_json(json: &str) -> Result<Self, LoadError> {
        let project: Project = serde_json::from_str(json).map_err(LoadError::Json)?;
        if project.version > FORMAT_VERSION {
            return Err(LoadError::UnsupportedVersion(project.version));
        }
        Ok(project)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("project serialization cannot fail")
    }

    pub fn segment(&self, id: SegmentId) -> Option<&Segment> {
        self.segments.iter().find(|s| s.id == id)
    }

    pub fn segment_mut(&mut self, id: SegmentId) -> Option<&mut Segment> {
        self.segments.iter_mut().find(|s| s.id == id)
    }

    /// A fresh id not used by any segment in this project.
    pub fn next_segment_id(&self) -> SegmentId {
        SegmentId(self.segments.iter().map(|s| s.id.0).max().map_or(1, |m| m + 1))
    }

    /// Adds a segment, assigning it a fresh id, and returns that id.
    pub fn add_segment(&mut self, mut segment: Segment) -> SegmentId {
        segment.id = self.next_segment_id();
        let id = segment.id;
        self.segments.push(segment);
        id
    }

    /// Copies a segment into an independent new segment (named "… (copy)").
    pub fn duplicate_segment(&mut self, id: SegmentId) -> Option<SegmentId> {
        let mut copy = self.segment(id)?.clone();
        copy.name.push_str(" (copy)");
        Some(self.add_segment(copy))
    }

    /// Removes a segment and every sheet entry that uses it.
    pub fn remove_segment(&mut self, id: SegmentId) {
        self.segments.retain(|s| s.id != id);
        self.sheet.retain(|e| e.segment != id);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Meta {
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub subtitle: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub artist: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub key: String,
    /// Default tempo in BPM, used for segments without their own tempo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<u32>,
    /// Free text printed under the header.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
)]
#[serde(transparent)]
pub struct SegmentId(pub u64);

/// A short excerpt of notation (a few bars) for one song section.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Segment {
    pub id: SegmentId,
    pub name: String,
    /// Tempo in BPM; `None` falls back to [`Meta::tempo`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<u32>,
    pub time_sig: TimeSig,
    pub staves: Vec<Staff>,
}

impl Segment {
    pub fn bar_count(&self) -> usize {
        self.staves.iter().map(|s| s.bars.len()).max().unwrap_or(0)
    }
}

/// One row of the song sheet.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct SheetEntry {
    pub segment: SegmentId,
    /// Overrides the segment name on the sheet if non-empty (e.g. "Verse 2").
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    /// How often the section is played; > 1 draws repeat signs and "×N".
    pub repeat: u32,
    /// Comment for everyone.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub comment: String,
    /// Comments only shown on the sheet of one part.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub part_comments: Vec<PartComment>,
}

impl Default for SheetEntry {
    fn default() -> Self {
        Self {
            segment: SegmentId::default(),
            label: String::new(),
            repeat: 1,
            comment: String::new(),
            part_comments: Vec::new(),
        }
    }
}

impl SheetEntry {
    pub fn new(segment: SegmentId) -> Self {
        Self { segment, ..Default::default() }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct PartComment {
    pub kind: StaffKind,
    pub text: String,
}

/// Which parts to show when rendering a sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PartFilter {
    #[default]
    All,
    Only(StaffKind),
}

impl PartFilter {
    pub fn shows(self, kind: StaffKind) -> bool {
        match self {
            PartFilter::All => true,
            PartFilter::Only(k) => k == kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bar, Beat, DrumHit, Duration, LoadError, NoteValue};

    #[test]
    fn json_roundtrip_and_segment_ops() {
        let mut p = Project::default();
        p.meta.title = "Song".into();
        let mut seg = Segment { name: "Verse".into(), tempo: Some(120), ..Default::default() };
        let mut staff = Staff::new(StaffKind::Drums);
        staff.bars.push(Bar::single(vec![Beat::drums(
            Duration::new(NoteValue::Whole),
            &[DrumHit::Kick, DrumHit::Crash],
        )]));
        seg.staves.push(staff);
        let a = p.add_segment(seg);
        let b = p.duplicate_segment(a).unwrap();
        assert_ne!(a, b);
        p.sheet.push(SheetEntry::new(a));
        p.sheet.push(SheetEntry::new(b));
        p.sheet.push(SheetEntry::new(a));

        let back = Project::from_json(&p.to_json()).unwrap();
        assert_eq!(back, p);

        p.remove_segment(a);
        assert_eq!(p.sheet.len(), 1);
        assert_eq!(p.segments.len(), 1);
    }

    #[test]
    fn missing_repeat_defaults_to_one() {
        let p = Project::from_json(r#"{"sheet": [{"segment": 1}]}"#).unwrap();
        assert_eq!(p.sheet[0].repeat, 1);
    }

    #[test]
    fn fixture_survives_save_and_load() {
        let json = include_str!("../../../tests/fixtures/demo.s2s.json");
        let p = Project::from_json(json).unwrap();
        assert_eq!(Project::from_json(&p.to_json()).unwrap(), p);
        // Optional fields really are used by the fixture.
        let beats = || p.segments.iter().flat_map(|s| &s.staves).flat_map(|st| st.beats());
        assert!(beats().any(|b| b.duration.dots > 0));
        assert!(beats().any(|b| b.duration.tuplet.is_some()));
        assert!(beats().any(|b| b.accent));
        assert!(beats().any(|b| b.chord.is_some()));
    }

    #[test]
    fn rejects_newer_version() {
        let json = format!("{{\"version\": {}}}", FORMAT_VERSION + 1);
        assert!(matches!(Project::from_json(&json), Err(LoadError::UnsupportedVersion(_))));
    }

    #[test]
    fn missing_fields_default() {
        let p = Project::from_json("{\"meta\": {\"title\": \"X\"}}").unwrap();
        assert_eq!(p.meta.title, "X");
        assert!(p.segments.is_empty());
    }
}
