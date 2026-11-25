# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

song2sheet is a native Rust desktop tool for making compact band song sheets: **one line per song section** (intro, verse, chorus…), fitting 1–2 A4 pages, with per-section tempo, a short notation excerpt (drum groove, rhythm slashes + chord symbols, tab) and comments. The workflow has three screens: a **section picker** that pulls bar ranges out of an imported MusicXML or Guitar Pro score, a **rhythm editor** for quick MuseScore/Guitar Pro-style step input, and an **arranger** that orders segments into a sheet and exports a PDF. KISS is a hard requirement: add no features beyond this scope and keep the UI uncluttered.

## Commands

```sh
cargo run -p song2sheet [file.s2s.json]       # the GUI app (package in app/)
cargo test --workspace
cargo test -p engrave beam                    # single test(s): -p <crate> <name filter>
cargo fmt                                     # rustfmt.toml: use_small_heuristics = "Max"
cargo clippy --all-targets -- -D warnings
./install.sh [--system] [--uninstall]         # install (Linux, macOS); install.ps1 on Windows
```

CI (`.github/workflows/ci.yml`) runs clippy and the tests on Linux, macOS and Windows, and installs and uninstalls with the install scripts on each.

`tests/fixtures/groove.musicxml` and `groove.mxl` are real MuseScore exports. If MuseScore is installed, make new fixtures by writing or editing a score and running `QT_QPA_PLATFORM=offscreen mscore -o out.musicxml in.musicxml`. Real files contain a `<!DOCTYPE>`, so the parser must allow DTDs.

Dependencies are built with `opt-level = 2` even in dev builds (root `Cargo.toml`). The raster and pdf tests write `target/test-output/demo-*.png` and `demo.pdf` from `tests/fixtures/demo.s2s.json`. **Look at those images after you change the engraving or the layout.** `pdftoppm -png demo.pdf x` rasterizes the PDF so you can compare it with the preview.

## Architecture

A Cargo workspace; dependency versions live in `[workspace.dependencies]`.

- `crates/model`: data types and serde for the `.s2s.json` file. It has no UI or rendering dependencies.
  - `edit.rs` holds the editing operations (`set_duration`, `fit`, `rests`, `normalize`, `prune`). **Invariant: every voice fills its bar exactly.** Changes to beats go through `edit::*`; `edit::normalize` runs when a section is opened in the editor. Rest-only extra voices are `prune`d when saving.
- `crates/engrave`: turns `Project` + `PartFilter` into a `DisplayList` of A4 pages. The pages hold primitives (lines, rects, text) in **mm, top-left origin, y down, with text placed at its baseline**.
  - `layout/`: the page structure. `header`, `row` (left column, notation and comments) and pagination in `mod.rs`. `segment.rs`'s `layout_segment` engraves one section at the same notation width as on the sheet, for the editor.
  - `notation/`: the music. `systems` measures bars into shared onset `Columns` and packs them into systems (balanced, bars may shrink to `MIN_SCALE`). `staff` draws one staff of one system, `events` draws one beat, and `beams` adds beams, flags and tuplets. All sizes are in `metrics.rs`. `BeatBox`es record where each beat is drawn, for the editor.
  - `fonts::glyph_run` is the **only** way text is turned into glyphs. It uses plain advances with no shaping. Measuring, the raster preview and the PDF all go through it, which is what makes the preview match the printout.
- `crates/import`: MusicXML (`.musicxml`/`.xml`/`.mxl`) → `Score` (one staff per part plus per-bar time signature and tempo).
  - `extract(score, bars, parts, name)` cuts out a `Segment`.
  - `musicxml/measure.rs` walks the measures in MusicXML divisions, following `<backup>`/`<forward>`.
  - Pitched notes without fret info become rhythm slashes; a song sheet shows rhythm, not melody.
  - Drums are mapped through `<midi-unpitched>` (General MIDI + 1), falling back to the staff position.
  - Chord symbols are collected by their position in the bar and attached to whichever staff of the part is kept. A part may have several staves; the tab staff wins.
- `crates/raster`: draws a `Page` with `tiny-skia`. The app shows this image as its preview. Don't add a separate egui-native notation renderer.
- `crates/pdf`: draws the `DisplayList` with `krilla`, via `draw_glyphs` and `glyph_run`, never `draw_text`.
- `app/`: the `eframe`/`egui` binary. **UI rules are in `docs/DESIGN.md`; follow them.**
  - `theme/`: fonts (Noto Sans + Phosphor fallback icons), sizes (`metrics.rs`) and colours. `kde.rs` reads KDE's `kdeglobals` (light/dark + accent, re-read live). `icon_theme.rs` finds system theme icons (Breeze) and rasterizes them with `resvg`, coloured white for tinting. Screens take colours only from `theme::palette`.
  - `app/`: layout and navigation (`mod.rs`), top bar, status line and preview header (`chrome.rs`), file actions (`files.rs`).
  - `history` (undo/redo snapshots of `Project`), `shortcuts` (app-wide keys), `status` (fading message line), `recent` (the recent-files list, kept in eframe's app storage), `document` (file I/O; saves via a temporary file and detects unsaved changes by comparing against the last saved `Project`), `preview` (rerasterizes when the project, filter or width differs from the last render).
  - `screens/`: each `show` returns an optional `Nav` (open a section, go to Import or Sheet).
    - `song_panel`: song details and the section pool.
    - `arranger`: sheet rows with drag-to-reorder.
    - `picker/`: imports a score, chooses parts and a bar range (overview drawn in chunks of up to 8 bars, split at time-signature changes) and creates a section.
    - `notation_view`: a rendered section with a highlight (cursor or bars) and click hit-testing, shared by the editor and the picker.
    - `editor/`: the section editor. `keys` is the **single source** of shortcuts; the legend and toolbar hints are generated from it. `commands` holds the pure edit commands, `cursor` the navigation, `header` the section settings and staves, and `toolbar` the buttons.
    - `widgets/`: shared constants and helpers, `buttons` (primary/secondary/ghost/danger/toggle, `combo`), `layout` (`header`, `card`, `caption`) and `icons`.

  Keyboard scheme: a Guitar Pro-style beat cursor with MuseScore duration numbers (2–7). On guitar and bass staves, digits type frets.
- Not built: native Guitar Pro import. The user deferred it, since exporting MusicXML from Guitar Pro works. It needs real `.gp` files to test against; the GPIF format is undocumented.

Fonts live in `assets/fonts/`: Bravura (SMuFL, 1 em = 4 staff spaces) and Noto Sans (both OFL), plus the Phosphor icon font (MIT) for the UI, all embedded with `include_bytes!`.

## Code quality

Keep files small and focused: split before a file approaches ~400 lines. After each chunk of work, run a review agent and address its findings.

**No magic numbers or strings.** A bare literal is fine only when its meaning is obvious where it stands, such as `0.0`, or `/ 2.0` for a centre. Anything used in two or more places becomes a named constant immediately; values shared across files go in a shared module. Engraving sizes live in `engrave/src/notation/metrics.rs`.

## Data model conventions (`crates/model`)

- A `Project` holds `meta`, a pool of `segments` and the `sheet` (`Vec<SheetEntry>`). Sheet entries **reference** segments by `SegmentId`, so a verse played three times is one segment used three times. `duplicate_segment` makes an independent copy instead, and `remove_segment` also removes the sheet entries that use it.
- Durations are measured in ticks, with `TICKS_PER_QUARTER = 960`. Tuplets scale by `normal/actual`.
- A `Bar` is `voices: Vec<Vec<Beat>>`. For drums, voice 0 is the hands (stems up) and voice 1 is the feet (stems down, see `DrumHit::is_foot`).
- A `Beat` that has no drums, no tab and `slash == false` is a rest. Guitar and bass staves without tab render as rhythm slashes, with optional chord symbols.
- Per-part comments (`PartComment`) and `PartFilter` let each player export a sheet with only their own staves and comments from the same file.
- All structs use `#[serde(default)]`, so missing fields load fine (`SheetEntry.repeat` defaults to 1). Empty and optional fields are skipped when saving (`skip_serializing_if`). When you make an incompatible format change, bump `FORMAT_VERSION`. `Project::from_json` rejects files whose version is newer than the app supports.
- Tuplets with a zero count are invalid: use `Duration::valid_tuplet()`, not the raw field.

## Milestone order

The approved plan builds in this order: (1) model + engraver + PDF export ✅, (2) arranger UI ✅, (3) rhythm editor ✅, (4) MusicXML import + section picker ✅, (5) Guitar Pro import (deferred), (6) polish ✅ (recent files, README). PDF export is the most important feature.
