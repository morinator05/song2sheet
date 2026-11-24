//! Central panel: the ordered list of sheet rows. Rows can be dragged to
//! reorder, and each has a label, repeat count and comments.

use eframe::egui;
use model::{PartComment, Project, SegmentId, SheetEntry};

use super::Nav;
use super::song_panel::new_section;
use super::widgets::{
    BLOCK_GAP, NAME_FIELD_W, SHEET_TITLE, caption, card_frame, combo, count, danger, ghost, header,
    icons, secondary, staff_kind_options,
};
use crate::theme::{self, metrics};

/// Largest repeat count offered for a row.
const MAX_REPEAT: u32 = 32;
/// How strongly the dragged row is tinted with the accent.
const DRAG_TINT: f32 = 0.12;

enum RowAction {
    Move { from: usize, to: usize },
    Duplicate(usize),
    Remove(usize),
    Edit(SegmentId),
}

/// Shows the sheet. `open` is the section shown in the editor, if any.
pub fn show(ui: &mut egui::Ui, project: &mut Project, open: Option<SegmentId>) -> Option<Nav> {
    let rows = project.sheet.len();
    header(ui, SHEET_TITLE, |ui| {
        caption(ui, format!("{} · drag {} to reorder", count(rows, "row"), icons::DRAG));
    });
    if project.sheet.is_empty() {
        return empty_state(ui, project);
    }
    let sections: Vec<(SegmentId, String)> =
        project.segments.iter().map(|s| (s.id, s.name.clone())).collect();
    let mut action = None;
    let p = theme::palette(ui.ctx());
    let dragging = egui::DragAndDrop::payload::<usize>(ui.ctx()).map(|i| *i);
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        for (i, entry) in project.sheet.iter_mut().enumerate() {
            let mut frame = card_frame(&p);
            if open == Some(entry.segment) {
                frame = frame.fill(theme::mix(p.card, p.accent, theme::OPEN_TINT));
            } else if dragging == Some(i) {
                frame = frame.fill(theme::mix(p.card, p.accent, DRAG_TINT));
            }
            let response = frame.show(ui, |ui| row(ui, i, entry, &sections, &mut action)).response;
            if let Some(from) = response.dnd_hover_payload::<usize>() {
                // The dropped row takes this row's place: show where it lands.
                let r = response.rect;
                let y = if *from < i { r.bottom() } else { r.top() };
                ui.painter().hline(r.x_range(), y, (metrics::DROP_LINE, p.accent));
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
        Some(RowAction::Edit(id)) => return Some(Nav::Edit(id)),
        None => {}
    }
    None
}

/// What to do on an empty sheet, with the two ways to start.
fn empty_state(ui: &mut egui::Ui, project: &mut Project) -> Option<Nav> {
    ui.add_space(BLOCK_GAP);
    let mut nav = None;
    ui.vertical_centered(|ui| {
        if project.segments.is_empty() {
            ui.label("Start with a new section, or import sections from a score.");
            ui.add_space(metrics::SPACE_S);
            ui.horizontal(|ui| {
                if secondary(ui, &icons::with(icons::ADD, "New section")).clicked() {
                    nav = Some(Nav::Edit(new_section(project)));
                }
                if secondary(ui, &icons::with(icons::SCORE, "Import a score")).clicked() {
                    nav = Some(Nav::Import);
                }
            });
        } else {
            ui.label(format!(
                "Add sections to the sheet with {} in the Sections list.",
                icons::ADD_TO_SHEET
            ));
        }
    });
    nav
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
        let handle = ui.add(
            egui::Label::new(
                egui::RichText::new(icons::DRAG).color(theme::palette(ui.ctx()).weak_text),
            )
            .sense(egui::Sense::drag()),
        );
        handle.dnd_set_drag_payload(index);
        if handle.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        ui.label(egui::RichText::new(format!("{}", index + 1)).strong());
        let options: Vec<(SegmentId, &str)> =
            sections.iter().map(|(id, name)| (*id, name.as_str())).collect();
        combo(ui, ("section", index), &mut entry.segment, &options)
            .on_hover_text("Which section this row shows");
        ui.add(
            egui::TextEdit::singleline(&mut entry.label)
                .hint_text(format!("Label: {section_name}"))
                .desired_width(NAME_FIELD_W),
        )
        .on_hover_text("Row label on the sheet (empty: the section name)");
        ui.label("Repeat ×");
        // Don't rewrite out-of-range values from a file just by showing them.
        ui.add(
            egui::DragValue::new(&mut entry.repeat)
                .range(1..=MAX_REPEAT)
                .clamp_existing_to_range(false),
        )
        .on_hover_text("How often the section is played");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if danger(ui, icons::DELETE, "Remove row").clicked() {
                *action = Some(RowAction::Remove(index));
            }
            if ghost(ui, icons::DUPLICATE, "Duplicate row").clicked() {
                *action = Some(RowAction::Duplicate(index));
            }
            if ghost(ui, icons::EDIT, "Edit this section").clicked() {
                *action = Some(RowAction::Edit(entry.segment));
            }
        });
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
    let kinds = staff_kind_options();
    for (ci, pc) in comments.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            combo(ui, ("part", index, ci), &mut pc.kind, &kinds);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if danger(ui, icons::DELETE, "Remove part comment").clicked() {
                    remove = Some(ci);
                }
                let field = egui::TextEdit::singleline(&mut pc.text)
                    .hint_text(format!("Only on the {} sheet", pc.kind.name()));
                ui.add(field.desired_width(ui.available_width()));
            });
        });
    }
    if let Some(ci) = remove {
        comments.remove(ci);
    }
    if secondary(ui, &icons::with(icons::ADD, "Part comment"))
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
