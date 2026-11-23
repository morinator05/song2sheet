//! Drum instruments.

use serde::{Deserialize, Serialize};

/// Drum instruments, mapped to fixed staff positions by the engraver.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DrumHit {
    Kick,
    Snare,
    SideStick,
    HiHatClosed,
    HiHatOpen,
    HiHatPedal,
    Ride,
    RideBell,
    Crash,
    China,
    Splash,
    TomHigh,
    TomMid,
    TomFloor,
    Cowbell,
}

impl DrumHit {
    pub const ALL: [DrumHit; 15] = [
        DrumHit::Kick,
        DrumHit::Snare,
        DrumHit::SideStick,
        DrumHit::HiHatClosed,
        DrumHit::HiHatOpen,
        DrumHit::HiHatPedal,
        DrumHit::Ride,
        DrumHit::RideBell,
        DrumHit::Crash,
        DrumHit::China,
        DrumHit::Splash,
        DrumHit::TomHigh,
        DrumHit::TomMid,
        DrumHit::TomFloor,
        DrumHit::Cowbell,
    ];

    pub fn name(self) -> &'static str {
        match self {
            DrumHit::Kick => "Kick",
            DrumHit::Snare => "Snare",
            DrumHit::SideStick => "Side stick",
            DrumHit::HiHatClosed => "Hi-hat",
            DrumHit::HiHatOpen => "Hi-hat open",
            DrumHit::HiHatPedal => "Hi-hat pedal",
            DrumHit::Ride => "Ride",
            DrumHit::RideBell => "Ride bell",
            DrumHit::Crash => "Crash",
            DrumHit::China => "China",
            DrumHit::Splash => "Splash",
            DrumHit::TomHigh => "High tom",
            DrumHit::TomMid => "Mid tom",
            DrumHit::TomFloor => "Floor tom",
            DrumHit::Cowbell => "Cowbell",
        }
    }

    /// Played with the feet (notated in the lower voice).
    pub fn is_foot(self) -> bool {
        matches!(self, DrumHit::Kick | DrumHit::HiHatPedal)
    }

    /// General MIDI percussion key, used by importers.
    pub fn from_gm(key: u8) -> Option<DrumHit> {
        Some(match key {
            35 | 36 => DrumHit::Kick,
            37 => DrumHit::SideStick,
            38 | 40 => DrumHit::Snare,
            42 => DrumHit::HiHatClosed,
            44 => DrumHit::HiHatPedal,
            46 => DrumHit::HiHatOpen,
            41 | 43 => DrumHit::TomFloor,
            45 | 47 => DrumHit::TomMid,
            48 | 50 => DrumHit::TomHigh,
            49 | 57 => DrumHit::Crash,
            51 | 59 => DrumHit::Ride,
            53 => DrumHit::RideBell,
            52 => DrumHit::China,
            55 => DrumHit::Splash,
            56 => DrumHit::Cowbell,
            _ => return None,
        })
    }
}
