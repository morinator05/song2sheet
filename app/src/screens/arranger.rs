//! Central panel: the ordered list of sheet rows. Rows can be dragged to
//! reorder, and each has a label, repeat count and comments.

use eframe::egui;
use model::{PartComment, Project, SegmentId, SheetEntry, StaffKind};

use super::widgets::{
    ICON_ADD, ICON_DELETE, ICON_DUPLICATE, NAME_FIELD_W, SHEET_TITLE, SMALL_COMBO_W,
};

/// Largest repeat count offered for a row.
const MAX_REPEAT: u32 = 32;
/// Room kept right of a part comment field for its delete button.
const DELETE_BUTTON_W: f32 = 30.0;

enum RowAction {
    Move { from: usize, to: usize },
    Duplicate(usize),
    Remove(usize),
}

pub fn show(ui: &mut egui::Ui, project: &mut Project) {
    ui.heading(SHEET_TITLE);
    if project.sheet.is_empty() {
        ui.label(format!("Add sections from the left panel with {ICON_ADD}."));
        return;
    }
    let sections: Vec<(SegmentId, String)> =
        project.segments.iter().map(|s| (s.id, s.name.clone())).collect();
    let mut action = None;
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        for (i, entry) in project.sheet.iter_mut().enumerate() {
            let frame = egui::Frame::group(ui.style()).show(ui, |ui| {
                row(ui, i, entry, &sections, &mut action);
            });
            let response = frame.response;
            if let Some(from) = response.dnd_hover_payload::<usize>() {
                // The dropped row takes this row's place: show where it lands.
                let r = response.rect;
                let y = if *from < i { r.bottom() } else { r.top() };
                ui.painter().hline(r.x_range(), y, ui.visuals().selection.stroke);
            }
            if let Some(from) = response.dnd_release_payload::<usize>() {
                action = Some(RowAction::Move { from: *from, to: i });
            }
        }
    });
    match action {
        Some(RowAction::Move { from, to }) => move_row(&mut project.sheet, from, to),
        Some(RowAction::Duplicate(i)) => {
            let copy = project.sheet[i].clone();
            project.sheet.insert(i + 1, copy);
        }
        Some(RowAction::Remove(i)) => {
            project.sheet.remove(i);
        }
        _ => {}
    }
}

/// Moves item `from` so that it ends up at index `to`.
fn move_row<T>(items: &mut Vec<T>, from: usize, to: usize) {
    if from != to && from < items.len() && to < items.len() {
        let item = items.remove(from);
        items.insert(to, item);
    }
}

fn row(
    ui: &mut egui::Ui,
    index: usize,
    entry: &mut SheetEntry,
    sections: &[(SegmentId, String)],
    action: &mut Option<RowAction>,
) {
    let section_name =
        sections.iter().find(|s| s.0 == entry.segment).map_or("?", |s| s.1.as_str()).to_string();
    ui.horizontal(|ui| {
        let handle = ui.add(egui::Label::new("☰").sense(egui::Sense::drag()));
        handle.dnd_set_drag_payload(index);
        if handle.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        ui.add(
            egui::TextEdit::singleline(&mut entry.label)
                .hint_text(&section_name)
                .desired_width(NAME_FIELD_W),
        );
        egui::ComboBox::from_id_salt(("section", index)).selected_text(&section_name).show_ui(
            ui,
            |ui| {
                for (id, name) in sections {
                    ui.selectable_value(&mut entry.segment, *id, name);
                }
            },
        );
        ui.label("×");
        // Don't rewrite out-of-range values from a file just by showing them.
        ui.add(
            egui::DragValue::new(&mut entry.repeat)
                .range(1..=MAX_REPEAT)
                .clamp_existing_to_range(false),
        );
        if ui.small_button(ICON_DUPLICATE).on_hover_text("Duplicate row").clicked() {
            *action = Some(RowAction::Duplicate(index));
        }
        if ui.small_button(ICON_DELETE).on_hover_text("Remove row").clicked() {
            *action = Some(RowAction::Remove(index));
        }
    });
    ui.add(
        egui::TextEdit::singleline(&mut entry.comment)
            .hint_text("Comment for everyone")
            .desired_width(f32::INFINITY),
    );
    part_comments(ui, index, &mut entry.part_comments);
}

/// Comments shown only on one part's sheet.
fn part_comments(ui: &mut egui::Ui, index: usize, comments: &mut Vec<PartComment>) {
    let mut remove = None;
    for (ci, pc) in comments.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(("part", index, ci))
                .selected_text(pc.kind.name())
                .width(SMALL_COMBO_W)
                .show_ui(ui, |ui| {
                    for kind in StaffKind::ALL {
                        ui.selectable_value(&mut pc.kind, kind, kind.name());
                    }
                });
            ui.add(
                egui::TextEdit::singleline(&mut pc.text)
                    .desired_width(ui.available_width() - DELETE_BUTTON_W),
            );
            if ui.small_button(ICON_DELETE).clicked() {
                remove = Some(ci);
            }
        });
    }
    if let Some(ci) = remove {
        comments.remove(ci);
    }
    if ui
        .small_button(format!("{ICON_ADD} part note"))
        .on_hover_text("Comment shown only on one part's sheet")
        .clicked()
    {
        comments.push(PartComment::default());
    }
}

#[cfg(test)]
mod tests {
    use super::move_row;

    #[test]
    fn moves_down_and_up() {
        let mut v = vec!['A', 'B', 'C', 'D'];
        move_row(&mut v, 0, 2);
        assert_eq!(v, ['B', 'C', 'A', 'D']);
        move_row(&mut v, 3, 0);
        assert_eq!(v, ['D', 'B', 'C', 'A']);
    }

    #[test]
    fn ignores_out_of_range() {
        let mut v = vec![1, 2];
        move_row(&mut v, 0, 5);
        assert_eq!(v, [1, 2]);
    }
}
