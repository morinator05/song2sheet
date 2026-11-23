//! Section picker: open a MusicXML score, choose parts and bars, and turn
//! them into a section of the song.

mod selection;

use std::ops::Range;

use eframe::egui;
use import::Score;
use model::{Project, Segment, SheetEntry, StaffKind};

use super::notation_view::{Highlight, NotationView};
use super::widgets::{BLOCK_GAP, NAME_FIELD_W, SMALL_COMBO_W};
use crate::document::dialogs;
use selection::{Selection, bar_range_label, chunks};

/// Heading and toolbar label of this screen.
pub const PICKER_TITLE: &str = "Import";

#[derive(Default)]
pub struct PickerState {
    score: Option<Score>,
    file_name: String,
    /// Which parts go into the section, by part index.
    use_part: Vec<bool>,
    selection: Option<Selection>,
    name: String,
    /// Also append the new section to the sheet.
    add_to_sheet: bool,
    overview: Overview,
    message: String,
}

/// The score cut into chunks for display, cached until the chosen parts or
/// their kinds change.
#[derive(Default)]
struct Overview {
    built_for: Option<(Vec<usize>, Vec<StaffKind>)>,
    chunks: Vec<(Range<usize>, Segment, NotationView)>,
}

impl Overview {
    fn refresh(&mut self, score: &Score, parts: Vec<usize>) {
        let kinds: Vec<StaffKind> = score.parts.iter().map(|p| p.staff.kind).collect();
        let key = Some((parts, kinds));
        if self.built_for == key {
            return;
        }
        let parts = &key.as_ref().expect("just set").0;
        self.chunks = chunks(&score.bars)
            .into_iter()
            .map(|c| {
                let seg = import::extract(score, c.clone(), parts, "");
                (c, seg, NotationView::default())
            })
            .collect();
        self.built_for = key;
    }
}

impl PickerState {
    fn open(&mut self, project: &mut Project) {
        let Some(path) = dialogs::open_score() else { return };
        let score = match import::load(&path) {
            Ok(score) => score,
            Err(e) => {
                self.message = e.to_string();
                return;
            }
        };
        let meta = &mut project.meta;
        let mut filled = Vec::new();
        if meta.title.trim().is_empty() && !score.title.is_empty() {
            meta.title = score.title.clone();
            filled.push("title");
        }
        if meta.artist.trim().is_empty() && !score.composer.is_empty() {
            meta.artist = score.composer.clone();
            filled.push("artist");
        }
        *self = PickerState {
            use_part: vec![true; score.parts.len()],
            file_name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            add_to_sheet: true,
            message: if filled.is_empty() {
                String::new()
            } else {
                format!("Took the song's {} from the score.", filled.join(" and "))
            },
            score: Some(score),
            ..Default::default()
        };
    }

    fn chosen_parts(&self) -> Vec<usize> {
        self.use_part.iter().enumerate().filter(|(_, on)| **on).map(|(i, _)| i).collect()
    }

    /// Adds the selected bars as a section. Returns a confirmation.
    fn create(&mut self, project: &mut Project) -> Option<String> {
        let score = self.score.as_ref()?;
        let bars = self.selection.as_ref()?.bars.clone();
        let name = self.name.trim().to_string();
        let id =
            project.add_segment(import::extract(score, bars.clone(), &self.chosen_parts(), &name));
        if self.add_to_sheet {
            project.sheet.push(SheetEntry::new(id));
        }
        let confirmation = format!("Added \"{name}\" ({}).", bar_range_label(&score.bars, &bars));
        self.selection = None;
        self.name.clear();
        Some(confirmation)
    }
}

pub fn show(ui: &mut egui::Ui, project: &mut Project, state: &mut PickerState) {
    ui.heading(PICKER_TITLE);
    ui.horizontal(|ui| {
        if ui.button("Open score…").clicked() {
            state.open(project);
        }
        ui.label(&state.file_name);
    });
    if !state.message.is_empty() {
        ui.label(&state.message);
    }
    if state.score.is_none() {
        ui.label("Export your song as MusicXML (MuseScore: File → Export; Guitar Pro: File → Export → MusicXML), open it here and pick the bars of each section.");
        return;
    }
    parts(ui, state);
    ui.separator();
    selection_row(ui, project, state);
    ui.add_space(BLOCK_GAP);
    overview(ui, state);
}

fn parts(ui: &mut egui::Ui, state: &mut PickerState) {
    let Some(score) = state.score.as_mut() else { return };
    ui.label("Parts");
    for (i, part) in score.parts.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.use_part[i], &part.name);
            let kind = &mut part.staff.kind;
            let combo = egui::ComboBox::from_id_salt(("part_kind", i))
                .selected_text(kind.name())
                .width(SMALL_COMBO_W);
            combo.show_ui(ui, |ui| {
                for k in StaffKind::ALL {
                    ui.selectable_value(kind, k, k.name());
                }
            });
        });
    }
}

fn selection_row(ui: &mut egui::Ui, project: &mut Project, state: &mut PickerState) {
    let (Some(score), Some(selection)) = (&state.score, &state.selection) else {
        ui.label("Click a bar to start a section, Shift+click to extend it.");
        return;
    };
    let mut text = format!("{} selected.", bar_range_label(&score.bars, &selection.bars));
    if selection.mixes_meters(&score.bars) {
        text.push_str(" They have different time signatures; the first one is used.");
    }
    ui.label(text);
    ui.horizontal(|ui| {
        ui.label("Name");
        ui.add(
            egui::TextEdit::singleline(&mut state.name)
                .hint_text("e.g. Verse")
                .desired_width(NAME_FIELD_W),
        );
        ui.checkbox(&mut state.add_to_sheet, "add to sheet");
        let ready = !state.name.trim().is_empty() && !state.chosen_parts().is_empty();
        if ui.add_enabled(ready, egui::Button::new("Create section")).clicked()
            && let Some(done) = state.create(project)
        {
            state.message = done;
        }
    });
}

/// The score in chunks of bars; clicks select bars.
fn overview(ui: &mut egui::Ui, state: &mut PickerState) {
    let Some(score) = &state.score else { return };
    let parts = state.chosen_parts();
    state.overview.refresh(score, parts);
    let mut clicked = None;
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        for (chunk, seg, view) in &mut state.overview.chunks {
            ui.label(bar_range_label(&score.bars, chunk));
            let selected = state.selection.as_ref().map(|s| s.within(chunk)).unwrap_or_default();
            if let Some(b) = view.show(ui, seg, Highlight::Bars(&selected)) {
                clicked = Some(chunk.start + b.bar);
            }
        }
    });
    if let Some(bar) = clicked {
        let extend = ui.input(|i| i.modifiers.shift);
        match &mut state.selection {
            Some(s) if extend => s.extend_to(bar),
            _ => state.selection = Some(Selection::new(bar)),
        }
    }
}
