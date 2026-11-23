# song2sheet

Compact song sheets for bands: **one line per song section** (intro, verse, chorus…) with tempo, a short notation excerpt (drum groove, chords with rhythm slashes, tab) and comments, on one or two A4 pages. Every player can export a PDF with only their own part.

## Build and run

Install Rust from <https://rustup.rs>, then:

```sh
cargo run --release -p song2sheet              # start the app
cargo run --release -p song2sheet song.s2s.json  # start with a song open
```

Song files are saved as `*.s2s.json`, a small text file you can share or put under version control.

## How to make a sheet

1. **Import** (optional): export your song from MuseScore (*File → Export → MusicXML*) or Guitar Pro (*File → Export → MusicXML*). Open it on the **Import** tab, tick the parts you want, click a bar and Shift+click another to select a section, give it a name, then click **Create section**.
2. **Section editor**: click ✏ next to a section (or *New empty section*) to write or fix the notation. Keys:
   - ←/→ select a beat; ↑/↓ switch staff (guitar/bass: string); Ctrl+↑/↓ always switch staff
   - 2–7 set the duration, MuseScore style (4 = 1/8, 5 = 1/4); +/− longer/shorter (use these on guitar/bass, where digits type frets); `.` dot; Ctrl+3 triplet
   - Drums: K kick, S snare, H hi-hat, R ride, C crash, T/M/F toms … ; V switches hands/feet
   - Guitar/bass: digits type frets; N adds a rhythm slash
   - A accent, Del rest (drums and slash staves: also 0), Ctrl+K chord symbol, Ctrl+R repeat the bar

   On macOS, use Cmd instead of Ctrl. The full list is under *Keyboard shortcuts* in the editor.
3. **Sheet**: click + next to a section to add it to the sheet. Drag rows to reorder them, set how often each is played, and add comments for everyone or only for one part.
4. **Export PDF**: choose *All parts* or one part above the preview, then click **Export PDF…**. The preview shows exactly what will be printed.

## Development

See `CLAUDE.md` for architecture and conventions. `cargo test --workspace` runs all tests.
