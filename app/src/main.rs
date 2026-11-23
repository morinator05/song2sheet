//! song2sheet desktop app.

mod app;
mod document;
mod preview;
mod screens;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("song2sheet")
            .with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    // Optional first argument: a song sheet file to open.
    let app = match std::env::args_os().nth(1) {
        Some(path) => app::App::with_file(std::path::Path::new(&path)),
        None => app::App::default(),
    };
    eframe::run_native("song2sheet", options, Box::new(|_cc| Ok(Box::new(app))))
}
