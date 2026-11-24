//! The app's look: fonts, spacing and colours. Colours follow the KDE
//! Plasma colour scheme (light/dark and accent) when `kdeglobals` exists,
//! and egui's defaults otherwise. Rules in `docs/DESIGN.md`.

mod fonts;
pub mod kde;
pub mod metrics;

use std::path::PathBuf;
use std::time::SystemTime;

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Shadow, Stroke, TextStyle, Theme, Visuals,
};
use kde::Ini;
use metrics::*;

pub use fonts::BOLD_FAMILY;

/// How often (seconds) the colour scheme file is checked for changes.
const POLL_SECS: f32 = 2.0;
/// Window backgrounds darker than this relative luminance mean dark mode.
const DARK_LUMINANCE: f32 = 0.4;
/// How strongly hover/press states tint a button with the accent.
const HOVER_TINT: f32 = 0.18;
const ACTIVE_TINT: f32 = 0.32;
/// How far a card's fill moves from the panel colour towards the text.
const CARD_LIFT: f32 = 0.045;
/// Borders: this much of the text colour over the background.
const BORDER_MIX: f32 = 0.16;
/// Weak text when the scheme has no inactive colour.
const WEAK_MIX: f32 = 0.55;
/// How strongly the item open in the editor is tinted with the accent.
pub const OPEN_TINT: f32 = 0.22;
/// Opacity of the paper shadow in the preview.
const SHADOW_ALPHA: u8 = 60;

/// Semantic colours for the screens, so none hard-codes a colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub accent: Color32,
    /// Text drawn on the accent colour.
    pub on_accent: Color32,
    pub success: Color32,
    pub warn: Color32,
    pub error: Color32,
    /// Fill and border of cards (grouped content).
    pub card: Color32,
    pub border: Color32,
    pub weak_text: Color32,
    pub shadow: Color32,
}

/// The palette of the current theme.
pub fn palette(ctx: &egui::Context) -> Palette {
    ctx.data(|d| d.get_temp(palette_id()))
        .unwrap_or_else(|| fallback_palette(&ctx.global_style().visuals))
}

fn palette_id() -> egui::Id {
    egui::Id::unique("song2sheet_palette")
}

/// Applies the theme and notices when the system colour scheme changes.
pub struct ThemeWatcher {
    path: Option<PathBuf>,
    /// Modification time of the scheme file when last applied.
    applied: Option<SystemTime>,
    /// When (app time, seconds) the file was last checked.
    checked_at: f64,
}

impl ThemeWatcher {
    /// Installs fonts and style, and applies the current colours.
    pub fn install(ctx: &egui::Context) -> Self {
        fonts::install(ctx);
        ctx.all_styles_mut(apply_metrics);
        let mut watcher = Self { path: kde::path(), applied: None, checked_at: 0.0 };
        watcher.apply(ctx);
        watcher
    }

    /// Re-applies the colours if the scheme file changed. Call every frame.
    pub fn poll(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if now - self.checked_at >= f64::from(POLL_SECS) {
            self.checked_at = now;
            if self.modified() != self.applied {
                self.apply(ctx);
            }
        }
        ctx.request_repaint_after_secs(POLL_SECS);
    }

    fn modified(&self) -> Option<SystemTime> {
        let path = self.path.as_ref()?;
        std::fs::metadata(path).and_then(|m| m.modified()).ok()
    }

    fn apply(&mut self, ctx: &egui::Context) {
        self.applied = self.modified();
        let ini = self
            .path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|t| Ini::parse(&t));
        let Some(ini) = ini.filter(Ini::has_colors) else {
            // No KDE scheme: egui's own visuals, palette derived from them.
            for (theme, mut visuals) in
                [(Theme::Dark, Visuals::dark()), (Theme::Light, Visuals::light())]
            {
                round_widgets(&mut visuals);
                ctx.set_visuals_of(theme, visuals);
            }
            ctx.set_theme(egui::ThemePreference::System);
            ctx.data_mut(|d| d.remove::<Palette>(palette_id()));
            return;
        };
        let (theme, visuals, palette) = from_kde(&ini);
        ctx.set_visuals_of(theme, visuals);
        ctx.set_theme(theme);
        ctx.data_mut(|d| d.insert_temp(palette_id(), palette));
    }
}

/// Spacing, type scale and corner radii (the same in light and dark).
fn apply_metrics(style: &mut egui::Style) {
    let s = &mut style.spacing;
    s.item_spacing = ITEM_SPACING.into();
    s.button_padding = BUTTON_PADDING.into();
    s.interact_size.y = CONTROL_HEIGHT;
    s.window_margin = egui::Margin::same(PANEL_MARGIN as i8);
    s.menu_margin = egui::Margin::same(SPACE_S as i8);
    style.text_styles = [
        (TextStyle::Heading, FontId::new(HEADING_SIZE, FontFamily::Name(BOLD_FAMILY.into()))),
        (TextStyle::Body, FontId::proportional(BODY_SIZE)),
        (TextStyle::Button, FontId::proportional(BODY_SIZE)),
        (TextStyle::Small, FontId::proportional(SMALL_SIZE)),
        (TextStyle::Monospace, FontId::monospace(MONO_SIZE)),
    ]
    .into();
    let v = &mut style.visuals;
    round_widgets(v);
}

fn round_widgets(v: &mut Visuals) {
    let w = &mut v.widgets;
    for state in
        [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open]
    {
        state.corner_radius = CornerRadius::same(WIDGET_RADIUS);
        state.expansion = 0.0;
    }
    v.window_corner_radius = CornerRadius::same(WINDOW_RADIUS);
    v.menu_corner_radius = CornerRadius::same(CARD_RADIUS);
}

/// Builds egui visuals from a KDE colour scheme.
fn from_kde(ini: &Ini) -> (Theme, Visuals, Palette) {
    use kde::*;
    let window_bg = ini.color(WINDOW, BACKGROUND).expect("checked by has_colors");
    let dark = luminance(window_bg) < DARK_LUMINANCE;
    let (theme, mut v) =
        if dark { (Theme::Dark, Visuals::dark()) } else { (Theme::Light, Visuals::light()) };
    round_widgets(&mut v);
    let pick = |group, key, default: Color32| ini.color(group, key).unwrap_or(default);

    let text = pick(WINDOW, FOREGROUND, v.text_color());
    let weak = pick(WINDOW, FOREGROUND_INACTIVE, mix(window_bg, text, WEAK_MIX));
    let view_bg = pick(VIEW, BACKGROUND, v.extreme_bg_color);
    let button_bg = pick(BUTTON, BACKGROUND, v.widgets.inactive.weak_bg_fill);
    let button_fg = pick(BUTTON, FOREGROUND, text);
    let accent = ini
        .color(SELECTION, BACKGROUND)
        .or_else(|| ini.color(GENERAL, ACCENT))
        .unwrap_or(v.selection.bg_fill);
    let on_accent = pick(SELECTION, FOREGROUND, Color32::WHITE);
    let hover = pick(BUTTON, DECORATION_HOVER, accent);
    let focus = pick(BUTTON, DECORATION_FOCUS, accent);
    let border = mix(window_bg, text, BORDER_MIX);

    v.panel_fill = window_bg;
    v.window_fill = window_bg;
    v.extreme_bg_color = view_bg;
    v.text_edit_bg_color = Some(view_bg);
    v.faint_bg_color = pick(VIEW, BACKGROUND_ALT, mix(view_bg, text, CARD_LIFT));
    v.code_bg_color = view_bg;
    v.weak_text_color = Some(weak);
    v.hyperlink_color = pick(VIEW, FOREGROUND_LINK, accent);
    v.warn_fg_color = pick(VIEW, FOREGROUND_NEUTRAL, v.warn_fg_color);
    v.error_fg_color = pick(VIEW, FOREGROUND_NEGATIVE, v.error_fg_color);
    v.selection.bg_fill = accent;
    v.selection.stroke = Stroke::new(HAIRLINE, on_accent);
    v.window_stroke = Stroke::new(HAIRLINE, border);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = window_bg;
    w.noninteractive.weak_bg_fill = window_bg;
    w.noninteractive.bg_stroke = Stroke::new(HAIRLINE, border);
    w.noninteractive.fg_stroke = Stroke::new(HAIRLINE, text);
    for (state, tint, stroke) in [
        (&mut w.inactive, 0.0, Stroke::new(HAIRLINE, border)),
        (&mut w.hovered, HOVER_TINT, Stroke::new(HAIRLINE, hover)),
        (&mut w.active, ACTIVE_TINT, Stroke::new(HAIRLINE, focus)),
        (&mut w.open, HOVER_TINT, Stroke::new(HAIRLINE, focus)),
    ] {
        state.bg_fill = mix(button_bg, accent, tint);
        state.weak_bg_fill = state.bg_fill;
        state.bg_stroke = stroke;
        state.fg_stroke = Stroke::new(HAIRLINE, button_fg);
    }

    let palette = Palette {
        accent,
        on_accent,
        success: pick(VIEW, FOREGROUND_POSITIVE, fallback_palette(&v).success),
        warn: v.warn_fg_color,
        error: v.error_fg_color,
        card: mix(window_bg, text, CARD_LIFT),
        border,
        weak_text: weak,
        shadow: Color32::from_black_alpha(SHADOW_ALPHA),
    };
    (theme, v, palette)
}

/// Positive colour when the scheme has none (Breeze's).
const FALLBACK_SUCCESS_DARK: Color32 = Color32::from_rgb(39, 174, 96);
const FALLBACK_SUCCESS_LIGHT: Color32 = Color32::from_rgb(30, 130, 70);

fn fallback_palette(v: &Visuals) -> Palette {
    let text = v.text_color();
    Palette {
        accent: v.selection.bg_fill,
        on_accent: v.selection.stroke.color,
        success: if v.dark_mode { FALLBACK_SUCCESS_DARK } else { FALLBACK_SUCCESS_LIGHT },
        warn: v.warn_fg_color,
        error: v.error_fg_color,
        card: mix(v.panel_fill, text, CARD_LIFT),
        border: mix(v.panel_fill, text, BORDER_MIX),
        weak_text: v.weak_text_color(),
        shadow: Color32::from_black_alpha(SHADOW_ALPHA),
    }
}

/// `a` moved the fraction `t` towards `b` (in sRGB; fine for UI tints).
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(ch(a.r(), b.r()), ch(a.g(), b.g()), ch(a.b(), b.b()))
}

/// Relative luminance (0 = black, 1 = white), WCAG weights on sRGB.
fn luminance(c: Color32) -> f32 {
    let lin = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

/// The paper pages' shadow in the preview.
pub fn paper_shadow(palette: &Palette) -> Shadow {
    Shadow {
        offset: PAPER_SHADOW_OFFSET,
        blur: PAPER_SHADOW_BLUR,
        spread: 0,
        color: palette.shadow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scheme(window: &str) -> Ini {
        Ini::parse(&format!(
            "[Colors:Window]\nBackgroundNormal={window}\nForegroundNormal=10,10,10\n\
             [Colors:Selection]\nBackgroundNormal=44,113,30\nForegroundNormal=255,255,255\n"
        ))
    }

    #[test]
    fn dark_and_light_follow_the_window_background() {
        assert_eq!(from_kde(&scheme("32,35,38")).0, Theme::Dark);
        assert_eq!(from_kde(&scheme("239,240,241")).0, Theme::Light);
    }

    #[test]
    fn accent_comes_from_the_selection_colour() {
        let (_, v, p) = from_kde(&scheme("32,35,38"));
        assert_eq!(p.accent, Color32::from_rgb(44, 113, 30));
        assert_eq!(v.selection.bg_fill, p.accent);
        assert_eq!(v.panel_fill, Color32::from_rgb(32, 35, 38));
    }

    #[test]
    fn mix_ends() {
        let (a, b) = (Color32::BLACK, Color32::WHITE);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
    }
}
