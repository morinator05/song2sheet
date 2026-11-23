//! Left panel: song details and the pool of sections (segments).

use eframe::egui;
use model::{Bar, Project, Segment, SegmentId, SheetEntry, Staff, StaffKind};

pub fn show(ui: &mut egui::Ui, project: &mut Project) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Song");
        song_details(ui, project);
        ui.separator();
        ui.heading("Sections");
        ui.label("Click + to add a section to the sheet.");
        section_pool(ui, project);
    });
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
        ui.label("Tempo");
        optional_bpm(ui, &mut meta.tempo);
        ui.end_row();
    });
    ui.label("Notes");
    ui.add(egui::TextEdit::multiline(&mut meta.notes).desired_rows(2).desired_width(f32::INFINITY));
}

/// Checkbox + BPM field for an optional tempo.
fn optional_bpm(ui: &mut egui::Ui, tempo: &mut Option<u32>) {
    ui.horizontal(|ui| {
        let mut on = tempo.is_some();
        if ui.checkbox(&mut on, "").changed() {
            *tempo = on.then_some(120);
        }
        if let Some(bpm) = tempo {
            ui.add(
                egui::DragValue::new(bpm)
                    .range(20..=400)
                    .suffix(" bpm")
                    .clamp_existing_to_range(false),
            );
        }
    });
}

fn section_pool(ui: &mut egui::Ui, project: &mut Project) {
    let mut add: Option<SegmentId> = None;
    let mut duplicate: Option<SegmentId> = None;
    let mut remove: Option<SegmentId> = None;
    for seg in &mut project.segments {
        ui.horizontal(|ui| {
            if ui.button("+").on_hover_text("Add to sheet").clicked() {
                add = Some(seg.id);
            }
            ui.add(egui::TextEdit::singleline(&mut seg.name).desired_width(140.0));
            if ui.small_button("⧉").on_hover_text("Duplicate as new section").clicked() {
                duplicate = Some(seg.id);
            }
            if ui.small_button("🗑").on_hover_text("Delete section (and its sheet rows)").clicked()
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
        let mut staff = Staff::new(StaffKind::Drums);
        staff.bars.push(Bar::default());
        let name = unused_name(project);
        project.add_segment(Segment { name, staves: vec![staff], ..Default::default() });
    }
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
    let mut choice = None;
    egui::Modal::new(key).show(ui.ctx(), |ui| {
        ui.label(format!("Delete section \"{name}\" and its {rows} sheet row(s)?"));
        ui.horizontal(|ui| {
            if ui.button("Delete").clicked() {
                choice = Some(true);
            }
            if ui.button("Cancel").clicked() {
                choice = Some(false);
            }
        });
    });
    if let Some(delete) = choice {
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
