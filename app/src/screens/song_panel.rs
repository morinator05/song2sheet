//! Left panel: song details and the pool of sections (segments).

use eframe::egui;
use model::{Project, Segment, SegmentId, SheetEntry};

use super::Nav;
use super::widgets::{
    BLOCK_GAP, Icon, TEMPO_LABEL, caption, card, card_frame, clickable, confirm, count, danger,
    empty_state, ghost, header, icon_button, open_card_frame, optional_bpm,
};
use crate::theme;

/// Rows of the song notes field.
const NOTES_ROWS: usize = 2;

/// Shows the panel. `open` is the section shown in the editor, if any.
pub fn show(ui: &mut egui::Ui, project: &mut Project, open: Option<SegmentId>) -> Option<Nav> {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            header(ui, "Song", |_| {});
            card(ui, |ui| song_details(ui, project));
            ui.add_space(BLOCK_GAP);
            let nav = header(ui, "Sections", |ui| {
                if icon_button(ui, Icon::Add, "New section")
                    .on_hover_text("New empty section, opened in the editor")
                    .clicked()
                {
                    return Some(Nav::Edit(new_section(project)));
                }
                if icon_button(ui, Icon::Import, "Import score")
                    .on_hover_text("Take sections from a MusicXML score")
                    .clicked()
                {
                    return Some(Nav::Import);
                }
                None
            });
            nav.or(section_pool(ui, project, open))
        })
        .inner
}

/// Adds an empty section with an unused name. Returns its id.
pub fn new_section(project: &mut Project) -> SegmentId {
    let name = unused_name(project);
    project.add_segment(Segment { name, ..Default::default() })
}

fn song_details(ui: &mut egui::Ui, project: &mut Project) {
    let meta = &mut project.meta;
    egui::Grid::new("song_details").num_columns(2).show(ui, |ui| {
        for (label, value, hint) in [
            ("Title", &mut meta.title, "Song title"),
            ("Subtitle", &mut meta.subtitle, "e.g. live version"),
            ("Artist", &mut meta.artist, ""),
            ("Key", &mut meta.key, "e.g. E minor"),
        ] {
            ui.label(label);
            let field = egui::TextEdit::singleline(value).hint_text(hint);
            ui.add(field.desired_width(ui.available_width()));
            ui.end_row();
        }
        ui.label(TEMPO_LABEL);
        optional_bpm(ui, "", &mut meta.tempo);
        ui.end_row();
    });
    ui.label("Notes");
    let notes = egui::TextEdit::multiline(&mut meta.notes)
        .hint_text("Printed under the title")
        .desired_rows(NOTES_ROWS);
    ui.add(notes.desired_width(f32::INFINITY));
}

fn section_pool(ui: &mut egui::Ui, project: &mut Project, open: Option<SegmentId>) -> Option<Nav> {
    if project.segments.is_empty() {
        let hint = "Create one with New section, or take them from a score with Import score.";
        empty_state(ui, "No sections yet.", Some(hint), |_| {});
    }
    let mut nav = None;
    let mut add: Option<SegmentId> = None;
    let mut duplicate: Option<SegmentId> = None;
    let mut remove: Option<SegmentId> = None;
    let p = theme::palette(ui.ctx());
    for seg in &mut project.segments {
        let uses = project.sheet.iter().filter(|e| e.segment == seg.id).count();
        let frame = if open == Some(seg.id) { open_card_frame(&p) } else { card_frame(&p) };
        let frame = frame.inner_margin(theme::metrics::ROW_CARD_MARGIN);
        let row = clickable(ui, |ui| {
            frame.show(ui, |ui| {
                // One row: tools from the right, the name field takes the rest.
                let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                let layout = egui::Layout::right_to_left(egui::Align::Center);
                ui.allocate_ui_with_layout(size, layout, |ui| {
                    if danger(ui, Icon::Delete, "Delete section (and its sheet rows)").clicked() {
                        remove = Some(seg.id);
                    }
                    if ghost(ui, Icon::Duplicate, "Duplicate as a new section").clicked() {
                        duplicate = Some(seg.id);
                    }
                    if ghost(ui, Icon::AddToSheet, "Add to the end of the sheet").clicked() {
                        add = Some(seg.id);
                    }
                    if ghost(ui, Icon::Edit, "Edit notation").clicked() {
                        nav = Some(Nav::Edit(seg.id));
                    }
                    let used = match uses {
                        0 => "unused".to_string(),
                        n => format!("×{n}"),
                    };
                    caption(ui, used).on_hover_text("How often the sheet uses this section");
                    let name = egui::TextEdit::singleline(&mut seg.name).hint_text("Name");
                    ui.add(name.desired_width(ui.available_width()));
                });
            })
        });
        if row.response.on_hover_text("Double-click to edit").double_clicked() {
            nav = Some(Nav::Edit(seg.id));
        }
    }
    if let Some(id) = add {
        project.sheet.push(SheetEntry::new(id));
    }
    if let Some(id) = duplicate {
        project.duplicate_segment(id);
    }
    if let Some(id) = remove {
        let used = project.sheet.iter().any(|e| e.segment == id);
        if used {
            set_pending_delete(ui, Some(id));
        } else {
            project.remove_segment(id);
        }
    }
    confirm_delete(ui, project);
    nav
}

/// "Section N" with the smallest N not already taken.
fn unused_name(project: &Project) -> String {
    (1..)
        .map(|n| format!("Section {n}"))
        .find(|name| project.segments.iter().all(|s| &s.name != name))
        .expect("infinite candidates")
}

fn pending_delete_id(ui: &egui::Ui) -> egui::Id {
    ui.scope_id().with("pending_section_delete")
}

fn set_pending_delete(ui: &egui::Ui, id: Option<SegmentId>) {
    let key = pending_delete_id(ui);
    ui.data_mut(|d| d.insert_temp(key, id));
}

/// Asks before deleting a section that is used in the sheet, since its
/// rows are deleted with it.
fn confirm_delete(ui: &egui::Ui, project: &mut Project) {
    let key = pending_delete_id(ui);
    let Some(id) = ui.data(|d| d.get_temp::<Option<SegmentId>>(key)).flatten() else { return };
    let name = project.segment(id).map_or(String::new(), |s| s.name.clone());
    let rows = project.sheet.iter().filter(|e| e.segment == id).count();
    let question =
        format!("Delete section \"{name}\" and the {} that use it?", count(rows, "sheet row"));
    if let Some(delete) = confirm(ui.ctx(), key, &question, "Delete") {
        if delete {
            project.remove_segment(id);
        }
        set_pending_delete(ui, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_section_names_are_unique() {
        let mut p = Project::default();
        p.add_segment(Segment { name: "Section 1".into(), ..Default::default() });
        p.add_segment(Segment { name: "Section 3".into(), ..Default::default() });
        assert_eq!(unused_name(&p), "Section 2");
    }
}
