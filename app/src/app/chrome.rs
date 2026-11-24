//! Window chrome: the top bar (file actions, undo, screen tabs), the
//! status line and the preview panel's header.

use eframe::egui::{self, RichText};
use model::{PartFilter, Project, StaffKind};

use super::{App, Discarding, Screen};
use crate::screens::widgets::{
    EDITOR_TITLE, Icon, PICKER_TITLE, SHEET_TITLE, caption, combo, count, ghost, header,
    icon_button, menu, primary_enabled, secondary, toggle,
};
use crate::shortcuts as keys;
use crate::theme::{self, metrics};

/// A song sheet should fit on this many pages; more is flagged.
const MAX_PAGES: usize = 2;
const TOO_LONG: &str = "Longer than 2 pages: shorten comments or use fewer rows";
/// Hover text of the part chooser.
const PART_HINT: &str = "Which part the preview and the exported PDF show";

impl App {
    pub(super) fn top_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let tip = |s| ctx.format_shortcut(&s);
        ui.horizontal(|ui| {
            if icon_button(ui, Icon::New, "New").on_hover_text(tip(keys::NEW)).clicked() {
                self.request(ui.ctx(), Discarding::New);
            }
            if icon_button(ui, Icon::Open, "Open").on_hover_text(tip(keys::OPEN)).clicked() {
                self.request(ui.ctx(), Discarding::Open);
            }
            self.recent_menu(ui);
            if icon_button(ui, Icon::Save, "Save").on_hover_text(tip(keys::SAVE)).clicked() {
                self.save_file(ui.ctx(), false);
            }
            if secondary(ui, "Save as").on_hover_text(tip(keys::SAVE_AS)).clicked() {
                self.save_file(ui.ctx(), true);
            }
            ui.separator();
            let undo = format!("Undo ({})", tip(keys::UNDO));
            if ui
                .add_enabled_ui(self.history.can_undo(), |ui| ghost(ui, Icon::Undo, &undo))
                .inner
                .clicked()
            {
                self.undo(ui.ctx(), false);
            }
            let redo = format!("Redo ({})", tip(keys::REDO));
            if ui
                .add_enabled_ui(self.history.can_redo(), |ui| ghost(ui, Icon::Redo, &redo))
                .inner
                .clicked()
            {
                self.undo(ui.ctx(), true);
            }
            ui.separator();
            for (screen, title, shortcut) in [
                (Screen::Sheet, SHEET_TITLE, keys::SHOW_SHEET),
                (Screen::Editor, EDITOR_TITLE, keys::SHOW_EDITOR),
                (Screen::Picker, PICKER_TITLE, keys::SHOW_PICKER),
            ] {
                if toggle(ui, self.screen == screen, title).on_hover_text(tip(shortcut)).clicked() {
                    self.screen = screen;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                menu(ui, Icon::Keyboard, "Shortcuts", shortcut_list);
                if self.doc.is_dirty() {
                    let warn = theme::palette(ui.ctx()).warn;
                    ui.label(RichText::new("Unsaved changes").color(warn));
                }
            });
        });
    }

    pub(super) fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_min_height(metrics::CONTROL_HEIGHT);
            self.status.show(ui);
        });
    }

    pub(super) fn preview_panel(&mut self, ui: &mut egui::Ui) {
        header(ui, "Preview", |_| {});
        ui.horizontal(|ui| {
            ui.label("Part").on_hover_text(PART_HINT);
            let options = part_options(&self.doc.project, self.filter);
            combo(ui, "part_filter", &mut self.filter, &options).on_hover_text(PART_HINT);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let hint = ui.ctx().format_shortcut(&keys::EXPORT);
                let empty = self.doc.project.sheet.is_empty();
                let why = "The sheet has no rows yet";
                let export = primary_enabled(ui, !empty, Icon::Export, "Export PDF", why);
                if export.on_hover_text(hint).clicked() {
                    self.export_pdf(ui.ctx());
                }
            });
        });
        let pages = self.preview.page_count();
        if pages > 0 {
            let text = count(pages, "page");
            if pages > MAX_PAGES {
                let warn = theme::palette(ui.ctx()).warn;
                ui.label(RichText::new(text).small().color(warn)).on_hover_text(TOO_LONG);
            } else {
                caption(ui, text);
            }
        }
        ui.add_space(metrics::SPACE_S);
        self.preview.show(ui, &self.doc.project, self.filter);
    }

    /// "Recent" menu with the last opened or saved files.
    fn recent_menu(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;
        ui.add_enabled_ui(!self.recent.paths().is_empty(), |ui| {
            menu(ui, Icon::Recent, "Recent", |ui| {
                for path in self.recent.paths() {
                    let name = crate::status::file_name(path);
                    if ui.button(name).on_hover_text(path.to_string_lossy()).clicked() {
                        chosen = Some(path.clone());
                        ui.close();
                    }
                }
            })
            .on_disabled_hover_text("No recent files yet");
        });
        if let Some(path) = chosen {
            self.request(ui.ctx(), Discarding::OpenPath(path));
        }
    }
}

/// "All parts" and every instrument the song has a staff or a part
/// comment for (plus the chosen one).
fn part_options(project: &Project, chosen: PartFilter) -> Vec<(PartFilter, String)> {
    let used = |k: StaffKind| {
        project.segments.iter().any(|s| s.staves.iter().any(|st| st.kind == k))
            || project.sheet.iter().any(|e| e.part_comments.iter().any(|c| c.kind == k))
            || chosen == PartFilter::Only(k)
    };
    let mut options = vec![(PartFilter::All, "All parts".to_string())];
    options.extend(
        StaffKind::ALL
            .into_iter()
            .filter(|k| used(*k))
            .map(|k| (PartFilter::Only(k), k.name().into())),
    );
    options
}

/// The app-wide shortcuts, in the "Shortcuts" menu.
fn shortcut_list(ui: &mut egui::Ui) {
    egui::Grid::new("shortcut_list").num_columns(2).show(ui, |ui| {
        for (shortcut, what) in keys::LEGEND {
            ui.label(RichText::new(ui.ctx().format_shortcut(&shortcut)).strong());
            ui.label(what);
            ui.end_row();
        }
        ui.label(RichText::new(keys::BACK.name()).strong());
        ui.label("Back to the sheet");
        ui.end_row();
    });
    caption(ui, "The section editor lists its own keys under Shortcuts there.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::{PartComment, Segment, SheetEntry, Staff};

    #[test]
    fn parts_with_only_comments_can_be_chosen() {
        let mut p = Project::default();
        let mut seg = Segment::default();
        seg.staves.push(Staff::new(StaffKind::Drums));
        let id = p.add_segment(seg);
        let mut row = SheetEntry::new(id);
        row.part_comments.push(PartComment { kind: StaffKind::Vocals, ..Default::default() });
        p.sheet.push(row);
        let kinds: Vec<PartFilter> =
            part_options(&p, PartFilter::All).iter().map(|o| o.0).collect();
        assert_eq!(
            kinds,
            [
                PartFilter::All,
                PartFilter::Only(StaffKind::Drums),
                PartFilter::Only(StaffKind::Vocals)
            ]
        );
    }
}
