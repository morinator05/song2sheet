//! song2sheet desktop app.

// Release builds on Windows open no console window next to the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod document;
mod history;
mod preview;
mod recent;
mod screens;
mod shortcuts;
mod status;
mod theme;

use model::APP_NAME;

const INITIAL_WINDOW_SIZE: [f32; 2] = [1440.0, 900.0];
/// Below this the three panels no longer fit side by side.
const MIN_WINDOW_SIZE: [f32; 2] = [1100.0, 600.0];

fn main() -> eframe::Result {
    // The app id lets Plasma match the window to song2sheet.desktop (and
    // its icon) on Wayland; the window icon covers X11.
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_app_id(APP_NAME)
        .with_inner_size(INITIAL_WINDOW_SIZE)
        .with_min_inner_size(MIN_WINDOW_SIZE);
    if let Some(icon) = theme::icon_theme::app_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    // Optional first argument: a song sheet file to open.
    let file = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, file.as_deref())))),
    )
}
