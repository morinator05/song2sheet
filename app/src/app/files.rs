//! File actions: new, open, save, export, and the unsaved-changes guard.

use std::path::Path;

use eframe::egui;
use model::PartFilter;

use super::{App, Discarding};
use crate::document::{Document, dialogs};
use crate::screens::widgets::confirm;
use crate::status::file_name;

impl App {
    pub(super) fn open_path(&mut self, ctx: &egui::Context, path: &Path) {
        match Document::open(path) {
            Ok(doc) => {
                self.replace_document(doc);
                self.recent.add(path);
                self.status.success(ctx, format!("Opened {}", file_name(path)));
            }
            Err(e) => {
                self.recent.remove(path);
                self.status.error(ctx, e);
            }
        }
    }

    /// Runs `action`, first asking if it would discard unsaved changes.
    pub(super) fn request(&mut self, ctx: &egui::Context, action: Discarding) {
        if self.doc.is_dirty() {
            self.confirm = Some(action);
        } else {
            self.run(ctx, action);
        }
    }

    fn run(&mut self, ctx: &egui::Context, action: Discarding) {
        match action {
            Discarding::Quit => {
                self.quitting = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Discarding::New => {
                self.replace_document(Document::new());
                self.status.clear();
            }
            Discarding::Open => {
                if let Some(path) = dialogs::open() {
                    self.open_path(ctx, &path);
                }
            }
            Discarding::OpenPath(path) => self.open_path(ctx, &path),
        }
    }

    pub(super) fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(action) = self.confirm.clone() else { return };
        let id = egui::Id::unique("confirm_discard");
        let question = "This song has unsaved changes. Discard them?";
        if let Some(discard) = confirm(ctx, id, question, "Discard changes") {
            self.confirm = None;
            if discard {
                self.run(ctx, action);
            }
        }
    }

    pub(super) fn save_file(&mut self, ctx: &egui::Context, choose_path: bool) {
        self.editor.commit_chord(&mut self.doc.project);
        let path = match (&self.doc.path, choose_path) {
            (Some(p), false) => p.clone(),
            _ => match dialogs::save(&self.doc.file_stem()) {
                Some(p) => p,
                None => return,
            },
        };
        let result = self.doc.save_to(&path).map(|()| {
            self.recent.add(&path);
            format!("Saved {}", file_name(&path))
        });
        self.status.report(ctx, result);
    }

    pub(super) fn export_pdf(&mut self, ctx: &egui::Context) {
        self.editor.commit_chord(&mut self.doc.project);
        let stem = match self.filter {
            PartFilter::All => self.doc.file_stem(),
            PartFilter::Only(k) => format!("{} - {}", self.doc.file_stem(), k.name()),
        };
        let Some(path) = dialogs::export_pdf(&stem) else { return };
        let result = self
            .doc
            .export_pdf(&path, self.filter)
            .map(|()| format!("Exported {}", file_name(&path)));
        self.status.report(ctx, result);
    }
}
