//! Icons. They come from the system icon theme (KDE's Breeze) and are
//! tinted like the text next to them. Where the theme is missing, the
//! embedded Phosphor font stands in.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eframe::egui::{self, Button, Color32, Image, RichText, TextureHandle, vec2};

use crate::theme::icon_theme;

use crate::theme::metrics::ICON_SIZE;
/// Gap between a fallback glyph and its text.
const GLYPH_GAP: &str = "  ";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    New,
    Open,
    Recent,
    Save,
    Export,
    Undo,
    Redo,
    Back,
    Keyboard,
    Add,
    Remove,
    Edit,
    Duplicate,
    Delete,
    Drag,
    AddToSheet,
    Import,
    Success,
    Warning,
    Error,
}

impl Icon {
    /// Name in the freedesktop / Breeze icon theme.
    fn theme_name(self) -> &'static str {
        use Icon::*;
        match self {
            New => "document-new",
            Open => "document-open",
            Recent => "document-open-recent",
            Save => "document-save",
            Export => "document-export",
            Undo => "edit-undo",
            Redo => "edit-redo",
            Back => "go-previous",
            Keyboard => "input-keyboard",
            Add => "list-add",
            Remove => "list-remove",
            Edit => "document-edit",
            Duplicate => "edit-copy",
            Delete => "edit-delete",
            Drag => "drag-handle-symbolic",
            AddToSheet => "media-playlist-append",
            Import => "document-import",
            Success => "dialog-positive",
            Warning => "dialog-warning",
            Error => "dialog-error",
        }
    }

    /// The Phosphor glyph used without an icon theme.
    pub fn glyph(self) -> &'static str {
        use Icon::*;
        match self {
            New => "\u{e236}",
            Open => "\u{e256}",
            Recent => "\u{e1a0}",
            Save => "\u{e248}",
            Export => "\u{e702}",
            Undo => "\u{e038}",
            Redo => "\u{e036}",
            Back => "\u{e058}",
            Keyboard => "\u{e2d8}",
            Add => "\u{e3d4}",
            Remove => "\u{e32a}",
            Edit => "\u{e3b4}",
            Duplicate => "\u{e1ca}",
            Delete => "\u{e4a6}",
            Drag => "\u{eae2}",
            AddToSheet => "\u{e2f8}",
            Import => "\u{e340}",
            Success => "\u{e184}",
            Warning => "\u{e4e0}",
            Error => "\u{e4f8}",
        }
    }
}

/// Loaded icon textures by icon and pixel size (`None`: not in the theme).
#[derive(Default)]
struct Cache {
    theme: Option<String>,
    textures: HashMap<(Icon, u32), Option<TextureHandle>>,
}

fn texture(ctx: &egui::Context, icon: Icon) -> Option<TextureHandle> {
    let px = (ICON_SIZE * ctx.pixels_per_point()).round() as u32;
    let cache = ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Arc<Mutex<Cache>>>(egui::Id::unique("icon_cache")).clone()
    });
    let mut cache = cache.lock().ok()?;
    let theme = cache
        .theme
        .get_or_insert_with(|| icon_theme::theme_name(icon_theme::config().as_ref()))
        .clone();
    cache
        .textures
        .entry((icon, px))
        .or_insert_with(|| {
            let path = icon_theme::find(&theme, icon.theme_name())?;
            icon_theme::load(ctx, &path, px)
        })
        .clone()
}

/// `icon` as an image in `tint`, if the icon theme has it.
pub fn image(ctx: &egui::Context, icon: Icon, tint: Color32) -> Option<Image<'static>> {
    let tex = texture(ctx, icon)?;
    Some(Image::from_texture((tex.id(), vec2(ICON_SIZE, ICON_SIZE))).tint(tint))
}

/// A button showing `icon` (if any) and `text` (may be empty), both in
/// `color` (`None`: the widget's own text colour).
pub fn button(
    ui: &egui::Ui,
    icon: Option<Icon>,
    text: &str,
    color: Option<Color32>,
) -> Button<'static> {
    let tint = color.unwrap_or(ui.visuals().widgets.inactive.fg_stroke.color);
    let rich = |t: String| match color {
        Some(c) => RichText::new(t).color(c),
        None => RichText::new(t),
    };
    match icon.map(|i| (i, image(ui.ctx(), i, tint))) {
        Some((_, Some(img))) if text.is_empty() => Button::image(img),
        Some((_, Some(img))) => Button::image_and_text(img, rich(text.to_string())),
        Some((i, None)) if text.is_empty() => Button::new(rich(i.glyph().to_string())),
        Some((i, None)) => Button::new(rich(format!("{}{GLYPH_GAP}{text}", i.glyph()))),
        None => Button::new(rich(text.to_string())),
    }
}

/// `icon` followed by `text`, both in `color` (a frameless, inert
/// button, so the order holds in right-to-left layouts too).
pub fn label(ui: &mut egui::Ui, icon: Icon, text: &str, color: Color32) -> egui::Response {
    let content = button(ui, Some(icon), text, Some(color));
    ui.add(content.frame(false).sense(egui::Sense::hover()))
}
