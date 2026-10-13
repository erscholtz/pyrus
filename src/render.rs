use std::path::PathBuf;

use krilla::Document;
use krilla::action::Action;
use krilla::action::LinkAction;
use krilla::annotation::LinkAnnotation;
use krilla::annotation::Target;
use krilla::geom::Point;
use krilla::geom::Rect;
use krilla::page::PageSettings;
use krilla::text::Font;
use krilla::text::TextDirection;

use crate::hir::hir_types::TextType;
use crate::layout::Page;
use crate::layout::RowStyle;
use crate::layout::allocate::Alignment;
use crate::render;

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
        let mut annotations = Vec::new();

        for content in &page_layout.content {
            let mut row_count = 0;
            let (x, y) = content.coord;
            for row in &content.rows {
                let row_format_padding = match &row.row_style {
                    RowStyle::BulletRow => 2 as f32 * 5.66,
                    RowStyle::TrailingBullet => 2 as f32 * 5.66,
                    RowStyle::None => 0 as f32 * 5.66,
                };
                let mut render_x = match row.alignment {
                    Alignment::Right => {
                        let text_len = match row
                            .row
                            .iter()
                            .map(|text| match text {
                                TextType::Text(s) => s.len(),
                                TextType::Bold(s) => s.len(),
                                TextType::Italic(s) => s.len(),
                                TextType::Nerd(s) => s.len(),
                                TextType::Link { label, .. } => label.len(),
                            })
                            .sum::<usize>()
                        {
                            0 => 0,
                            len => len,
                        };
                        let allignment_padding =
                            (page_layout.config.grid_width - x - text_len)
                                as f32
                                * 5.66;
                        page_layout.config.left_margin as f32
                            + (x as f32) * 5.66
                            + row_format_padding
                            + allignment_padding
                    }
                    Alignment::Center => {
                        page_layout.config.left_margin as f32
                            + (x as f32) * 5.66
                            + row_format_padding
                    }
                    Alignment::Left => {
                        page_layout.config.left_margin as f32
                            + (x as f32) * 5.66
                            + row_format_padding
                    }
                };
                let render_y = page_layout.config.top_margin as f32
                    + (y + row_count) as f32 * 10.5;
                if row.row_style == RowStyle::BulletRow {
                    surface.draw_text(
                        Point::from_xy(render_x - 2.0 * 5.66, render_y),
                        nerd_font_reg.clone(),
                        10.0,
                        "•",
                        true,
                        TextDirection::Auto,
                    );
                }
                for text in &row.row {
                    let (text_string, font) = match text {
                        TextType::Text(s) => (s, &serif_font_reg),
                        TextType::Bold(s) => (s, &serif_font_bold),
                        TextType::Italic(s) => (s, &serif_font_reg),
                        TextType::Nerd(s) => (s, &nerd_font_reg),
                        TextType::Link { label, href } => {
                            let target = Target::Action(
                                LinkAction::new(href.clone()).into(),
                            );
                            let annotation = LinkAnnotation::new(
                                Rect::from_xywh(
                                    render_x,
                                    render_y - 10 as f32,
                                    label.len() as f32 * 5.66,
                                    10 as f32,
                                )
                                .unwrap(),
                                target,
                            );
                            annotations.push(annotation.into());
                            (label, &serif_font_reg)
                        }
                    };
                    surface.draw_text(
                        Point::from_xy(render_x, render_y),
                        font.to_owned(),
                        10.0,
                        &text_string,
                        false,
                        TextDirection::Auto,
                    );
                    render_x += text_string.len() as f32 * 5.66;
                }
                row_count += 1;
            }
        }
        surface.finish();
        for annotation in annotations {
            page.add_annotation(annotation);
        }
        page.finish();

        let pdf_bytes = document.finish().unwrap();
        std::fs::write(output_path, pdf_bytes).unwrap();
    }
}
