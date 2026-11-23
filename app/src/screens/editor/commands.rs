//! Editing commands: what a key press or toolbar button does to the
//! section being edited. Pure functions of (segment, cursor, input state).

use model::edit::{self, empty_bar};
use model::{
    Beat, DrumHit, Duration, NoteValue, Segment, Staff, StaffKind, TabNote, TimeSig, Tuplet,
};

use super::cursor::Cursor;

/// Two fret digits typed within this many seconds form one number (e.g. 12).
const FRET_COMBINE_SECS: f64 = 0.8;
pub const MAX_FRET: u8 = 24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Set the note value (keeps the triplet setting, removes dots).
    Value(NoteValue),
    ToggleDot,
    ToggleTriplet,
    Longer,
    Shorter,
    Drum(DrumHit),
    /// Toggle a rhythm slash (non-drum staves without tab).
    Slash,
    /// A fret digit for the selected string.
    FretDigit(u8),
    /// Make the selected beat a rest (or remove the fret on the selected string).
    Clear,
    Accent,
    Left,
    Right,
    StringUp,
    StringDown,
    StaffUp,
    StaffDown,
    /// Drums: switch between hands (voice 1) and feet (voice 2).
    ToggleVoice,
    /// Copy the selected bar of this staff into the next bar.
    RepeatBar,
}

/// Why a command changed nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Refused {
    /// The chosen duration would cross the barline.
    DoesNotFit,
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refused::DoesNotFit => write!(f, "That duration doesn't fit in the bar here."),
        }
    }
}

/// State carried between commands.
#[derive(Clone, Debug)]
pub struct Input {
    /// Duration used to fill freed-up time.
    pub duration: Duration,
    /// Last fret typed: where, when and which, to combine two digits.
    last_fret: Option<(Cursor, f64, u8)>,
}

impl Default for Input {
    fn default() -> Self {
        Self { duration: Duration::new(NoteValue::Eighth), last_fret: None }
    }
}

/// Applies `cmd` at `now` (seconds, any monotonic clock).
pub fn apply(
    cmd: Command,
    seg: &mut Segment,
    cursor: &mut Cursor,
    input: &mut Input,
    now: f64,
) -> Result<(), Refused> {
    use Command::*;
    let mut result = Ok(());
    match cmd {
        Value(_) | ToggleDot | ToggleTriplet | Longer | Shorter => {
            result = change_duration(cmd, seg, cursor, input)
        }
        Drum(hit) => toggle_drum(seg, cursor, hit),
        Slash => edit_beat(seg, cursor, |b| b.slash = !b.slash),
        FretDigit(d) => type_fret(seg, cursor, input, d, now),
        Clear => clear(seg, cursor),
        Accent => edit_beat(seg, cursor, |b| b.accent = !b.accent && !b.is_rest()),
        Left => cursor.left(seg),
        Right => cursor.right(seg),
        StringUp => cursor.string = cursor.string.saturating_sub(1).max(1),
        StringDown => cursor.string += 1,
        StaffUp => cursor.switch(seg, cursor.staff.saturating_sub(1), 0),
        StaffDown => cursor.switch(seg, cursor.staff + 1, 0),
        ToggleVoice => cursor.switch(seg, cursor.staff, 1 - cursor.voice.min(1)),
        RepeatBar => repeat_bar(seg, cursor),
    }
    if !matches!(cmd, FretDigit(_)) {
        input.last_fret = None;
    }
    cursor.clamp(seg);
    result
}

fn edit_beat(seg: &mut Segment, cursor: &Cursor, f: impl FnOnce(&mut Beat)) {
    if let Some(b) = cursor.beat_mut(seg) {
        f(b);
    }
}

fn change_duration(
    cmd: Command,
    seg: &mut Segment,
    cursor: &Cursor,
    input: &mut Input,
) -> Result<(), Refused> {
    let bar_ticks = seg.time_sig.bar_ticks();
    let Some(current) = cursor.beat(seg).map(|b| b.duration) else { return Ok(()) };
    let new = match cmd {
        Command::Value(value) => Duration { value, dots: 0, tuplet: input.duration.valid_tuplet() },
        Command::ToggleDot => Duration { dots: if current.dots == 0 { 1 } else { 0 }, ..current },
        Command::ToggleTriplet => Duration {
            tuplet: if current.valid_tuplet().is_some() { None } else { Some(Tuplet::TRIPLET) },
            ..current
        },
        Command::Longer => Duration { value: step_value(current.value, -1), ..current },
        Command::Shorter => Duration { value: step_value(current.value, 1), ..current },
        _ => return Ok(()),
    };
    let Some(voice) = cursor.voice_mut(seg) else { return Ok(()) };
    if !edit::set_duration(voice, cursor.beat, new, bar_ticks) {
        return Err(Refused::DoesNotFit);
    }
    input.duration = new;
    Ok(())
}

/// Neighbouring note value; positive = shorter.
fn step_value(v: NoteValue, by: i32) -> NoteValue {
    let i = NoteValue::ALL.iter().position(|x| *x == v).expect("ALL lists every value") as i32;
    NoteValue::ALL[(i + by).clamp(0, NoteValue::ALL.len() as i32 - 1) as usize]
}

fn toggle_drum(seg: &mut Segment, cursor: &Cursor, hit: DrumHit) {
    edit_beat(seg, cursor, |b| {
        if let Some(i) = b.drums.iter().position(|d| *d == hit) {
            b.drums.remove(i);
        } else {
            b.drums.push(hit);
        }
        if b.is_rest() {
            b.accent = false;
        }
    });
}

fn type_fret(seg: &mut Segment, cursor: &Cursor, input: &mut Input, digit: u8, now: f64) {
    let fret = match input.last_fret {
        Some((at, time, prev))
            if at == *cursor && now - time < FRET_COMBINE_SECS && prev * 10 + digit <= MAX_FRET =>
        {
            prev * 10 + digit
        }
        _ => digit,
    };
    let string = cursor.string;
    edit_beat(seg, cursor, |b| {
        b.tab.retain(|n| n.string != string);
        b.tab.push(TabNote { string, fret });
        b.tab.sort_by_key(|n| n.string);
    });
    input.last_fret = Some((*cursor, now, fret));
}

fn clear(seg: &mut Segment, cursor: &Cursor) {
    let string = cursor.string;
    let on_string = cursor
        .beat(seg)
        .is_some_and(|b| b.tab.iter().any(|n| n.string == string) && b.tab.len() > 1);
    if on_string {
        edit_beat(seg, cursor, |b| b.tab.retain(|n| n.string != string));
    } else if let Some(voice) = cursor.voice_mut(seg) {
        edit::clear(voice, cursor.beat);
    }
}

fn repeat_bar(seg: &mut Segment, cursor: &mut Cursor) {
    let next = cursor.bar + 1;
    if next >= seg.bar_count() {
        add_bar(seg);
    }
    let Some(staff) = seg.staves.get_mut(cursor.staff) else { return };
    let Some(bar) = staff.bars.get(cursor.bar).cloned() else { return };
    if let Some(target) = staff.bars.get_mut(next) {
        *target = bar;
        cursor.bar = next;
        cursor.beat = 0;
    }
}

/// Appends a bar of rests to every staff.
pub fn add_bar(seg: &mut Segment) {
    let bar_ticks = seg.time_sig.bar_ticks();
    for staff in &mut seg.staves {
        staff.bars.push(empty_bar(1, bar_ticks));
    }
}

/// Removes the last bar of every staff (keeps at least one).
pub fn remove_last_bar(seg: &mut Segment) {
    if seg.bar_count() > 1 {
        for staff in &mut seg.staves {
            staff.bars.pop();
        }
    }
}

pub fn add_staff(seg: &mut Segment, kind: StaffKind) {
    let bar_ticks = seg.time_sig.bar_ticks();
    let bars = seg.bar_count().max(1);
    seg.staves.push(Staff {
        bars: (0..bars).map(|_| empty_bar(1, bar_ticks)).collect(),
        ..Staff::new(kind)
    });
}

/// Removes staff `index` (keeping at least one), and keeps the cursor on
/// the staff it was on.
pub fn remove_staff(seg: &mut Segment, cursor: &mut Cursor, index: usize) {
    if seg.staves.len() > 1 && index < seg.staves.len() {
        seg.staves.remove(index);
        if index < cursor.staff {
            cursor.staff -= 1;
        }
        cursor.clamp(seg);
    }
}

/// Changes the time signature (made supported, see
/// [`TimeSig::supported`]), cutting or padding every voice to fit.
pub fn set_time_sig(seg: &mut Segment, time_sig: TimeSig) {
    seg.time_sig = time_sig.supported();
    edit::normalize(seg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::Bar;

    fn drum_segment() -> Segment {
        let mut seg = Segment::default();
        add_staff(&mut seg, StaffKind::Drums);
        seg
    }

    fn run(seg: &mut Segment, cursor: &mut Cursor, input: &mut Input, cmds: &[Command]) {
        for &c in cmds {
            apply(c, seg, cursor, input, 0.0).unwrap();
        }
    }

    #[test]
    fn typing_a_hi_hat_groove() {
        use Command::*;
        let (mut seg, mut c, mut input) = (drum_segment(), Cursor::default(), Input::default());
        run(&mut seg, &mut c, &mut input, &[Value(NoteValue::Eighth)]);
        for _ in 0..8 {
            run(&mut seg, &mut c, &mut input, &[Drum(DrumHit::HiHatClosed), Right]);
        }
        let voice = &seg.staves[0].bars[0].voices[0];
        assert_eq!(voice.len(), 8);
        assert!(voice.iter().all(|b| b.drums == [DrumHit::HiHatClosed]));
    }

    #[test]
    fn drum_toggles_off_into_a_rest() {
        let (mut seg, mut c, mut input) = (drum_segment(), Cursor::default(), Input::default());
        run(&mut seg, &mut c, &mut input, &[Command::Drum(DrumHit::Snare), Command::Accent]);
        assert!(seg.staves[0].bars[0].voices[0][0].accent);
        run(&mut seg, &mut c, &mut input, &[Command::Drum(DrumHit::Snare)]);
        let b = &seg.staves[0].bars[0].voices[0][0];
        assert!(b.is_rest() && !b.accent);
    }

    #[test]
    fn two_quick_digits_make_one_fret() {
        let mut seg = Segment::default();
        add_staff(&mut seg, StaffKind::Guitar);
        let (mut c, mut input) = (Cursor { string: 2, ..Default::default() }, Input::default());
        apply(Command::FretDigit(1), &mut seg, &mut c, &mut input, 10.0).unwrap();
        apply(Command::FretDigit(2), &mut seg, &mut c, &mut input, 10.3).unwrap();
        assert_eq!(seg.staves[0].bars[0].voices[0][0].tab, [TabNote { string: 2, fret: 12 }]);
        // A slow second digit replaces the fret instead.
        apply(Command::FretDigit(5), &mut seg, &mut c, &mut input, 20.0).unwrap();
        assert_eq!(seg.staves[0].bars[0].voices[0][0].tab, [TabNote { string: 2, fret: 5 }]);
    }

    #[test]
    fn repeat_bar_appends_a_copy() {
        let (mut seg, mut c, mut input) = (drum_segment(), Cursor::default(), Input::default());
        run(&mut seg, &mut c, &mut input, &[Command::Drum(DrumHit::Kick), Command::RepeatBar]);
        assert_eq!(seg.bar_count(), 2);
        assert_eq!(seg.staves[0].bars[1], seg.staves[0].bars[0]);
        assert_eq!(c.bar, 1);
    }

    #[test]
    fn too_long_duration_is_refused() {
        let (mut seg, mut c, mut input) = (drum_segment(), Cursor::default(), Input::default());
        run(&mut seg, &mut c, &mut input, &[Command::Value(NoteValue::Quarter)]);
        c.beat = 3; // last quarter of the bar
        let r = apply(Command::Value(NoteValue::Half), &mut seg, &mut c, &mut input, 0.0);
        assert_eq!(r, Err(Refused::DoesNotFit));
    }

    #[test]
    fn invalid_time_signature_is_made_supported() {
        let mut seg = drum_segment();
        set_time_sig(&mut seg, TimeSig { num: 40, den: 3 });
        assert_eq!(seg.time_sig, TimeSig { num: TimeSig::MAX_NUM, den: 4 });
    }

    #[test]
    fn removing_a_staff_above_keeps_the_cursor_on_its_staff() {
        let mut seg = drum_segment();
        add_staff(&mut seg, StaffKind::Guitar);
        let mut c = Cursor { staff: 1, ..Default::default() };
        remove_staff(&mut seg, &mut c, 0);
        assert_eq!(c.staff, 0);
        assert_eq!(seg.staves[0].kind, StaffKind::Guitar);
    }

    #[test]
    fn longer_and_shorter_step_note_values() {
        assert_eq!(step_value(NoteValue::Quarter, 1), NoteValue::Eighth);
        assert_eq!(step_value(NoteValue::Quarter, -1), NoteValue::Half);
        assert_eq!(step_value(NoteValue::Whole, -1), NoteValue::Whole);
    }

    #[test]
    fn time_signature_change_refits_bars() {
        let mut seg = drum_segment();
        set_time_sig(&mut seg, TimeSig { num: 3, den: 4 });
        let ticks: u32 = seg.staves[0].bars[0].voices[0].iter().map(|b| b.duration.ticks()).sum();
        assert_eq!(ticks, 2880);
        assert_ne!(seg.staves[0].bars[0], Bar::default());
    }
}
