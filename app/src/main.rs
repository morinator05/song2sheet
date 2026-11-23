//! song2sheet desktop app.

mod app;
mod document;
mod preview;
mod screens;

use model::APP_NAME;

const INITIAL_WINDOW_SIZE: [f32; 2] = [1400.0, 900.0];

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size(INITIAL_WINDOW_SIZE),
        ..Default::default()
    };
    // Optional first argument: a song sheet file to open.
    let app = match std::env::args_os().nth(1) {
        Some(path) => app::App::with_file(std::path::Path::new(&path)),
        None => app::App::default(),
    };
    eframe::run_native(APP_NAME, options, Box::new(|_cc| Ok(Box::new(app))))
}
