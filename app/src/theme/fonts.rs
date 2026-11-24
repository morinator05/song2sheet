//! UI fonts: Noto Sans (the KDE default, already embedded for the sheet)
//! and the Phosphor icon font as a fallback for icon code points.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontTweak};

/// Font family of headings.
pub const BOLD_FAMILY: &str = "bold";

const REGULAR: &str = "noto-sans";
const BOLD: &str = "noto-sans-bold";
const ICONS: &str = "phosphor";

static PHOSPHOR: &[u8] = include_bytes!("../../../assets/fonts/Phosphor.ttf");

/// Phosphor glyphs sit on the baseline; this centres them on the text.
const ICON_Y_OFFSET: f32 = 0.1;
/// Icons are drawn a bit larger than the text next to them.
const ICON_SCALE: f32 = 1.15;

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, data: FontData| {
        fonts.font_data.insert(name.to_string(), Arc::new(data));
    };
    add(REGULAR, FontData::from_static(engrave::fonts::TEXT_REGULAR));
    add(BOLD, FontData::from_static(engrave::fonts::TEXT_BOLD));
    add(
        ICONS,
        FontData::from_static(PHOSPHOR).tweak(FontTweak {
            scale: ICON_SCALE,
            y_offset_factor: ICON_Y_OFFSET,
            ..Default::default()
        }),
    );

    let defaults = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let family = |first: &str| {
        let mut list = vec![first.to_string(), ICONS.to_string()];
        list.extend(defaults.iter().cloned());
        list
    };
    fonts.families.insert(FontFamily::Proportional, family(REGULAR));
    fonts.families.insert(FontFamily::Name(BOLD_FAMILY.into()), family(BOLD));
    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.push(ICONS.to_string());
    }
    ctx.set_fonts(fonts);
}
