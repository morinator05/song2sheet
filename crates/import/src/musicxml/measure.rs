//! Walking the measures of one MusicXML part.
//!
//! Positions are tracked in MusicXML "divisions" (per quarter note) and
//! converted to ticks. `<backup>`/`<forward>` move the position; gaps in a
//! voice become rests, and every voice is fitted to its bar. Chord symbols
//! are kept by position and attached to the chosen staff at the end, since
//! they may sit on another staff of the part (e.g. notation above tab).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use model::edit::{self, DEFAULT_REST};
use model::{Bar, Beat, Staff, TICKS_PER_QUARTER, TimeSig};
use roxmltree::Node;

use super::{child, child_number, child_text, harmony, notes};
use crate::BarInfo;

/// MusicXML clef sign of percussion staves.
const PERCUSSION_CLEF: &str = "percussion";
/// Staff and voice numbers default to 1 when a note gives none.
const FIRST: u32 = 1;
/// The only metronome beat unit read as BPM (others need conversion).
const QUARTER_BEAT_UNIT: &str = "quarter";

pub struct PartData {
    pub staff: Staff,
    pub bars: Vec<BarInfo>,
    /// The part uses a percussion clef or unpitched notes.
    pub percussion: bool,
}

/// One parsed measure.
struct Measure {
    info: BarInfo,
    /// A fitted bar per MusicXML staff number.
    bars: BTreeMap<u32, Bar>,
    /// Chord symbols by tick within the bar.
    chords: Vec<(u32, String)>,
}

/// One voice being built: beats so far and the position they reach.
#[derive(Default)]
struct VoiceBuild {
    beats: Vec<Beat>,
    filled_to: u32,
}

/// State carried from measure to measure.
struct PartState {
    divisions: u32,
    time_sig: TimeSig,
    tempo: Option<u32>,
    percussion: bool,
}

impl PartState {
    /// Divisions to ticks, saturating instead of overflowing.
    fn ticks(&self, divisions: u32) -> u32 {
        let ticks = divisions as u64 * TICKS_PER_QUARTER as u64 / self.divisions.max(1) as u64;
        ticks.min(u32::MAX as u64) as u32
    }

    fn bar_ticks(&self) -> u32 {
        self.time_sig.bar_ticks()
    }
}

pub fn parse_part(part: Node, drums: &HashMap<String, u8>) -> PartData {
    let mut state =
        PartState { divisions: 1, time_sig: TimeSig::default(), tempo: None, percussion: false };
    let measures: Vec<Measure> = part
        .children()
        .filter(|c| c.has_tag_name("measure"))
        .map(|m| parse_measure(m, &mut state, drums))
        .collect();
    let staff_no = pick_staff(&measures);
    let bars = measures
        .iter()
        .map(|m| {
            let mut bar = m
                .bars
                .get(&staff_no)
                .cloned()
                .unwrap_or_else(|| edit::empty_bar(1, m.info.time_sig.bar_ticks()));
            attach_chords(&mut bar, &m.chords);
            bar
        })
        .collect();
    PartData {
        staff: Staff { bars, ..Default::default() },
        bars: measures.into_iter().map(|m| m.info).collect(),
        percussion: state.percussion,
    }
}

/// The staff to keep: the first one with tab (it has the frets), else the
/// lowest-numbered one.
fn pick_staff(measures: &[Measure]) -> u32 {
    let numbers: BTreeSet<u32> = measures.iter().flat_map(|m| m.bars.keys().copied()).collect();
    let has_tab = |no: &u32| {
        measures
            .iter()
            .filter_map(|m| m.bars.get(no))
            .flat_map(|b| b.voices.iter().flatten())
            .any(|b| !b.tab.is_empty())
    };
    numbers.iter().copied().find(has_tab).or(numbers.first().copied()).unwrap_or(FIRST)
}

fn attach_chords(bar: &mut Bar, chords: &[(u32, String)]) {
    if let Some(voice) = bar.voices.first_mut().filter(|v| !v.is_empty()) {
        for (tick, text) in chords {
            let i = edit::beat_at(voice, *tick);
            voice[i].chord = Some(text.clone());
        }
    }
}

fn parse_measure(measure: Node, state: &mut PartState, drums: &HashMap<String, u8>) -> Measure {
    let tempo_before = state.tempo;
    let mut voices: BTreeMap<(u32, u32), VoiceBuild> = BTreeMap::new();
    let mut chords = Vec::new();
    let mut pos = 0u32;
    for el in measure.children().filter(Node::is_element) {
        match el.tag_name().name() {
            "attributes" => read_attributes(el, state),
            "direction" | "sound" => {
                if let Some(bpm) = tempo(el) {
                    state.tempo = Some(bpm);
                }
            }
            "harmony" => {
                if let Some(text) = harmony::chord_text(el) {
                    chords.push((state.ticks(pos), text));
                }
            }
            "backup" => pos = pos.saturating_sub(duration_divs(el)),
            "forward" => pos = pos.saturating_add(duration_divs(el)),
            "note" => read_note(el, state, drums, &mut voices, &mut pos),
            _ => {}
        }
    }

    let bar_ticks = state.bar_ticks();
    // A pickup bar ("implicit") is short: its notes belong at the end.
    let content =
        voices.values().map(|v| state.ticks(v.filled_to)).max().unwrap_or(0).min(bar_ticks);
    let lead_in =
        if measure.attribute("implicit") == Some("yes") { bar_ticks - content } else { 0 };
    let mut bars: BTreeMap<u32, Bar> = BTreeMap::new();
    for ((staff, _), v) in voices {
        let mut beats = edit::rests(lead_in, DEFAULT_REST);
        beats.extend(v.beats);
        edit::fit(&mut beats, bar_ticks);
        bars.entry(staff).or_default().voices.push(beats);
    }
    let chords = chords.into_iter().map(|(t, c)| (t + lead_in, c)).collect();
    let number = measure.attribute("number").unwrap_or_default().to_string();
    let info = BarInfo { time_sig: state.time_sig, tempo: state.tempo.or(tempo_before), number };
    Measure { info, bars, chords }
}

fn read_note(
    el: Node,
    state: &mut PartState,
    drums: &HashMap<String, u8>,
    voices: &mut BTreeMap<(u32, u32), VoiceBuild>,
    pos: &mut u32,
) {
    if child(el, "grace").is_some() || child(el, "cue").is_some() {
        return;
    }
    state.percussion |= notes::is_unpitched(el);
    let key =
        (child_number(el, "staff").unwrap_or(FIRST), child_number(el, "voice").unwrap_or(FIRST));
    let voice = voices.entry(key).or_default();
    if child(el, "chord").is_some() {
        if let Some(last) = voice.beats.last_mut() {
            notes::add_to_beat(last, el, drums);
        }
        return;
    }
    // Every length below is clamped to the bar: `fit` cuts the rest anyway.
    let bar_ticks = state.bar_ticks();
    if *pos > voice.filled_to {
        let gap = state.ticks(*pos - voice.filled_to).min(bar_ticks);
        voice.beats.extend(edit::rests(gap, DEFAULT_REST));
    }
    let written = duration_divs(el);
    let mut beats = match notes::duration(el) {
        Some(d) => vec![Beat::rest(d)],
        // No type (whole-bar rest): rests for the written time.
        None => edit::rests(state.ticks(written).min(bar_ticks), DEFAULT_REST),
    };
    if let Some(first) = beats.first_mut()
        && child(el, "rest").is_none()
    {
        notes::add_to_beat(first, el, drums);
    }
    voice.beats.append(&mut beats);
    *pos = pos.saturating_add(written);
    voice.filled_to = *pos;
}

/// The `<duration>` of a note, backup or forward, in divisions.
fn duration_divs(el: Node) -> u32 {
    child_number(el, "duration").unwrap_or(0)
}

/// Tempo (BPM) from a `<sound tempo>` or a quarter-note metronome mark.
fn tempo(el: Node) -> Option<u32> {
    let sound = if el.has_tag_name("sound") { Some(el) } else { child(el, "sound") };
    let from_sound = sound.and_then(|s| s.attribute("tempo")).and_then(|t| t.parse::<f32>().ok());
    let from_metronome = || {
        let metronome = el.descendants().find(|n| n.has_tag_name("metronome"))?;
        (child_text(metronome, "beat-unit")? == QUARTER_BEAT_UNIT).then_some(())?;
        child_text(metronome, "per-minute")?.parse::<f32>().ok()
    };
    let bpm = from_sound.or_else(from_metronome)?;
    (bpm.is_finite() && bpm >= 1.0).then(|| bpm.round() as u32)
}

fn read_attributes(attributes: Node, state: &mut PartState) {
    if let Some(d) = child_number(attributes, "divisions") {
        state.divisions = d;
    }
    if let Some(time) = child(attributes, "time")
        && let (Some(num), Some(den)) =
            (child_number(time, "beats"), child_number(time, "beat-type"))
    {
        // Unsupported meters (e.g. 3/1) are mapped to the nearest supported one.
        state.time_sig = TimeSig { num, den }.supported();
    }
    let mut clefs = attributes.children().filter(|c| c.has_tag_name("clef"));
    if clefs.any(|c| child_text(c, "sign") == Some(PERCUSSION_CLEF)) {
        state.percussion = true;
    }
}
