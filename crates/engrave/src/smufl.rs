//! SMuFL code points used by the engraver (Bravura).

pub const NOTEHEAD_WHOLE: char = '\u{E0A2}';
pub const NOTEHEAD_HALF: char = '\u{E0A3}';
pub const NOTEHEAD_BLACK: char = '\u{E0A4}';
pub const NOTEHEAD_X_WHOLE: char = '\u{E0A7}';
pub const NOTEHEAD_X_HALF: char = '\u{E0A8}';
pub const NOTEHEAD_X_BLACK: char = '\u{E0A9}';
pub const NOTEHEAD_CIRCLE_X: char = '\u{E0B3}';
pub const NOTEHEAD_TRIANGLE_UP: char = '\u{E0BE}';
pub const NOTEHEAD_DIAMOND_BLACK: char = '\u{E0DB}';
pub const SLASH_BLACK: char = '\u{E101}';
pub const SLASH_WHOLE: char = '\u{E102}';
pub const SLASH_HALF: char = '\u{E103}';

pub const CLEF_PERCUSSION: char = '\u{E069}';
pub const CLEF_TAB_6: char = '\u{E06D}';
pub const CLEF_TAB_4: char = '\u{E06E}';

pub const TIME_SIG_0: u32 = 0xE080;
pub const TUPLET_0: u32 = 0xE880;

pub const FLAG_8TH_UP: char = '\u{E240}';
pub const FLAG_8TH_DOWN: char = '\u{E241}';
pub const FLAG_16TH_UP: char = '\u{E242}';
pub const FLAG_16TH_DOWN: char = '\u{E243}';
pub const FLAG_32ND_UP: char = '\u{E244}';
pub const FLAG_32ND_DOWN: char = '\u{E245}';

pub const REST_WHOLE: char = '\u{E4E3}';
pub const REST_HALF: char = '\u{E4E4}';
pub const REST_QUARTER: char = '\u{E4E5}';
pub const REST_8TH: char = '\u{E4E6}';
pub const REST_16TH: char = '\u{E4E7}';
pub const REST_32ND: char = '\u{E4E8}';

pub const AUG_DOT: char = '\u{E1E7}';
pub const ACCENT_ABOVE: char = '\u{E4A0}';
pub const ACCENT_BELOW: char = '\u{E4A1}';
pub const OPEN: char = '\u{E5E7}';
pub const MET_QUARTER_UP: char = '\u{ECA5}';

pub fn digits(base: u32, n: u32) -> String {
    n.to_string().chars().map(|d| char::from_u32(base + d.to_digit(10).unwrap()).unwrap()).collect()
}
