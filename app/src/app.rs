//! Top-level app: file actions, the toolbar and screen layout.

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use model::{PartFilter, StaffKind};

use crate::document::{Document, dialogs};
use crate::preview::Preview;
use crate::screens;

/// File actions that may discard unsaved changes.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Discarding {
    New,
    Open,
    Quit,
}

pub struct App {
    doc: Document,
    preview: Preview,
    filter: PartFilter,
    /// Last error or confirmation, shown in the toolbar.
    status: String,
    /// Action waiting for "discard unsaved changes?" confirmation.
    confirm: Option<Discarding>,
    /// Set once the user agreed to quit; lets the next close request through.
    quitting: bool,
    window_title: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            doc: Document::new(),
            preview: Preview::default(),
            filter: PartFilter::All,
            status: String::new(),
            confirm: None,
            quitting: false,
            window_title: String::new(),
        }
    }
}

impl App {
    /// Starts with `path` opened, or with an empty song and an error
    /// message if it cannot be read.
    pub fn with_file(path: &std::path::Path) -> Self {
        let mut app = Self::default();
        match Document::open(path) {
            Ok(doc) => app.doc = doc,
            Err(e) => app.status = e,
        }
        app
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ui.ctx());
        self.guard_close(ui.ctx());
        self.update_title(ui.ctx());

        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        egui::Panel::left("song").default_size(300.0).show(ui, |ui| {
            screens::song_panel::show(ui, &mut self.doc.project);
        });
        egui::Panel::right("preview").default_size(560.0).show(ui, |ui| {
            self.preview_panel(ui);
        });
        egui::CentralPanel::default().show(ui, |ui| {
            screens::arranger::show(ui, &mut self.doc.project);
        });
        self.confirm_dialog(ui.ctx());
    }
}

impl App {
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("New").on_hover_text("Ctrl+N").clicked() {
                self.request(ui.ctx(), Discarding::New);
            }
            if ui.button("Open…").on_hover_text("Ctrl+O").clicked() {
                self.request(ui.ctx(), Discarding::Open);
            }
            if ui.button("Save").on_hover_text("Ctrl+S").clicked() {
                self.save(false);
            }
            if ui.button("Save as…").clicked() {
                self.save(true);
            }
            ui.separator();
            ui.label(&self.status);
        });
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("part_filter")
                .selected_text(filter_name(self.filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.filter,
                        PartFilter::All,
                        filter_name(PartFilter::All),
                    );
                    for kind in StaffKind::ALL {
                        let f = PartFilter::Only(kind);
                        ui.selectable_value(&mut self.filter, f, filter_name(f));
                    }
                });
            if ui.button("Export PDF…").clicked() {
                self.export_pdf();
            }
        });
        ui.separator();
        self.preview.show(ui, &self.doc.project, self.filter);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let shortcut = |key| KeyboardShortcut::new(Modifiers::COMMAND, key);
        let (new, open, save) = ctx.input_mut(|i| {
            (
                i.consume_shortcut(&shortcut(Key::N)),
                i.consume_shortcut(&shortcut(Key::O)),
                i.consume_shortcut(&shortcut(Key::S)),
            )
        });
        if new {
            self.request(ctx, Discarding::New);
        }
        if open {
            self.request(ctx, Discarding::Open);
        }
        if save {
            self.save(false);
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = self.doc.title();
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }
    }

    /// Asks before closing the window with unsaved changes.
    fn guard_close(&mut self, ctx: &egui::Context) {
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && self.doc.is_dirty() && !self.quitting {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm = Some(Discarding::Quit);
        }
    }

    fn request(&mut self, ctx: &egui::Context, action: Discarding) {
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
                self.doc = Document::new();
                self.status.clear();
            }
            Discarding::Open => {
                let Some(path) = dialogs::open() else { return };
                match Document::open(&path) {
                    Ok(doc) => {
                        self.doc = doc;
                        self.status = format!("Opened {}", path.display());
                    }
                    Err(e) => self.status = e,
                }
            }
        }
    }

    fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(action) = self.confirm else { return };
        let mut choice = None;
        egui::Modal::new(egui::Id::unique("confirm_discard")).show(ctx, |ui| {
            ui.label("Discard unsaved changes?");
            ui.horizontal(|ui| {
                if ui.button("Discard").clicked() {
                    choice = Some(true);
                }
                if ui.button("Cancel").clicked() {
                    choice = Some(false);
                }
            });
        });
        if let Some(discard) = choice {
            self.confirm = None;
            if discard {
                self.run(ctx, action);
            }
        }
    }

    fn save(&mut self, choose_path: bool) {
        let path = match (&self.doc.path, choose_path) {
            (Some(p), false) => p.clone(),
            _ => match dialogs::save(&self.doc.file_stem()) {
                Some(p) => p,
                None => return,
            },
        };
        self.status = match self.doc.save_to(&path) {
            Ok(()) => format!("Saved {}", path.display()),
            Err(e) => e,
        };
    }

    fn export_pdf(&mut self) {
        let stem = match self.filter {
            PartFilter::All => self.doc.file_stem(),
            PartFilter::Only(k) => format!("{} - {}", self.doc.file_stem(), k.name()),
        };
        let Some(path) = dialogs::export_pdf(&stem) else { return };
        self.status = match self.doc.export_pdf(&path, self.filter) {
            Ok(()) => format!("Exported {}", path.display()),
            Err(e) => e,
        };
    }
}

fn filter_name(f: PartFilter) -> String {
    match f {
        PartFilter::All => "All parts".to_string(),
        PartFilter::Only(k) => k.name().to_string(),
    }
}
