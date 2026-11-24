# Design guidelines

These rules keep the app consistent. The code for them lives in `app/src/theme/` (colours, fonts, sizes) and `app/src/screens/widgets/` (buttons, cards, headers, icons). Use those helpers. Don't style widgets ad hoc.

## Layout

There are three fixed areas: **Sections** on the left, the **centre screen** in the middle (Sheet, Section editor or Import) and the **Preview** on the right. A slim top bar holds the file actions, undo/redo, the screen tabs and Shortcuts. A bottom status line shows the result of the last action.

- Each area starts with a `header`: a bold title, with that area's actions aligned right.
- Group related controls in a `card` (a lifted background with a hairline border and radius 8).
- The preview shows pages as white paper with a soft shadow.

## Spacing and type (`theme/metrics.rs`)

- Spacing scale: 4 · 8 · 12 · 16. Item spacing is 8×6, panel and card margins are 12, and the gap between blocks is 16.
- Controls are 26 pt high, and buttons have 10×4 padding. The 6 and 10 are deliberate off-scale exceptions for denser rows and wider buttons.
- Type scale:
  - Heading 17 (Noto Sans Bold)
  - Body and buttons 13.5 (Noto Sans)
  - Small 11, for captions, hints and counts in the weak colour
  - Monospace 12.5
- Corner radii: 6 for widgets, 8 for cards and menus, 10 for dialogs.

## Buttons (`widgets/buttons.rs`)

| Kind | Helper | Use |
|---|---|---|
| Primary | `primary`, `primary_enabled` | The one main action of an area (Export PDF, Create section). Accent fill. |
| Secondary | `secondary` | Ordinary actions. |
| Ghost | `ghost` | Tools repeated in every row. Icon only, always with a tooltip. |
| Danger | `danger`, `danger_filled` | Destructive actions: red on hover, red fill in a confirmation. |
| Toggle | `toggle` | On/off state (durations, drums, screen tabs, active staff). Accent when on. |

- Button text never wraps or truncates (`TextWrapMode::Extend`). Don't give buttons a fixed width.
- Use `combo` for combo boxes. It sizes the box to its longest option.
- A disabled primary button says why in a caption and in its tooltip.
- Labels are sentence case and use verbs. Don't end a label with "…".
- Icons come from the embedded Phosphor font (`widgets/icons.rs`). Pair an icon with text, except in ghost buttons. Never use emoji.

## Colour (`theme/mod.rs`)

Colours come from KDE Plasma's `kdeglobals`: the window, view, button and selection groups, and the accent. Light or dark is decided by the window background. The file is checked every 2 s, so a scheme change in System Settings applies live. Without the file, egui's own light and dark visuals are used, following the system setting where egui can detect it. Screens read colours only from `theme::palette(ctx)`.

- **Accent** marks state and is used for nothing else:
  - selection and toggles that are on
  - the primary button
  - the active screen tab
  - focus and hover strokes
  - the drop line while dragging
  - the editor cursor
  - the section open in the editor, tinted in the Sections list and in the sheet rows
- **Semantic colours** from the scheme's positive, neutral and negative colours:
  - success for confirmations (they fade after 5 s)
  - warn for "Unsaved changes" and for refused editor commands
  - error for failures (they stay until replaced)
- **Weak text** is for captions, counts and hints.

## Keyboard

- App-wide shortcuts are defined in `app/src/shortcuts.rs`.
- The editor's keys are in `screens/editor/keys.rs`, the single source for its tooltips and legend.
- Esc cancels a dialog, clears the import selection, cancels the chord field and goes back to the sheet. Enter confirms a dialog. On the Import screen, Enter in the Name field creates the section.
- While a text field has focus, Ctrl+Z belongs to the field.
