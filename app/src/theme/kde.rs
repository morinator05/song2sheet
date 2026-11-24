//! Reads the colour scheme KDE Plasma writes to `kdeglobals`.
//!
//! The file is a plain INI file: `[Group]` headers and `Key=value` lines.
//! Colours are written as `r,g,b` (sometimes `r,g,b,a`).

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui::Color32;

/// File name of KDE's global settings, inside the XDG config directory.
const FILE_NAME: &str = "kdeglobals";

/// Colour groups of a KDE colour scheme.
pub const WINDOW: &str = "Colors:Window";
pub const VIEW: &str = "Colors:View";
pub const BUTTON: &str = "Colors:Button";
pub const SELECTION: &str = "Colors:Selection";
pub const GENERAL: &str = "General";

/// Keys inside a colour group.
pub const BACKGROUND: &str = "BackgroundNormal";
pub const BACKGROUND_ALT: &str = "BackgroundAlternate";
pub const FOREGROUND: &str = "ForegroundNormal";
pub const FOREGROUND_INACTIVE: &str = "ForegroundInactive";
pub const FOREGROUND_LINK: &str = "ForegroundLink";
pub const FOREGROUND_NEGATIVE: &str = "ForegroundNegative";
pub const FOREGROUND_NEUTRAL: &str = "ForegroundNeutral";
pub const FOREGROUND_POSITIVE: &str = "ForegroundPositive";
pub const DECORATION_FOCUS: &str = "DecorationFocus";
pub const DECORATION_HOVER: &str = "DecorationHover";
/// In `[General]`: the accent colour chosen in System Settings.
pub const ACCENT: &str = "AccentColor";

/// Where `kdeglobals` lives: `$XDG_CONFIG_HOME`, else `~/.config`.
pub fn path() -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(config.join(FILE_NAME))
}

/// The parsed file: values by (group, key).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ini {
    values: HashMap<(String, String), String>,
}

impl Ini {
    pub fn parse(text: &str) -> Self {
        let mut values = HashMap::new();
        let mut group = String::new();
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                group = name.to_string();
            } else if let Some((key, value)) = line.split_once('=') {
                values.insert((group.clone(), key.trim().to_string()), value.trim().to_string());
            }
        }
        Self { values }
    }

    pub fn get(&self, group: &str, key: &str) -> Option<&str> {
        self.values.get(&(group.to_string(), key.to_string())).map(String::as_str)
    }

    /// An `r,g,b` colour (alpha, if present, is ignored).
    pub fn color(&self, group: &str, key: &str) -> Option<Color32> {
        let mut parts = self.get(group, key)?.split(',').map(|p| p.trim().parse::<u8>());
        let (r, g, b) = (parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?);
        Some(Color32::from_rgb(r, g, b))
    }

    /// Whether the file holds a colour scheme at all.
    pub fn has_colors(&self) -> bool {
        self.color(WINDOW, BACKGROUND).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
[ColorEffects:Disabled]
Color=56,56,56

[Colors:Window]
BackgroundNormal=32,35,38
ForegroundNormal = 252,252,252

# comment
[General]
AccentColor=44,113,30,255
Broken=1,2
";

    #[test]
    fn reads_colors_by_group() {
        let ini = Ini::parse(SAMPLE);
        assert_eq!(ini.color(WINDOW, BACKGROUND), Some(Color32::from_rgb(32, 35, 38)));
        assert_eq!(ini.color(WINDOW, FOREGROUND), Some(Color32::from_rgb(252, 252, 252)));
        assert_eq!(ini.color(GENERAL, ACCENT), Some(Color32::from_rgb(44, 113, 30)));
        assert!(ini.has_colors());
    }

    #[test]
    fn missing_or_broken_values_are_none() {
        let ini = Ini::parse(SAMPLE);
        assert_eq!(ini.color(VIEW, BACKGROUND), None);
        assert_eq!(ini.color(GENERAL, "Broken"), None);
        assert!(!Ini::parse("").has_colors());
    }
}
