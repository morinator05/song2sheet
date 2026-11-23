//! Left panel: song details and the pool of sections (segments).

use eframe::egui;
use model::{Project, Segment, SegmentId, SheetEntry};

use super::widgets::{
    ICON_ADD, ICON_DELETE, ICON_DUPLICATE, ICON_EDIT, NAME_FIELD_W, TEMPO_LABEL, confirm,
};

/// Shows the panel. Returns a section the user wants to open in the editor.
pub fn show(ui: &mut egui::Ui, project: &mut Project) -> Option<SegmentId> {
    egui::ScrollArea::vertical()
        .show(ui, |ui| {
            ui.heading("Song");
            song_details(ui, project);
            ui.separator();
            ui.heading("Sections");
            ui.label(format!("{ICON_ADD} adds a section to the sheet, {ICON_EDIT} edits it."));
            section_pool(ui, project)
        })
        .inner
}

fn song_details(ui: &mut egui::Ui, project: &mut Project) {
    let meta = &mut project.meta;
    egui::Grid::new("song_details").num_columns(2).show(ui, |ui| {
        for (label, value) in [
            ("Title", &mut meta.title),
            ("Subtitle", &mut meta.subtitle),
            ("Artist", &mut meta.artist),
            ("Key", &mut meta.key),
        ] {
            ui.label(label);
            ui.text_edit_singleline(value);
            ui.end_row();
        }
        ui.label(TEMPO_LABEL);
        super::widgets::optional_bpm(ui, &mut meta.tempo);
        ui.end_row();
    });
    ui.label("Notes");
    ui.add(egui::TextEdit::multiline(&mut meta.notes).desired_rows(2).desired_width(f32::INFINITY));
}

fn section_pool(ui: &mut egui::Ui, project: &mut Project) -> Option<SegmentId> {
    let mut edit: Option<SegmentId> = None;
    let mut add: Option<SegmentId> = None;
    let mut duplicate: Option<SegmentId> = None;
    let mut remove: Option<SegmentId> = None;
    for seg in &mut project.segments {
        ui.horizontal(|ui| {
            if ui.button(ICON_ADD).on_hover_text("Add to sheet").clicked() {
                add = Some(seg.id);
            }
            ui.add(egui::TextEdit::singleline(&mut seg.name).desired_width(NAME_FIELD_W));
            if ui.small_button(ICON_EDIT).on_hover_text("Edit notation").clicked() {
                edit = Some(seg.id);
            }
            if ui.small_button(ICON_DUPLICATE).on_hover_text("Duplicate as new section").clicked() {
                duplicate = Some(seg.id);
            }
            if ui
                .small_button(ICON_DELETE)
                .on_hover_text("Delete section (and its sheet rows)")
                .clicked()
            {
                remove = Some(seg.id);
            }
        });
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
    if ui.button("New empty section").clicked() {
        let name = unused_name(project);
        edit = Some(project.add_segment(Segment { name, ..Default::default() }));
    }
    edit
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
    let question = format!("Delete section \"{name}\" and its {rows} sheet row(s)?");
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
