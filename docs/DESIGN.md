# Design guidelines

These rules keep the app consistent. The code for them lives in `app/src/theme/` (colours, fonts, sizes) and `app/src/screens/widgets/` (buttons, cards, headers, icons). Use those helpers. Don't style widgets ad hoc.

## Layout

There are three fixed areas: **Sections** on the left, the **centre screen** in the middle (Sheet, Section editor or Import) and the **Preview** on the right. A slim top bar holds the file actions, undo/redo, the screen tabs and Shortcuts. A bottom status line shows the result of the last action.

- Each area starts with a `header`: a bold title, with that area's actions aligned right. The actions keep their size; a title too long for the rest is cut with "…", and the full title is in its tooltip. Captions (counts, file names) go on their own line under the header, never inside it.
- Group related controls in a `card` (a lifted background with a hairline border and radius 8). Notation (the editor's section, the import overview) sits in a card too. Repeated compact rows (Sections) use cards with the `ROW_CARD_MARGIN`.
- The preview shows pages as white paper with a soft shadow.
- Panels have a smallest width (`SONG_PANEL_MIN_W`, `PREVIEW_PANEL_MIN_W`) at which their header actions still fit.
- Separate groups inside one card with a separator; separate cards and blocks with the 16-point gap.

### Empty states

Every screen or list with nothing in it uses `empty_state`: a centred sentence saying what is missing, the buttons that fix it (at most two), and a weak hint under them.

### Forms

Fields with labels in a column (song details, import parts) use a two-column grid with labels on the left. Single rows of settings (the editor's section settings) put each label directly before its control. Field widths come from `NAME_FIELD_W` and `SHORT_FIELD_W`; a field that should fill its row asks for the available width. Column headings use `group_label`.

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
- Icons (`widgets/icons.rs`, the `Icon` enum) come from the system icon theme: KDE's configured theme, falling back to Breeze. They are tinted like the text next to them and centred as images. Without an icon theme, the embedded Phosphor font stands in. Pair an icon with text, except in ghost buttons. Never use emoji.

## Messages and tooltips

- The status line shows the result of every action: file actions, imports, undo/redo, refused editor commands and warnings such as cut bars. Screens send messages with `status::post`. Confirmations and warnings fade after 5 s; errors stay until replaced.
- A message may also show inline when it is context for what the user is typing (an editor refusal next to the chord field).
- Tooltips say what a control does, followed by its shortcut in brackets: "Quarter note (5)", "Export the chosen part as PDF (Ctrl+E)". Build them with `widgets::hint`.

## Words

- **Song:** the file. **Section:** one part of the song (intro, verse…) with its notation. **Row:** one line of the sheet, showing a section (a section can appear in many rows). **Staff:** one instrument's line in a section. **Part:** an instrument's sheet (preview and export); on the Import screen the score's own parts are called **score parts**.
- Buttons that create or open say what: "New song", "New section", "Open song", "Open score", "Import score".

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
- **Contrast:** accent tints (`OPEN_TINT`) only colour backgrounds behind normal text, so text keeps the scheme's contrast. Text on accent or error fills uses the scheme's selection foreground (`on_accent`, `on_error`).

## Fonts

The UI uses the embedded Noto Sans, not the KDE font setting. The sheet is engraved with the same font, so the preview matches the printout exactly; Noto Sans is also Plasma's default.

## Keyboard

- App-wide shortcuts are defined in `app/src/shortcuts.rs`.
- The editor's keys are in `screens/editor/keys.rs`, the single source for its tooltips and legend.
- Esc cancels a dialog, clears the import selection, cancels the chord field and goes back to the sheet. Enter never confirms a destructive dialog (Discard, Delete): only a click does. On the Import screen, Enter in the Name field creates the section.
- While a text field has focus, Ctrl+Z belongs to the field. Typing merges into one undo step; every other change is its own step.
- While a dialog or menu is open, no screen or app shortcut acts behind it (`widgets::keys_free`).
- On the Import screen, Left/Right move the bar selection and Shift+Left/Right extend it, scrolling it into view.
- Double-clicking the background of a sheet row or a Sections row opens that section in the editor.
