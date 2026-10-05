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
            for row in &content.content {
                let y = 20.0 + i as f32 * 10.0;
                surface.draw_text(
                    Point::from_xy(18.0, y),
                    serif_font_reg.clone(),
                    10.0,
                    &row,
                    false,
                    TextDirection::Auto,
                );
            }
            i += 1;
        }
        surface.finish();
        page.finish();

        let pdf_bytes = document.finish().unwrap();
        std::fs::write(output_path, pdf_bytes).unwrap();
    }
}
