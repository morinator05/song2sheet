//! Central panel: the ordered list of sheet rows. Rows can be dragged to
//! reorder, and each has a label, repeat count and comments.

use eframe::egui;
use model::{PartComment, Project, SegmentId, SheetEntry};

use super::Nav;
use super::song_panel::new_section;
use super::widgets::{
    Icon, NAME_FIELD_W, SHEET_TITLE, caption, card_frame, clickable, combo, count, danger,
    dialog_or_menu_open, empty_state, ghost, header, icon_button, icons, open_card_frame,
    staff_kind_options,
};
use crate::shortcuts;
use crate::theme::{self, metrics};

/// Largest repeat count offered for a row.
const MAX_REPEAT: u32 = 32;

enum RowAction {
    Move { from: usize, to: usize },
    Duplicate(usize),
    Remove(usize),
    Edit(SegmentId),
}

/// Shows the sheet. `open` is the section shown in the editor, if any.
pub fn show(ui: &mut egui::Ui, project: &mut Project, open: Option<SegmentId>) -> Option<Nav> {
    let rows = project.sheet.len();
    header(ui, SHEET_TITLE, |_| {});
    if project.sheet.is_empty() {
        return empty_sheet(ui, project);
    }
    caption(ui, count(rows, "row"));
    let sections: Vec<(SegmentId, String)> =
        project.segments.iter().map(|s| (s.id, s.name.clone())).collect();
    let mut action = None;
    let p = theme::palette(ui.ctx());
    keyboard_move(ui, &mut project.sheet);
    let mut focused_row = None;
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        for (i, entry) in project.sheet.iter_mut().enumerate() {
            let frame =
                if open == Some(entry.segment) { open_card_frame(&p) } else { card_frame(&p) };
            let response =
                clickable(ui, |ui| frame.show(ui, |ui| row(ui, i, entry, &sections, &mut action)))
                    .response;
            if response.double_clicked() {
                action = Some(RowAction::Edit(entry.segment));
            }
            if has_focus_inside(ui, response.rect) {
                focused_row = Some(i);
            }
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
    ui.data_mut(|d| d.insert_temp(focused_row_id(ui), focused_row));
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

fn focused_row_id(ui: &egui::Ui) -> egui::Id {
    ui.scope_id().with("focused_sheet_row")
}

/// Id of a row's label field (the same from any `Ui`), so focus can
/// follow a moved row.
fn label_id(index: usize) -> egui::Id {
    egui::Id::unique("sheet_row_label").with(index)
}

/// Whether the widget with keyboard focus lies inside `rect`.
fn has_focus_inside(ui: &egui::Ui, rect: egui::Rect) -> bool {
    let focused = ui.memory(|m| m.focused());
    focused.and_then(|id| ui.ctx().read_response(id)).is_some_and(|r| rect.contains_rect(r.rect))
}

/// Alt+Up/Down move the row whose field has focus (found last frame);
/// the focus moves along to the row's label field.
fn keyboard_move(ui: &mut egui::Ui, sheet: &mut Vec<SheetEntry>) {
    if dialog_or_menu_open(ui.ctx()) {
        return;
    }
    let Some(i) = ui.data(|d| d.get_temp::<Option<usize>>(focused_row_id(ui))).flatten() else {
        return;
    };
    let pressed = |ui: &mut egui::Ui, s| ui.input_mut(|inp| inp.consume_shortcut(&s));
    let to = if pressed(ui, shortcuts::MOVE_ROW_UP) {
        i.checked_sub(1)
    } else if pressed(ui, shortcuts::MOVE_ROW_DOWN) {
        Some(i + 1).filter(|t| *t < sheet.len())
    } else {
        None
    };
    if let Some(to) = to {
        move_row(sheet, i, to);
        // The label fields' undo histories belong to the rows that left.
        for index in [i, to] {
            ui.data_mut(|d| d.remove::<egui::text_edit::TextEditState>(label_id(index)));
        }
        let id = label_id(to);
        ui.memory_mut(|m| m.request_focus(id));
    }
}

/// Tooltip of the drag handle: the ways to rearrange and open a row.
fn handle_tip(ctx: &egui::Context) -> String {
    let (up, down) = (
        ctx.format_shortcut(&shortcuts::MOVE_ROW_UP),
        ctx.format_shortcut(&shortcuts::MOVE_ROW_DOWN),
    );
    format!(
        "Drag to reorder (or {up} / {down} while editing the row); double-click a row to edit it"
    )
}

/// What to do on an empty sheet, with the two ways to start.
fn empty_sheet(ui: &mut egui::Ui, project: &mut Project) -> Option<Nav> {
    if project.segments.is_empty() {
        let hint = "A section is one part of the song: intro, verse, chorus…";
        return empty_state(ui, "The sheet is empty.", Some(hint), |ui| {
            if icon_button(ui, Icon::Add, "New section").clicked() {
                Some(Nav::Edit(new_section(project)))
            } else if icon_button(ui, Icon::Import, "Import score").clicked() {
                Some(Nav::Import)
            } else {
                None
            }
        });
    }
    let hint = "Use the add-to-sheet button of a section in the Sections list.";
    empty_state(ui, "The sheet is empty.", Some(hint), |_| None)
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
        let weak = theme::palette(ui.ctx()).weak_text;
        let handle = match icons::image(ui.ctx(), Icon::Drag, weak) {
            Some(img) => ui.add(img.sense(egui::Sense::drag())).on_hover_text(handle_tip(ui.ctx())),
            None => ui
                .add(
                    egui::Label::new(egui::RichText::new(Icon::Drag.glyph()).color(weak))
                        .sense(egui::Sense::drag()),
                )
                .on_hover_text(handle_tip(ui.ctx())),
        };
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
                .id(label_id(index))
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
            if danger(ui, Icon::Delete, "Remove row").clicked() {
                *action = Some(RowAction::Remove(index));
            }
            if ghost(ui, Icon::Duplicate, "Duplicate row").clicked() {
                *action = Some(RowAction::Duplicate(index));
            }
            if ghost(ui, Icon::Edit, "Edit this section").clicked() {
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
                if danger(ui, Icon::Delete, "Remove part comment").clicked() {
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
    if icon_button(ui, Icon::Add, "Add part comment")
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
