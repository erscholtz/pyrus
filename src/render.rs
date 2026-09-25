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
    pub fn render_pdf(&self, page_layout: &Page, output_path: &str) {
        let mut document = Document::new();
        let serif_font_reg = {
            let path = PathBuf::from(
                "fonts/new-heterodox-mono/NewHeterodoxMono-Regular.otf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let serif_font_bold = {
            let path = PathBuf::from(
                "fonts/new-heterodox-mono/NewHeterodoxMono-Bold.otf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 1).unwrap()
        };

        let nerd_font_reg = {
            let path = PathBuf::from(
                "fonts/inconsolata-nerd-font-mono/InconsolataNerdFontMono-Regular.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 2).unwrap()
        };
        let nerd_font_bold = {
            let path = PathBuf::from(
                "fonts/inconsolata-nerd-font-mono/InconsolataNerdFontMono-Bold.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 3).unwrap()
        };

        let mut page = document
            .start_page_with(PageSettings::from_wh(200.0, 200.0).unwrap());

        for content in &page_layout.content {
            page.surface().draw_text(
                Point::from_xy(0.0, 0.0),
                serif_font_reg,
                12.0,
                content.content.clone(),
                false,
                TextDirection::RightToLeft,
            );
        }
    }
}
