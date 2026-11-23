//! Section editor: quick step input of drum grooves, rhythm slashes,
//! chord symbols and tab for one section.

mod commands;
mod cursor;
mod keys;
mod toolbar;

use eframe::egui;
use model::{Project, Segment, SegmentId, StaffKind, TimeSig, edit};

use super::notation_view::{Highlight, NotationView};
use super::widgets::{
    BLOCK_GAP, EDITOR_TITLE, ICON_ADD, ICON_DELETE, ICON_EDIT, ICON_REMOVE, NAME_FIELD_W,
    SHORT_FIELD_W, SMALL_COMBO_W, TEMPO_LABEL, optional_bpm,
};
use commands::{Command, Input};
pub use cursor::Cursor;
use keys::Action;

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
}

impl EditorState {
    /// Starts editing `id`, making it editable if needed.
    pub fn open(&mut self, project: &mut Project, id: SegmentId) {
        if let Some(seg) = project.segment_mut(id) {
            if seg.staves.is_empty() {
                commands::add_staff(seg, StaffKind::Drums);
            }
            edit::normalize(seg);
        }
        self.segment = Some(id);
        self.cursor = Cursor::default();
        self.message = None;
    }

    fn apply(&mut self, cmd: Command, seg: &mut Segment, now: f64) {
        let result = commands::apply(cmd, seg, &mut self.cursor, &mut self.input, now);
        self.message = result.err().map(|e| e.to_string());
    }
}

pub fn show(ui: &mut egui::Ui, project: &mut Project, state: &mut EditorState) {
    let Some(seg) = state.segment.and_then(|id| project.segment_mut(id)) else {
        ui.heading(EDITOR_TITLE);
        ui.label(format!("Choose a section to edit with {ICON_EDIT} in the left panel."));
        return;
    };
    state.cursor.clamp(seg);
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        section_header(ui, seg);
        ui.separator();
        staves(ui, seg, &mut state.cursor);
        ui.separator();
        if let Some(cmd) = toolbar::show(ui, seg, &state.cursor) {
            state.apply(cmd, seg, ui.input(|i| i.time));
        }
        if let Some(b) = state.view.show(ui, seg, Highlight::Cursor(&state.cursor)) {
            state.cursor =
                Cursor { staff: b.staff, bar: b.bar, voice: b.voice, beat: b.beat, ..state.cursor };
        }
        beat_details(ui, seg, state);
        ui.add_space(BLOCK_GAP);
        egui::CollapsingHeader::new("Keyboard shortcuts").show(ui, |ui| {
            for line in toolbar::legend(ui.ctx()) {
                ui.label(line);
            }
        });
    });
    handle_keys(ui, seg, state);
}

fn section_header(ui: &mut egui::Ui, seg: &mut Segment) {
    ui.horizontal(|ui| {
        ui.label("Section");
        ui.add(egui::TextEdit::singleline(&mut seg.name).desired_width(NAME_FIELD_W));
        ui.label(TEMPO_LABEL);
        optional_bpm(ui, &mut seg.tempo);
    });
    ui.horizontal(|ui| {
        // Combo boxes, not drag values: every change is applied at once and
        // cuts bars, so intermediate values while dragging would lose notes.
        ui.label("Time");
        let mut ts = seg.time_sig;
        let combo = |ui: &mut egui::Ui, id: &str, value: &mut u8, options: &[u8]| {
            egui::ComboBox::from_id_salt(id)
                .selected_text(value.to_string())
                .width(SMALL_COMBO_W)
                .show_ui(ui, |ui| {
                    for &o in options {
                        ui.selectable_value(value, o, o.to_string());
                    }
                });
        };
        let numerators: Vec<u8> = (1..=TimeSig::MAX_NUM).collect();
        combo(ui, "time_num", &mut ts.num, &numerators);
        ui.label("/");
        combo(ui, "time_den", &mut ts.den, &TimeSig::DENOMINATORS);
        if ts != seg.time_sig {
            commands::set_time_sig(seg, ts);
        }
        ui.separator();
        ui.label(format!("Bars: {}", seg.bar_count()));
        if ui.small_button(ICON_ADD).on_hover_text("Add a bar").clicked() {
            commands::add_bar(seg);
        }
        if ui.small_button(ICON_REMOVE).on_hover_text("Remove the last bar").clicked() {
            commands::remove_last_bar(seg);
        }
    });
}

fn staves(ui: &mut egui::Ui, seg: &mut Segment, cursor: &mut Cursor) {
    let mut remove = None;
    for (i, staff) in seg.staves.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(cursor.staff == i, staff.kind.name())
                .on_hover_text("Edit this staff")
                .clicked()
            {
                cursor.staff = i;
                cursor.voice = 0;
            }
            let label =
                egui::TextEdit::singleline(&mut staff.label).hint_text(staff.kind.short_name());
            ui.add(label.desired_width(SHORT_FIELD_W));
            if ui.small_button(ICON_DELETE).on_hover_text("Remove staff").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        commands::remove_staff(seg, cursor, i);
    }
    ui.horizontal(|ui| {
        ui.label("Add staff:");
        for kind in StaffKind::ALL {
            if ui.small_button(kind.name()).clicked() {
                commands::add_staff(seg, kind);
            }
        }
    });
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
    ui.horizontal(|ui| {
        ui.label(format!("Bar {}, beat {} · voice {}", c.bar + 1, c.beat + 1, c.voice + 1));
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
            ui.colored_label(ui.visuals().warn_fg_color, message);
        }
    });
    if done {
        if let Some(beat) = state.chord_target.beat_mut(seg) {
            beat.chord = Some(state.chord_text.trim().to_string()).filter(|t| !t.is_empty());
        }
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            // Back to note input on the next beat, like MuseScore.
            state.apply(Command::Right, seg, ui.input(|i| i.time));
        }
    }
}

fn handle_keys(ui: &egui::Ui, seg: &mut Segment, state: &mut EditorState) {
    let dialog_open = ui.ctx().memory(|m| m.top_modal_layer().is_some());
    if ui.ctx().text_edit_focused() || dialog_open {
        return; // the keys belong to a text field or dialog
    }
    let kind = seg.staves.get(state.cursor.staff).map_or(StaffKind::Drums, |s| s.kind);
    let (events, now) = ui.input(|i| (i.events.clone(), i.time));
    for action in keys::actions(&events, kind) {
        match action {
            Action::Edit(cmd) => state.apply(cmd, seg, now),
            Action::ChordSymbol => state.focus_chord = true,
        }
    }
}
