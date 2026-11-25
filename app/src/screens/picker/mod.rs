//! Section picker: open a MusicXML score, choose parts and bars, and turn
//! them into a section of the song.

mod selection;

use std::ops::Range;

use eframe::egui;
use import::Score;
use model::{Project, Segment, SheetEntry, StaffKind};

use super::notation_view::{Highlight, NotationView};
use super::widgets::{
    Icon, NAME_FIELD_W, PICKER_TITLE, SECTION_NAME_HINT, caption, card, combo, empty_state,
    group_label, header, hint, icon_button, keys_free, primary_enabled, staff_kind_options,
};
use crate::document::dialogs;
use crate::shortcuts;
use crate::status::{self, Kind};
use crate::theme::metrics;
use selection::{Selection, bar_range_label, chunks};

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
    /// The focus-name shortcut was pressed: focus the field next frame.
    focus_name: bool,
    /// A bar the keyboard just selected, to scroll into view.
    reveal: Option<usize>,
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
    /// Asks for a score file and loads it.
    pub fn open(&mut self, ctx: &egui::Context, project: &mut Project) {
        if let Some(path) = dialogs::open_score() {
            self.load(ctx, project, &path);
        }
    }

    /// Loads the score at `path`, filling empty song details from it.
    fn load(&mut self, ctx: &egui::Context, project: &mut Project, path: &std::path::Path) {
        let score = match import::load(path) {
            Ok(score) => score,
            Err(e) => {
                status::post(ctx, Kind::Error, e.to_string());
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
        let file_name = status::file_name(path);
        let message = if filled.is_empty() {
            format!("Opened {file_name}")
        } else {
            format!("Opened {file_name} and took the song's {} from it", filled.join(" and "))
        };
        status::post(ctx, Kind::Success, message);
        *self = PickerState {
            use_part: vec![true; score.parts.len()],
            file_name,
            add_to_sheet: true,
            score: Some(score),
            ..Default::default()
        };
    }

    fn chosen_parts(&self) -> Vec<usize> {
        self.use_part.iter().enumerate().filter(|(_, on)| **on).map(|(i, _)| i).collect()
    }

    /// Adds the selected bars as a section, then selects the bar after
    /// them for the next one. Returns a confirmation.
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
        self.selection = next_start(bars.end, score.bars.len()).map(Selection::new);
        self.reveal = self.selection.as_ref().map(Selection::head);
        self.name.clear();
        Some(confirmation)
    }
}

/// Where the next section starts: the bar after `end`, if there is one.
fn next_start(end: usize, count: usize) -> Option<usize> {
    (end < count).then_some(end)
}

pub fn show(
    ui: &mut egui::Ui,
    project: &mut Project,
    state: &mut PickerState,
) -> Option<super::Nav> {
    let loaded = state.score.is_some();
    header(ui, PICKER_TITLE, |ui| {
        // Without a score the empty state offers the button.
        let tip = hint("Open another MusicXML score", ui.ctx().format_shortcut(&shortcuts::OPEN));
        if loaded && icon_button(ui, Icon::Open, "Open score").on_hover_text(tip).clicked() {
            state.open(ui.ctx(), project);
        }
    });
    if loaded {
        caption(ui, &state.file_name);
    }
    if state.score.is_none() {
        let text = "Open a MusicXML score and pick the bars of each section.";
        let hint = "MuseScore: File → Export. Guitar Pro: File → Export → MusicXML.";
        empty_state(ui, text, Some(hint), |ui| {
            if icon_button(ui, Icon::Open, "Open score").clicked() {
                state.open(ui.ctx(), project);
            }
        });
        return None;
    }
    if keys_free(ui.ctx()) {
        selection_keys(ui, state);
    }
    card(ui, |ui| parts(ui, state));
    ui.add_space(metrics::BLOCK_GAP);
    egui::Panel::bottom("picker_create")
        .frame(
            egui::Frame::new()
                .inner_margin(egui::Margin { top: metrics::SPACE_S as i8, ..Default::default() }),
        )
        .show_separator_line(false)
        .show(ui, |ui| card(ui, |ui| selection_row(ui, project, state)));
    card(ui, |ui| overview(ui, state));
    None
}

/// Left/Right move the selection by a bar, with Shift they extend it, and
/// Esc clears it (before Esc leaves the screen).
fn selection_keys(ui: &mut egui::Ui, state: &mut PickerState) {
    let count = state.score.as_ref().map_or(0, |s| s.bars.len());
    if count == 0 {
        return;
    }
    let before = state.selection.clone();
    let pressed = |ui: &mut egui::Ui, mods, key| ui.input_mut(|i| i.consume_key(mods, key));
    for (key, by) in [(egui::Key::ArrowLeft, -1), (egui::Key::ArrowRight, 1)] {
        if pressed(ui, egui::Modifiers::SHIFT, key) {
            match &mut state.selection {
                Some(s) => s.extend_by(by, count),
                None => state.selection = Some(Selection::new(0)),
            }
        } else if pressed(ui, egui::Modifiers::NONE, key) {
            state.selection =
                Some(state.selection.as_ref().map_or(Selection::new(0), |s| s.moved_by(by, count)));
        }
    }
    if state.selection != before {
        state.reveal = state.selection.as_ref().map(|s| s.head());
    }
    // The name field only exists while bars are selected.
    if state.selection.is_some() && ui.input_mut(|i| i.consume_shortcut(&shortcuts::FOCUS_NAME)) {
        state.focus_name = true;
    }
    if state.selection.is_some() && pressed(ui, egui::Modifiers::NONE, egui::Key::Escape) {
        state.selection = None;
    }
}

fn parts(ui: &mut egui::Ui, state: &mut PickerState) {
    let Some(score) = state.score.as_mut() else { return };
    let kinds = staff_kind_options();
    egui::Grid::new("picker_parts").num_columns(2).show(ui, |ui| {
        group_label(ui, "Score part");
        group_label(ui, "Instrument");
        ui.end_row();
        for (i, part) in score.parts.iter_mut().enumerate() {
            ui.checkbox(&mut state.use_part[i], &part.name);
            combo(ui, ("part_kind", i), &mut part.staff.kind, &kinds)
                .on_hover_text("How this part is drawn and which part sheet it belongs to");
            ui.end_row();
        }
    });
}

fn selection_row(ui: &mut egui::Ui, project: &mut Project, state: &mut PickerState) {
    let (Some(score), Some(selection)) = (&state.score, &state.selection) else {
        ui.label("Click a bar to start a section, Shift+click to extend it (or use the arrow keys, with Shift to extend).");
        return;
    };
    let mut text = format!("{} selected.", bar_range_label(&score.bars, &selection.bars));
    if selection.mixes_meters(&score.bars) {
        text.push_str(" They have different time signatures; the first one is used.");
    }
    ui.label(text);
    ui.horizontal(|ui| {
        ui.label("Name");
        let shortcut = ui.ctx().format_shortcut(&shortcuts::FOCUS_NAME);
        let field = ui.add(
            egui::TextEdit::singleline(&mut state.name)
                .hint_text(format!("{SECTION_NAME_HINT} ({shortcut})"))
                .desired_width(NAME_FIELD_W),
        );
        if std::mem::take(&mut state.focus_name) {
            field.request_focus();
        }
        ui.checkbox(&mut state.add_to_sheet, "Add to sheet");
        let why = if state.chosen_parts().is_empty() {
            Some("Choose at least one part")
        } else if state.name.trim().is_empty() {
            Some("Enter a name")
        } else {
            None
        };
        let create = primary_enabled(
            ui,
            why.is_none(),
            Icon::Add,
            "Create section",
            why.unwrap_or_default(),
        );
        if let Some(why) = why {
            caption(ui, why);
        }
        let entered = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if why.is_none()
            && (create.clicked() || entered)
            && let Some(done) = state.create(project)
        {
            status::post(ui.ctx(), Kind::Success, done);
        }
    });
}

/// The score in chunks of bars; clicks select bars.
fn overview(ui: &mut egui::Ui, state: &mut PickerState) {
    let Some(score) = &state.score else { return };
    let parts = state.chosen_parts();
    state.overview.refresh(score, parts);
    let mut clicked = None;
    let reveal = std::mem::take(&mut state.reveal).and_then(|bar| {
        state.overview.chunks.iter().find(|(c, _, _)| c.contains(&bar)).map(|(c, _, _)| c.start)
    });
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        for (i, (chunk, seg, view)) in state.overview.chunks.iter_mut().enumerate() {
            if i > 0 {
                ui.separator();
            }
            caption(ui, bar_range_label(&score.bars, chunk));
            let selected = state.selection.as_ref().map(|s| s.within(chunk)).unwrap_or_default();
            let top = ui.cursor().top();
            if let Some(b) = view.show(ui, seg, Highlight::Bars(&selected)) {
                clicked = Some(chunk.start + b.bar);
            }
            // After a keyboard move, scroll to the chunk with the moved end.
            if reveal == Some(chunk.start) {
                let rect =
                    egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), top..=ui.cursor().top());
                ui.scroll_to_rect(rect, None);
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

#[cfg(test)]
mod tests {
    use super::next_start;

    #[test]
    fn next_section_starts_after_the_last_one() {
        assert_eq!(next_start(8, 32), Some(8));
        assert_eq!(next_start(32, 32), None, "no bar after the end");
    }
}
