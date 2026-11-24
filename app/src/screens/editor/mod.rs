//! Section editor: quick step input of drum grooves, rhythm slashes,
//! chord symbols and tab for one section.

mod commands;
mod cursor;
mod drums;
mod header;
mod keys;
mod position;
mod toolbar;

use eframe::egui;
use model::{Project, Segment, SegmentId, StaffKind, edit};

use super::Nav;
use super::notation_view::{Highlight, NotationView};
use super::widgets::{
    BLOCK_GAP, EDITOR_TITLE, Icon, SHEET_TITLE, SHORT_FIELD_W, caption, card, header, icon_button,
    icons, keys_free, menu,
};
use crate::status::{self, Kind};
use crate::theme;
use commands::{Command, Input};
pub use cursor::Cursor;
use keys::Action;

/// Widest the keyboard legend popup gets before wrapping.
const LEGEND_MAX_W: f32 = 560.0;

#[derive(Default)]
pub struct EditorState {
    segment: Option<SegmentId>,
    cursor: Cursor,
    input: Input,
    view: NotationView,
    /// Raw chord symbol text while its field is being edited …
    chord_text: String,
    /// … and the beat it belongs to (the cursor may move meanwhile).
    chord_target: Cursor,
    /// The chord field had focus last frame (it may lose it to a click on
    /// the notation before it is drawn again, and must still commit).
    chord_was_editing: bool,
    /// The chord symbol shortcut was pressed: focus its field next frame.
    focus_chord: bool,
    /// Why the last command did nothing, shown until the next command.
    message: Option<String>,
    /// The cursor moved: scroll it into view.
    reveal: bool,
}

impl EditorState {
    /// Starts editing `id`, making it editable if needed.
    pub fn open(&mut self, project: &mut Project, id: SegmentId) {
        self.segment = Some(id);
        self.cursor = Cursor::default();
        self.message = None;
        self.refresh(project);
    }

    /// The section being edited.
    pub fn segment(&self) -> Option<SegmentId> {
        self.segment
    }

    /// Makes the section editable again after the project was replaced
    /// (undo), keeping the cursor where possible.
    pub fn refresh(&mut self, project: &mut Project) {
        let Some(seg) = self.segment.and_then(|id| project.segment_mut(id)) else {
            self.segment = None;
            return;
        };
        if seg.staves.is_empty() {
            commands::add_staff(seg, StaffKind::Drums);
        }
        edit::normalize(seg);
        self.cursor.clamp(seg);
        self.chord_was_editing = false;
    }

    /// Writes a chord symbol that is still being typed into its beat, so
    /// saving or exporting doesn't miss it.
    pub fn commit_chord(&mut self, project: &mut Project) {
        if !self.chord_was_editing {
            return;
        }
        if let Some(seg) = self.segment.and_then(|id| project.segment_mut(id)) {
            write_chord(seg, &self.chord_target, &self.chord_text);
        }
    }

    /// Runs `cmd`; a refusal shows next to the chord field and in the
    /// status line.
    fn apply(&mut self, ctx: &egui::Context, cmd: Command, seg: &mut Segment) {
        let now = ctx.input(|i| i.time);
        let result = commands::apply(cmd, seg, &mut self.cursor, &mut self.input, now);
        self.message = result.err().map(|e| e.to_string());
        if let Some(message) = &self.message {
            status::post(ctx, Kind::Warning, message.clone());
        }
        self.reveal = true;
    }
}

pub fn show(ui: &mut egui::Ui, project: &mut Project, state: &mut EditorState) -> Option<Nav> {
    let Some(seg) = state.segment.and_then(|id| project.segment_mut(id)) else {
        header(ui, EDITOR_TITLE, |_| {});
        caption(ui, "Choose a section to edit with its edit button in the Sections list.");
        return None;
    };
    state.cursor.clamp(seg);
    let title = if seg.name.trim().is_empty() {
        EDITOR_TITLE.to_string()
    } else {
        format!("{EDITOR_TITLE} · {}", seg.name.trim())
    };
    let back = header(ui, &title, |ui| {
        menu(ui, Icon::Keyboard, "Shortcuts", |ui| {
            ui.set_max_width(LEGEND_MAX_W);
            for line in toolbar::legend(ui.ctx()) {
                ui.label(line);
            }
        });
        let back = format!("Back to the sheet ({})", crate::shortcuts::BACK.name());
        icon_button(ui, Icon::Back, SHEET_TITLE).on_hover_text(back).clicked()
    });
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        card(ui, |ui| {
            header::section_settings(ui, seg);
            ui.separator();
            header::staves(ui, seg, &mut state.cursor);
        });
        ui.add_space(BLOCK_GAP);
        card(ui, |ui| {
            if let Some(cmd) = toolbar::show(ui, seg, &state.cursor) {
                state.apply(ui.ctx(), cmd, seg);
            }
        });
        ui.add_space(BLOCK_GAP);
        if let Some(b) = state.view.show(ui, seg, Highlight::Cursor(&state.cursor)) {
            state.cursor =
                Cursor { staff: b.staff, bar: b.bar, voice: b.voice, beat: b.beat, ..state.cursor };
        }
        if std::mem::take(&mut state.reveal)
            && let Some(rect) = state.view.highlight_rect()
        {
            ui.scroll_to_rect(rect, None);
        }
        ui.add_space(BLOCK_GAP);
        beat_details(ui, seg, state);
    });
    handle_keys(ui, seg, state);
    back.then_some(Nav::Sheet)
}

/// Position of the selected beat, its chord symbol field and the last
/// refusal message.
fn beat_details(ui: &mut egui::Ui, seg: &mut Segment, state: &mut EditorState) {
    let c = state.cursor;
    let Some(chord) = c.beat(seg).map(|b| b.chord.clone().unwrap_or_default()) else { return };
    let field_id = egui::Id::unique("chord_symbol_field");
    if !state.chord_was_editing {
        state.chord_text = chord;
        state.chord_target = c;
    }
    let mut done = false;
    let warn = theme::palette(ui.ctx()).warn;
    ui.horizontal(|ui| {
        caption(ui, position::caption(seg, &c));
        ui.separator();
        ui.label("Chord");
        let hint = ui.ctx().format_shortcut(&keys::CHORD_SYMBOL);
        let field = egui::TextEdit::singleline(&mut state.chord_text).id(field_id).hint_text(hint);
        let field = ui.add(field.desired_width(SHORT_FIELD_W));
        if std::mem::take(&mut state.focus_chord) {
            field.request_focus();
        }
        done = state.chord_was_editing && !field.has_focus();
        state.chord_was_editing = field.has_focus();
        if let Some(message) = &state.message {
            icons::label(ui, Icon::Warning, message, warn);
        }
    });
    // Esc leaves the field without keeping the typed text.
    let cancelled = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if done && !cancelled {
        write_chord(seg, &state.chord_target, &state.chord_text);
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            // On to the next beat, still typing chords (Esc stops).
            state.apply(ui.ctx(), Command::Right, seg);
            state.focus_chord = true;
        }
    }
}

/// Sets the chord symbol of the beat at `at` (empty text removes it).
fn write_chord(seg: &mut Segment, at: &Cursor, text: &str) {
    if let Some(beat) = at.beat_mut(seg) {
        beat.chord = Some(text.trim().to_string()).filter(|t| !t.is_empty());
    }
}

fn handle_keys(ui: &egui::Ui, seg: &mut Segment, state: &mut EditorState) {
    if !keys_free(ui.ctx()) {
        return; // the keys belong to a text field, menu or dialog
    }
    let kind = seg.staves.get(state.cursor.staff).map_or(StaffKind::Drums, |s| s.kind);
    let events = ui.input(|i| i.events.clone());
    for action in keys::actions(&events, kind) {
        match action {
            Action::Edit(cmd) => state.apply(ui.ctx(), cmd, seg),
            Action::ChordSymbol => state.focus_chord = true,
        }
    }
}
