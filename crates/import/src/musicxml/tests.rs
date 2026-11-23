use model::{Beat, DrumHit, NoteValue, StaffKind, TabNote, TimeSig, Tuplet};

use super::*;
use crate::extract;

const GROOVE: &str = include_str!("../../../../tests/fixtures/groove.musicxml");
const GROOVE_MXL: &[u8] = include_bytes!("../../../../tests/fixtures/groove.mxl");
const TAB: &str = include_str!("../../../../tests/fixtures/tab.musicxml");

fn total(v: &[Beat]) -> u32 {
    v.iter().map(|b| b.duration.ticks()).sum()
}

#[test]
fn reads_musescore_export() {
    let score = parse(GROOVE).unwrap();
    assert_eq!(score.title, "Test Groove");
    assert_eq!(score.composer, "Test Band");
    let kinds: Vec<StaffKind> = score.parts.iter().map(|p| p.staff.kind).collect();
    assert_eq!(kinds, [StaffKind::Drums, StaffKind::Guitar, StaffKind::Bass]);
    assert_eq!(score.bars.len(), 4);
    let tempos: Vec<Option<u32>> = score.bars.iter().map(|b| b.tempo).collect();
    assert_eq!(tempos, [Some(120), Some(120), Some(140), Some(140)]);
    for part in &score.parts {
        assert_eq!(part.staff.bars.len(), 4, "{}", part.name);
        for bar in &part.staff.bars {
            assert!(
                bar.voices.iter().all(|v| total(v) == TimeSig::default().bar_ticks()),
                "{}",
                part.name
            );
        }
    }
}

#[test]
fn drums_keep_both_voices_and_instruments() {
    let score = parse(GROOVE).unwrap();
    let bar = &score.parts[0].staff.bars[0];
    let hands = &bar.voices[0];
    assert_eq!(hands.len(), 8);
    assert!(hands.iter().all(|b| b.drums.contains(&DrumHit::HiHatClosed)));
    assert!(hands[2].drums.contains(&DrumHit::Snare) && hands[6].drums.contains(&DrumHit::Snare));
    let feet = &bar.voices[1];
    let kicks = feet.iter().filter(|b| b.drums == [DrumHit::Kick]).count();
    assert_eq!(kicks, 3);
}

#[test]
fn guitar_gets_chord_symbols_and_slashes() {
    let score = parse(GROOVE).unwrap();
    let guitar = &score.parts[1].staff;
    let chords: Vec<Option<String>> =
        guitar.bars.iter().map(|b| b.voices[0][0].chord.clone()).collect();
    assert_eq!(chords, [Some("Am".into()), Some("F".into()), Some("C".into()), Some("G7".into())]);
    assert!(
        guitar.bars[0].voices[0].iter().all(|b| b.slash && b.duration.value == NoteValue::Quarter)
    );
}

#[test]
fn compressed_mxl_matches_plain_file() {
    assert_eq!(parse_mxl(GROOVE_MXL).unwrap(), parse(GROOVE).unwrap());
}

#[test]
fn reads_tab_tuplets_accents_and_skips_grace_notes() {
    let score = parse(TAB).unwrap();
    let staff = &score.parts[0].staff;
    assert_eq!(staff.kind, StaffKind::Guitar);
    assert_eq!(score.bars[0].time_sig, TimeSig { num: 3, den: 4 });
    let voice = &staff.bars[0].voices[0];
    assert_eq!(total(voice), TimeSig { num: 3, den: 4 }.bar_ticks());
    assert_eq!(voice[0].chord.as_deref(), Some("Bbmaj7/F"));
    assert_eq!(voice[0].tab, [TabNote { string: 5, fret: 1 }, TabNote { string: 4, fret: 3 }]);
    assert!(voice[0].accent);
    assert_eq!(voice[1].tab, [TabNote { string: 5, fret: 12 }], "grace note skipped");
    assert_eq!(voice[1].duration.tuplet, Some(Tuplet::TRIPLET));
    assert_eq!(voice[1].duration.dots, 1);
}

#[test]
fn extract_takes_bars_parts_and_tempo() {
    let score = parse(GROOVE).unwrap();
    let seg = extract(&score, 2..4, &[0, 2], "Chorus");
    assert_eq!(seg.name, "Chorus");
    assert_eq!(seg.tempo, Some(140));
    assert_eq!(seg.staves.len(), 2);
    assert_eq!(seg.staves[1].kind, StaffKind::Bass);
    assert!(seg.staves.iter().all(|s| s.bars.len() == 2));
}

#[test]
fn rejects_non_scores() {
    assert!(parse("<html/>").is_err());
    assert!(parse("not xml").is_err());
}

/// A one-part score with the given measures (raw MusicXML).
fn one_part(name: &str, measures: &str) -> String {
    format!(
        r#"<score-partwise><part-list><score-part id="P1"><part-name>{name}</part-name></score-part></part-list><part id="P1">{measures}</part></score-partwise>"#
    )
}

const ATTRS: &str = "<attributes><divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>";

fn quarter_note(staff: u32, fret: Option<(u8, u8)>) -> String {
    let tech = fret.map_or(String::new(), |(s, f)| {
        format!(
            "<notations><technical><string>{s}</string><fret>{f}</fret></technical></notations>"
        )
    });
    format!(
        "<note><pitch><step>A</step><octave>3</octave></pitch><duration>1</duration><voice>1</voice><type>quarter</type><staff>{staff}</staff>{tech}</note>"
    )
}

#[test]
fn absurd_durations_neither_panic_nor_hang() {
    let huge = one_part(
        "X",
        &format!(
            r#"<measure number="1">{ATTRS}<note><rest/><duration>4000000000</duration></note><note><rest/><duration>4000000000</duration></note></measure>"#
        ),
    );
    let odd = one_part(
        "X",
        r#"<measure number="1"><attributes><divisions>7</divisions></attributes><note><rest/><duration>700001</duration></note></measure>"#,
    );
    for xml in [huge, odd] {
        let start = std::time::Instant::now();
        let score = parse(&xml).unwrap();
        assert!(start.elapsed().as_secs() < 2);
        assert_eq!(total(&score.parts[0].staff.bars[0].voices[0]), TimeSig::default().bar_ticks());
    }
}

#[test]
fn chords_from_the_notation_staff_reach_the_tab_staff() {
    // Guitar with notation (staff 1) and linked tab (staff 2), as MuseScore writes it.
    let chord = r#"<harmony><root><root-step>E</root-step></root><kind>minor</kind></harmony>"#;
    let notation: String = (0..4).map(|_| quarter_note(1, None)).collect();
    let tab: String = (0..4).map(|_| quarter_note(2, Some((5, 0)))).collect();
    let xml = one_part(
        "Guitar",
        &format!(
            r#"<measure number="1">{ATTRS}{chord}{notation}<backup><duration>4</duration></backup>{tab}</measure>"#
        ),
    );
    let staff = &parse(&xml).unwrap().parts[0].staff;
    assert!(staff.has_tab());
    assert_eq!(staff.bars[0].voices[0][0].chord.as_deref(), Some("Em"));
}

#[test]
fn pickup_bar_notes_sit_at_the_end_and_numbers_are_kept() {
    let pickup = r#"<measure number="0" implicit="yes"><attributes><divisions>1</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes><note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>1</duration><type>quarter</type></note></measure>"#;
    let full = r#"<measure number="1"><note><rest/><duration>4</duration><type>whole</type></note></measure>"#;
    let score = parse(&one_part("Drums", &format!("{pickup}{full}"))).unwrap();
    let voice = &score.parts[0].staff.bars[0].voices[0];
    assert!(voice.last().unwrap().drums.contains(&DrumHit::Snare), "upbeat at the end");
    assert!(voice[0].is_rest());
    assert_eq!(score.bars[0].label(0), "0");
    assert_eq!(score.bars[1].label(1), "1");
}

#[test]
fn parts_of_different_length_are_evened_out() {
    let bar = |n: u32| {
        format!(
            r#"<measure number="{n}"><note><rest/><duration>4</duration><type>whole</type></note></measure>"#
        )
    };
    let xml = format!(
        r#"<score-partwise><part-list><score-part id="A"><part-name>a</part-name></score-part><score-part id="B"><part-name>b</part-name></score-part></part-list><part id="A">{ATTRS}{}</part><part id="B">{}{}{}</part></score-partwise>"#,
        bar(1),
        bar(1),
        bar(2),
        bar(3)
    );
    let score = parse(&xml).unwrap();
    assert_eq!(score.bars.len(), 3);
    assert!(score.parts.iter().all(|p| p.staff.bars.len() == 3));
}

#[test]
fn tempo_from_metronome_mark_and_bad_tempos_ignored() {
    let metronome = r#"<direction><direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>96</per-minute></metronome></direction-type></direction>"#;
    let bad = r#"<direction><sound tempo="-5"/></direction>"#;
    let rest = r#"<note><rest/><duration>4</duration><type>whole</type></note>"#;
    let xml = one_part(
        "X",
        &format!(
            r#"<measure number="1">{ATTRS}{metronome}{rest}</measure><measure number="2">{bad}{rest}</measure>"#
        ),
    );
    let tempos: Vec<Option<u32>> = parse(&xml).unwrap().bars.iter().map(|b| b.tempo).collect();
    assert_eq!(tempos, [Some(96), Some(96)]);
}

#[test]
fn chord_kinds_and_degrees() {
    let harmony = |inner: &str| {
        let doc = roxmltree::Document::parse(inner).unwrap();
        harmony::chord_text(doc.root_element())
    };
    let c13 =
        r#"<harmony><root><root-step>C</root-step></root><kind>dominant-13th</kind></harmony>"#;
    assert_eq!(harmony(c13).as_deref(), Some("C13"));
    let add9 = r#"<harmony><root><root-step>G</root-step></root><kind>major</kind><degree><degree-value>9</degree-value><degree-alter>0</degree-alter><degree-type>add</degree-type></degree></harmony>"#;
    assert_eq!(harmony(add9).as_deref(), Some("Gadd9"));
    let b9 = r#"<harmony><root><root-step>A</root-step></root><kind>dominant</kind><degree><degree-value>9</degree-value><degree-alter>-1</degree-alter><degree-type>alter</degree-type></degree></harmony>"#;
    assert_eq!(harmony(b9).as_deref(), Some("A7b9"));
}

#[test]
fn reads_utf16_files() {
    let xml = one_part("X", "");
    let mut bytes = UTF16_LE_BOM.to_vec();
    bytes.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
    assert_eq!(decode(&bytes).unwrap(), xml);
}
