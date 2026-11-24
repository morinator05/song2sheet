//! song2sheet desktop app.

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
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size(INITIAL_WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
        ..Default::default()
    };
    // Optional first argument: a song sheet file to open.
    let file = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, file.as_deref())))),
    )
}
