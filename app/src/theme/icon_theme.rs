//! Finds and rasterizes icons of the system icon theme (KDE's Breeze by
//! default). Breeze's symbolic icons colour their shapes through a
//! `color:` stylesheet rule; it is set to white here, so one texture can be
//! tinted to any colour when drawn.

use std::path::PathBuf;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use resvg::{tiny_skia, usvg};

use super::kde::{self, Ini};

/// Theme used when the configured one lacks an icon.
const FALLBACK_THEME: &str = "breeze";
/// kdeglobals entry naming the icon theme.
const ICONS_GROUP: &str = "Icons";
const THEME_KEY: &str = "Theme";
/// Icon theme folders holding the icons we use, and the sizes looked at
/// (smallest first: drawn at 16 points they are pixel-aligned).
const CONTEXTS: [&str; 6] = ["actions", "devices", "status", "emblems", "mimetypes", "places"];
const SIZES: [&str; 2] = ["16", "22"];
/// Data directories searched when `XDG_DATA_DIRS` is unset.
const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";
/// The colour written into the icons before tinting.
const BASE_COLOR: &str = "#ffffff";
const COLOR_RULE: &str = "color:#";
/// Length of an `rrggbb` colour.
const HEX_LEN: usize = 6;

/// The configured icon theme, from kdeglobals.
pub fn theme_name(ini: Option<&Ini>) -> String {
    ini.and_then(|i| i.get(ICONS_GROUP, THEME_KEY)).unwrap_or(FALLBACK_THEME).to_string()
}

/// The SVG file of icon `name`, if the theme (or Breeze) has it.
pub fn find(theme: &str, name: &str) -> Option<PathBuf> {
    let file = format!("{name}.svg");
    let themes = [theme, FALLBACK_THEME];
    themes.iter().flat_map(|t| icon_dirs().into_iter().map(move |d| d.join(t))).find_map(|root| {
        CONTEXTS.iter().find_map(|ctx| {
            SIZES.iter().map(|s| root.join(ctx).join(s).join(&file)).find(|p| p.is_file())
        })
    })
}

/// `…/icons` folders: the user's data folder first, then the system ones.
fn icon_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    let system = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_DATA_DIRS.to_string());
    home.into_iter().chain(system.split(':').map(PathBuf::from)).map(|d| d.join("icons")).collect()
}

/// Every `color:#rrggbb` rule of the icon's stylesheet set to white.
fn whiten(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(i) = rest.find(COLOR_RULE) {
        let start = i + COLOR_RULE.len();
        let hex = rest.get(start..start + HEX_LEN);
        out.push_str(&rest[..i]);
        if hex.is_some_and(|h| h.chars().all(|c| c.is_ascii_hexdigit())) {
            out.push_str("color:");
            out.push_str(BASE_COLOR);
            rest = &rest[start + HEX_LEN..];
        } else {
            out.push_str(COLOR_RULE);
            rest = &rest[start..];
        }
    }
    out.push_str(rest);
    out
}

/// Loads `path` as a white texture `px` pixels square.
pub fn load(ctx: &egui::Context, path: &std::path::Path, px: u32) -> Option<TextureHandle> {
    let svg = whiten(&std::fs::read_to_string(path).ok()?);
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(px, px)?;
    let size = tree.size();
    let scale =
        tiny_skia::Transform::from_scale(px as f32 / size.width(), px as f32 / size.height());
    resvg::render(&tree, scale, &mut pixmap.as_mut());
    let image = ColorImage::from_rgba_premultiplied([px as usize, px as usize], pixmap.data());
    Some(ctx.load_texture(path.to_string_lossy(), image, TextureOptions::LINEAR))
}

/// The parsed `kdeglobals`, if there is one.
pub fn config() -> Option<Ini> {
    let text = std::fs::read_to_string(kde::path()?).ok()?;
    Some(Ini::parse(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stylesheet_colours_become_white() {
        let svg = ".ColorScheme-Text { color:#232629; } .x { color:#da4453 }";
        assert_eq!(whiten(svg), ".ColorScheme-Text { color:#ffffff; } .x { color:#ffffff }");
        assert_eq!(whiten("color:#abc;"), "color:#abc;", "short colours are left alone");
    }

    #[test]
    fn theme_defaults_to_breeze() {
        assert_eq!(theme_name(None), FALLBACK_THEME);
        let ini = Ini::parse("[Icons]\nTheme=breeze-dark\n");
        assert_eq!(theme_name(Some(&ini)), "breeze-dark");
    }
}
