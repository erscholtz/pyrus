use std::path::PathBuf;

use krilla::{
    Document,
    geom::Point,
    page::PageSettings,
    text::{Font, TextDirection},
};

use crate::layout::Page;

pub struct Renderer {}

impl Renderer {
    pub fn render_pdf(page_layout: &Page, output_path: &str) {
        let mut document = Document::new();
        let serif_font_reg = {
            let path = PathBuf::from(
                "src/fonts/new-heterodox-mono/NewHeterodoxMono-Book.otf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let serif_font_bold = {
            let path = PathBuf::from(
                "src/fonts/new-heterodox-mono/NewHeterodoxMono-Bold.otf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };

        let nerd_font_reg = {
            let path = PathBuf::from(
                "src/fonts/inconsolata-nerd-font-mono/InconsolataNerdFontMono-Regular.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let nerd_font_bold = {
            let path = PathBuf::from(
                "src/fonts/inconsolata-nerd-font-mono/InconsolataNerdFontMono-Bold.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };

        let mut page = document
            .start_page_with(PageSettings::from_wh(595.28, 841.89).unwrap());
        let mut surface = page.surface();

        let mut i = 0;
        for content in &page_layout.content {
            let mut row_count = 0;
            let (x, y) = content.coord;
            for row in &content.content {
                let render_x =
                    page_layout.config.left_margin as f32 + x as f32 * 5.66;
                let render_y = page_layout.config.top_margin as f32
                    + (y + row_count) as f32 * 10.5;
                surface.draw_text(
                    Point::from_xy(render_x, render_y),
                    serif_font_reg.clone(),
                    10.0,
                    &row,
                    false,
                    TextDirection::Auto,
                );
                row_count += 1;
            }
            i += row_count;
        }
        surface.finish();
        page.finish();

        let pdf_bytes = document.finish().unwrap();
        std::fs::write(output_path, pdf_bytes).unwrap();
    }
}
