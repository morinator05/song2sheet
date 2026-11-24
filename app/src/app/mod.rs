//! Top-level app: the screen layout, navigation and undo. The window
//! chrome is in `chrome`, file actions in `files`.

mod chrome;
mod files;

use std::path::{Path, PathBuf};

use eframe::egui;
use model::PartFilter;

use crate::document::Document;
use crate::history::History;
use crate::preview::Preview;
use crate::recent::RecentFiles;
use crate::screens::{self, Nav};
use crate::shortcuts as keys;
use crate::status::Status;
use crate::theme::{ThemeWatcher, metrics};

/// Side panel widths (points): default and smallest.
const SONG_PANEL_W: f32 = 360.0;
const SONG_PANEL_MIN_W: f32 = 290.0;
const PREVIEW_PANEL_W: f32 = 540.0;
const PREVIEW_PANEL_MIN_W: f32 = 340.0;

/// What the central area shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Sheet,
    Editor,
    Picker,
}

/// File actions that may discard unsaved changes.
#[derive(Clone, Debug, PartialEq)]
enum Discarding {
    New,
    /// Choose a file in a dialog.
    Open,
    /// Open this file (from the recent list).
    OpenPath(PathBuf),
    Quit,
}

pub struct App {
    doc: Document,
    history: History,
    screen: Screen,
    editor: screens::editor::EditorState,
    picker: screens::picker::PickerState,
    preview: Preview,
    filter: PartFilter,
    status: Status,
    /// Action waiting for "discard unsaved changes?" confirmation.
    confirm: Option<Discarding>,
    /// Set once the user agreed to quit; lets the next close request through.
    quitting: bool,
    window_title: String,
    recent: RecentFiles,
    theme: ThemeWatcher,
}

impl App {
    /// Starts with the system theme, the remembered recent files and, if
    /// given, `path` opened (or an error message if it cannot be read).
    pub fn new(cc: &eframe::CreationContext, path: Option<&Path>) -> Self {
        let doc = Document::new();
        let mut app = Self {
            history: History::new(&doc.project),
            doc,
            screen: Screen::Sheet,
            editor: Default::default(),
            picker: Default::default(),
            preview: Preview::default(),
            filter: PartFilter::All,
            status: Status::default(),
            confirm: None,
            quitting: false,
            window_title: String::new(),
            recent: RecentFiles::load(cc.storage),
            theme: ThemeWatcher::install(&cc.egui_ctx),
        };
        if let Some(path) = path {
            app.open_path(&cc.egui_ctx, path);
        }
        app
    }

    /// Replaces the document, resetting everything that belongs to it.
    fn replace_document(&mut self, doc: Document) {
        self.history = History::new(&doc.project);
        self.doc = doc;
        self.editor = Default::default();
        self.picker = Default::default();
        self.screen = Screen::Sheet;
    }

    fn navigate(&mut self, nav: Nav) {
        match nav {
            Nav::Edit(id) => {
                self.editor.open(&mut self.doc.project, id);
                self.screen = Screen::Editor;
            }
            Nav::Import => self.screen = Screen::Picker,
            Nav::Sheet => self.screen = Screen::Sheet,
        }
    }

    fn undo(&mut self, redo: bool) {
        let project = &mut self.doc.project;
        let done = if redo { self.history.redo(project) } else { self.history.undo(project) };
        if done {
            // The editor needs its section normalized again; that is not a step.
            self.editor.refresh(project);
            self.history.sync(project);
        }
    }

    /// The section open in the editor, while the editor is shown.
    fn open_section(&self) -> Option<model::SegmentId> {
        (self.screen == Screen::Editor).then(|| self.editor.segment()).flatten()
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.recent.save(storage);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.theme.poll(&ctx);
        // Taken before drawing: a field that loses focus to Esc this frame
        // still owns that key press.
        let typing = ctx.text_edit_focused();
        // Keys belong to an open dialog or menu, not to the app behind it.
        let blocked =
            ctx.memory(|m| m.top_modal_layer().is_some()) || egui::Popup::is_any_open(&ctx);
        if !blocked {
            self.handle_shortcuts(&ctx, typing);
        }
        self.guard_close(&ctx);
        self.update_title(&ctx);

        let style = ctx.global_style();
        let bar = egui::Frame::side_top_panel(&style)
            .inner_margin(egui::Margin::symmetric(metrics::SPACE_S as i8, metrics::SPACE_XS as i8));
        egui::Panel::top("toolbar").frame(bar).show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("status").frame(bar).show(ui, |ui| self.status_bar(ui));

        let side = egui::Frame::side_top_panel(&style).inner_margin(metrics::PANEL_MARGIN);
        let open = self.open_section();
        let mut nav = egui::Panel::left("song")
            .frame(side)
            .default_size(SONG_PANEL_W)
            .min_size(SONG_PANEL_MIN_W)
            .show(ui, |ui| screens::song_panel::show(ui, &mut self.doc.project, open))
            .inner;
        egui::Panel::right("preview")
            .frame(side)
            .default_size(PREVIEW_PANEL_W)
            .min_size(PREVIEW_PANEL_MIN_W)
            .show(ui, |ui| self.preview_panel(ui));
        let centre = egui::Frame::central_panel(&style).inner_margin(metrics::PANEL_MARGIN);
        let centre_nav = egui::CentralPanel::default()
            .frame(centre)
            .show(ui, |ui| match self.screen {
                Screen::Sheet => screens::arranger::show(ui, &mut self.doc.project, open),
                Screen::Editor => {
                    screens::editor::show(ui, &mut self.doc.project, &mut self.editor)
                }
                Screen::Picker => {
                    screens::picker::show(ui, &mut self.doc.project, &mut self.picker)
                }
            })
            .inner;
        nav = nav.or(centre_nav);
        if !typing
            && !blocked
            && self.screen != Screen::Sheet
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, keys::BACK))
        {
            nav = nav.or(Some(Nav::Sheet));
        }
        // Record edits first: opening a section normalizes it, which is
        // not a step of its own.
        self.history.record(&self.doc.project, ctx.input(|i| i.time));
        if let Some(nav) = nav {
            self.navigate(nav);
            self.history.sync(&self.doc.project);
        }
        self.confirm_dialog(&ctx);
    }
}

impl App {
    fn handle_shortcuts(&mut self, ctx: &egui::Context, typing: bool) {
        let pressed = |s| ctx.input_mut(|i| i.consume_shortcut(&s));
        if pressed(keys::NEW) {
            self.request(ctx, Discarding::New);
        }
        if pressed(keys::OPEN) {
            self.request(ctx, Discarding::Open);
        }
        if pressed(keys::SAVE_AS) {
            self.save_file(ctx, true);
        } else if pressed(keys::SAVE) {
            self.save_file(ctx, false);
        }
        if pressed(keys::EXPORT) {
            self.export_pdf(ctx);
        }
        if pressed(keys::QUIT) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        for (shortcut, screen) in [
            (keys::SHOW_SHEET, Screen::Sheet),
            (keys::SHOW_EDITOR, Screen::Editor),
            (keys::SHOW_PICKER, Screen::Picker),
        ] {
            if pressed(shortcut) {
                self.screen = screen;
            }
        }
        // While typing, text fields undo their own edits.
        if !typing {
            if pressed(keys::REDO) || pressed(keys::REDO_ALT) {
                self.undo(true);
            } else if pressed(keys::UNDO) {
                self.undo(false);
            }
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
}
