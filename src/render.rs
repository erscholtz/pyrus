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
            .start_page_with(PageSettings::from_wh(200.0, 200.0).unwrap());
        let mut surface = page.surface();
        let content = &page_layout.content[0];
        let row = &content.content[0];
        let text: String = row.glyphs.iter().map(|glyph| glyph.char).collect();
        surface.draw_text(
            Point::from_xy(18.0, 20.0),
            serif_font_reg.clone(),
            10.0,
            &text,
            false,
            TextDirection::Auto,
        );
        surface.finish();
        page.finish();

        let pdf_bytes = document.finish().unwrap();
        std::fs::write(output_path, pdf_bytes).unwrap();
    }
}
