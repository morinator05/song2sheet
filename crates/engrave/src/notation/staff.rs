//! Draws one staff of one system: lines, clef, time signature, barlines,
//! repeat signs, the voices of every bar and chord symbols.

use model::{Bar, Beat, Segment, Staff, TimeSig};

use super::beams;
use super::events::Event;
use super::metrics::*;
use super::systems::{BeatBox, SysBar, System};
use crate::display::{Item, bbox_of};
use crate::fonts::{self, FontId};
use crate::smufl::*;

/// Items for `staff` in `sys`, with the top staff line at y = 0.
pub fn engrave_staff(
    staff: &Staff,
    style: Style,
    seg: &Segment,
    sys: &System,
    repeat: bool,
) -> Vec<Item> {
    let mut items = Vec::new();
    draw_frame(&mut items, style, seg, sys, repeat);

    let mut chords = Vec::new();
    for sb in &sys.bars {
        match staff.bars.get(sb.index) {
            Some(bar) => {
                draw_bar(&mut items, bar, sb, style, seg.time_sig.beam_group_ticks());
                chords.extend(chord_symbols(bar, sb));
            }
            None => items.push(whole_bar_rest(sb, style)),
        }
        draw_barline(&mut items, sb, sys, style, repeat);
    }
    draw_chords(&mut items, chords);
    items
}

/// Clickable areas of every beat of `staff` in `sys`, in the same
/// coordinates as [`engrave_staff`]. Hidden (all-rest) voices are included
/// so an editor can still place a cursor in them.
pub fn beat_boxes(staff: &Staff, staff_index: usize, sys: &System, style: Style) -> Vec<BeatBox> {
    let mut boxes = Vec::new();
    for sb in &sys.bars {
        let Some(bar) = staff.bars.get(sb.index) else { continue };
        for (voice_index, voice) in bar.voices.iter().enumerate() {
            let starts: Vec<f32> = onsets(voice).map(|(onset, _)| sb.x_of(onset)).collect();
            for (beat, &x) in starts.iter().enumerate() {
                let next = starts.get(beat + 1).copied().unwrap_or(sb.end());
                boxes.push(BeatBox {
                    staff: staff_index,
                    bar: sb.index,
                    voice: voice_index,
                    beat,
                    x0: x - BEAT_BOX_LEAD,
                    x1: next - BEAT_BOX_LEAD,
                    y0: 0.0,
                    y1: style.height(),
                });
            }
        }
    }
    boxes
}

/// Staff lines, clef, and on the first system the time signature and the
/// opening repeat sign.
fn draw_frame(items: &mut Vec<Item>, style: Style, seg: &Segment, sys: &System, repeat: bool) {
    for y in style.line_ys() {
        items.push(line(0.0, y, sys.end_x, y, LINE_W));
    }
    let mid = style.height() / 2.0;
    let clef = match style {
        Style::Percussion => Some((CLEF_PERCUSSION, 1.0)),
        Style::Tab(strings) if strings <= 4 => Some((CLEF_TAB_4, TAB4_CLEF_SCALE)),
        Style::Tab(_) => Some((CLEF_TAB_6, 1.0)),
        Style::Slash => None,
    };
    if let Some((c, scale)) = clef {
        items.push(music_scaled(CLEF_X, centered_baseline(c, mid, scale), c, scale));
    }
    if !sys.first {
        return;
    }
    let [num, den] = time_sig_digits(seg.time_sig);
    let w_num = fonts::text_width(FontId::Music, MUSIC, &num);
    let w_den = fonts::text_width(FontId::Music, MUSIC, &den);
    let w = w_num.max(w_den);
    for (text, tw, y) in
        [(num, w_num, mid - TIME_DIGIT_OFFSET), (den, w_den, mid + TIME_DIGIT_OFFSET)]
    {
        let x = CLEF_W + (w - tw) / 2.0;
        items.push(Item::Text { x, y, size: MUSIC, font: FontId::Music, text });
    }
    if repeat {
        items.extend(repeat_sign(CLEF_W + time_sig_width(seg.time_sig) + REPEAT_GAP, style, true));
    }
}

fn time_sig_digits(ts: TimeSig) -> [String; 2] {
    [digits(TIME_SIG_0, ts.num as u32), digits(TIME_SIG_0, ts.den as u32)]
}

/// Horizontal room taken by the time signature, including padding.
pub fn time_sig_width(ts: TimeSig) -> f32 {
    let widths = time_sig_digits(ts).map(|d| fonts::text_width(FontId::Music, MUSIC, &d));
    widths[0].max(widths[1]) + TIME_SIG_PAD
}

/// Draws all voices of a bar. Voice 0 has stems up, others down (tab: all
/// down). Voices without notes are hidden; a bar without notes gets a
/// centred whole rest.
fn draw_bar(items: &mut Vec<Item>, bar: &Bar, sb: &SysBar, style: Style, group_ticks: u32) {
    let active: Vec<(usize, &Vec<Beat>)> =
        bar.voices.iter().enumerate().filter(|(_, v)| v.iter().any(|b| !b.is_rest())).collect();
    if active.is_empty() {
        items.push(whole_bar_rest(sb, style));
        return;
    }
    let multi_voice = active.len() > 1;
    let mid = style.height() / 2.0;
    for (vi, voice) in active {
        let up = vi == 0 && !matches!(style, Style::Tab(_));
        // With two voices, rests move out of each other's way.
        let rest_y = match (multi_voice, up) {
            (false, _) => mid,
            (true, true) => mid - REST_SHIFT_UP,
            (true, false) => mid + REST_SHIFT_DOWN,
        };
        let mut evs: Vec<Event> = onsets(voice)
            .map(|(onset, beat)| Event::new(beat, onset, sb.x_of(onset), up, style))
            .collect();
        let groups = beams::beam_groups(&evs, group_ticks);
        beams::align_stems(&mut evs, &groups);
        for ev in &evs {
            ev.draw(items, style, rest_y);
        }
        beams::draw_beams(items, &evs, &groups);
        beams::draw_tuplets(items, &evs, group_ticks);
    }
}

/// Beats of a voice paired with their onset tick within the bar.
fn onsets(voice: &[Beat]) -> impl Iterator<Item = (u32, &Beat)> {
    voice.iter().scan(0u32, |t, b| {
        let onset = *t;
        *t += b.duration.ticks();
        Some((onset, b))
    })
}

struct ChordSymbol {
    x: f32,
    text: String,
}

fn chord_symbols(bar: &Bar, sb: &SysBar) -> Vec<ChordSymbol> {
    bar.voices
        .iter()
        .flat_map(|v| onsets(v))
        .filter_map(|(onset, beat)| {
            beat.chord.as_ref().map(|c| ChordSymbol { x: sb.x_of(onset), text: c.clone() })
        })
        .collect()
}

/// Chord symbols on one baseline above everything else on the staff,
/// nudged right where they would collide.
fn draw_chords(items: &mut Vec<Item>, mut chords: Vec<ChordSymbol>) {
    if chords.is_empty() {
        return;
    }
    chords.sort_by(|a, b| a.x.total_cmp(&b.x));
    let y = bbox_of(items).map_or(0.0, |b| b.y0).min(0.0) - CHORD_GAP;
    let mut min_x = f32::MIN;
    for ChordSymbol { x, text } in chords {
        let x = x.max(min_x);
        min_x = x + fonts::text_width(FontId::Bold, CHORD_SIZE, &text) + CHORD_GAP;
        items.push(Item::Text { x, y, size: CHORD_SIZE, font: FontId::Bold, text });
    }
}

fn whole_bar_rest(sb: &SysBar, style: Style) -> Item {
    let x = sb.x + sb.width() / 2.0 - glyph_w(REST_WHOLE) / 2.0;
    music(x, style.height() / 2.0 - WHOLE_REST_RAISE, REST_WHOLE)
}

fn draw_barline(items: &mut Vec<Item>, sb: &SysBar, sys: &System, style: Style, repeat: bool) {
    let x = sb.end();
    if repeat && sys.is_final_bar(sb) {
        items.extend(repeat_sign(x, style, false));
    } else {
        items.push(line(x, 0.0, x, style.height(), BAR_W));
    }
}

/// Repeat barline: `start` draws |: beginning at x, otherwise :| ending at x.
fn repeat_sign(x: f32, style: Style, start: bool) -> Vec<Item> {
    let h = style.height();
    let mid = h / 2.0;
    let (thick_x, thin_x, dot_x) = if start {
        (x, x + THICK_W + REPEAT_THIN_X, x + THICK_W + REPEAT_DOT_X)
    } else {
        (x - THICK_W, x - THICK_W - REPEAT_THIN_X, x - THICK_W - REPEAT_DOT_X - glyph_w(AUG_DOT))
    };
    vec![
        rect(thick_x, 0.0, THICK_W, h),
        line(thin_x, 0.0, thin_x, h, BAR_W),
        music(dot_x, mid - HALF_SPACE, AUG_DOT),
        music(dot_x, mid + HALF_SPACE, AUG_DOT),
    ]
}
