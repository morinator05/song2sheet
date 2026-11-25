//! Sizes of the design system (points). See `docs/DESIGN.md`.

/// Spacing scale: every gap and margin is one of these.
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_S: f32 = 8.0;
pub const SPACE_M: f32 = 12.0;
pub const SPACE_L: f32 = 16.0;

/// Gap between items in a row / between rows.
pub const ITEM_SPACING: [f32; 2] = [SPACE_S, 6.0];
/// Space between a button's frame and its text.
pub const BUTTON_PADDING: [f32; 2] = [10.0, SPACE_XS];
/// Height of buttons, fields and combo boxes.
pub const CONTROL_HEIGHT: f32 = 26.0;
/// Inner margin of panels and cards.
pub const PANEL_MARGIN: f32 = SPACE_M;
pub const CARD_MARGIN: f32 = SPACE_M;
/// Gap between blocks (cards, groups) of a screen.
pub const BLOCK_GAP: f32 = SPACE_L;

/// Type scale.
pub const HEADING_SIZE: f32 = 17.0;
pub const BODY_SIZE: f32 = 13.5;
pub const SMALL_SIZE: f32 = 11.0;
pub const MONO_SIZE: f32 = 12.5;

/// Corner radii.
pub const WIDGET_RADIUS: u8 = 6;
pub const CARD_RADIUS: u8 = 8;
pub const WINDOW_RADIUS: u8 = 10;

/// Stroke widths.
pub const HAIRLINE: f32 = 1.0;
/// The drop indicator while dragging sheet rows.
pub const DROP_LINE: f32 = 2.0;

/// Shadow under the preview's paper pages.
pub const PAPER_SHADOW_BLUR: u8 = 10;
pub const PAPER_SHADOW_OFFSET: [i8; 2] = [0, 2];

/// Compact cards (Sections rows): inner margin.
pub const ROW_CARD_MARGIN: f32 = SPACE_XS;

/// Side of an icon.
pub const ICON_SIZE: f32 = 16.0;
/// Width of single-line name fields (sections, sheet row labels).
pub const NAME_FIELD_W: f32 = 160.0;
/// Width of short text fields (staff labels, chord symbols).
pub const SHORT_FIELD_W: f32 = 70.0;
/// Widest a text popup (the editor's key legend) gets before wrapping.
pub const POPUP_TEXT_MAX_W: f32 = 560.0;

/// Side panels: default and smallest width. At the smallest width the
/// panel headers still fit their actions.
pub const SONG_PANEL_W: f32 = 360.0;
pub const SONG_PANEL_MIN_W: f32 = 290.0;
pub const PREVIEW_PANEL_W: f32 = 540.0;
pub const PREVIEW_PANEL_MIN_W: f32 = 360.0;

/// The highlight under notation (cursor, selected bars): opacity of the
/// accent fill (it also gets an accent outline) and corner radius.
pub const NOTATION_HIGHLIGHT_ALPHA: f32 = 0.35;
pub const NOTATION_HIGHLIGHT_RADIUS: f32 = 2.0;
