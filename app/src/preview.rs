//! Live preview of the sheet: the same display list as the PDF, rasterized
//! and cached as egui textures until the project, filter or size changes.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use model::{PartFilter, Project};

use crate::screens::widgets::{BLOCK_GAP, image_widths};

#[derive(Default)]
pub struct Preview {
    /// What the current textures were rendered from.
    rendered: Option<(Project, PartFilter, f32)>,
    pages: Vec<TextureHandle>,
}

impl Preview {
    /// Shows all pages scaled to the available width.
    pub fn show(&mut self, ui: &mut egui::Ui, project: &Project, filter: PartFilter) {
        let (width, px_width) = image_widths(ui);
        let up_to_date = self
            .rendered
            .as_ref()
            .is_some_and(|(p, f, w)| p == project && *f == filter && *w == px_width);
        if !up_to_date {
            self.render(ui.ctx(), project, filter, px_width);
        }

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            for tex in &self.pages {
                let size = tex.size_vec2();
                let scaled = egui::vec2(width, width * size.y / size.x);
                ui.add(egui::Image::new((tex.id(), scaled)));
                ui.add_space(BLOCK_GAP);
            }
        });
    }

    fn render(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        filter: PartFilter,
        px_width: f32,
    ) {
        let list = engrave::layout_sheet(project, filter);
        self.pages = list
            .pages
            .iter()
            .enumerate()
            .map(|(i, page)| {
                let pixmap = raster::render(page, px_width / page.width);
                let size = [pixmap.width() as usize, pixmap.height() as usize];
                let image = ColorImage::from_rgba_premultiplied(size, pixmap.data());
                ctx.load_texture(format!("sheet-page-{i}"), image, TextureOptions::LINEAR)
            })
            .collect();
        self.rendered = Some((project.clone(), filter, px_width));
    }
}
