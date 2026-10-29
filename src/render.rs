use std::path::PathBuf;

use krilla::Document;
use krilla::action::LinkAction;
use krilla::annotation::{LinkAnnotation, Target};
use krilla::geom::{PathBuilder, Point, Rect};
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};

use crate::hir::hir_types::{Sizing, TextType};
use crate::layout::allocate::Alignment;
use crate::layout::{BULLET_INDENT, ContentBoxContents, Page, RowStyle};

pub struct Renderer {}

impl Renderer {
    pub fn render_pdf(page_layout: &Page, output_path: &str) {
        let mut document = Document::new();
        // FIXME super inefficient importing the fonts again, use the warm versions in shaping
        let serif_font_reg = {
            let path = PathBuf::from(
                "fonts/newsreader/static/Newsreader_9pt-Light.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let serif_font_bold = {
            let path = PathBuf::from(
                "fonts/newsreader/static/Newsreader_9pt-Medium.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let serif_font_italic = {
            let path = PathBuf::from(
                "fonts/newsreader/static/Newsreader_9pt-LightItalic.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let nerd_font_reg = {
            let path = PathBuf::from(
                "fonts/inconsolata-nerd-font-mono/InconsolataNerdFontMono-Regular.ttf",
            );
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };

        let (width, height) = crate::layout::calculate_grid(
            &page_layout.config.page_type,
            &page_layout.config.orientation,
        )
        .unwrap();
        let mut page = document
            .start_page_with(PageSettings::from_wh(width, height).unwrap());
        let mut surface = page.surface();
        let mut annotations = Vec::new();

        for content in &page_layout.content {
            let (x, y) = content.coord;
            let rows = match &content.content {
                ContentBoxContents::Text(rows) => rows,
                ContentBoxContents::Seperator(width) => {
                    // TEMP: full-width underline as a 0.5pt black rectangle.
                    let left = page_layout.config.left_margin + x;
                    let top = page_layout.config.top_margin + y;
                    if *width > 0.0 {
                        let mut path = PathBuilder::new();
                        path.move_to(left, top);
                        path.line_to(left + width, top);
                        path.line_to(left + width, top + 0.5);
                        path.line_to(left, top + 0.5);
                        path.close();
                        if let Some(path) = path.finish() {
                            surface.draw_path(&path);
                        }
                    }
                    continue;
                }
            };
            for (row_count, row) in rows.iter().enumerate() {
                let row_format_padding = match row.row_style {
                    RowStyle::BulletRow | RowStyle::TrailingBullet => {
                        BULLET_INDENT
                    }
                    RowStyle::None => 0.0,
                };
                let text_width: f32 =
                    row.row.iter().map(|text| text.width).sum();
                let remaining =
                    (content.width - row_format_padding - text_width).max(0.0);
                let alignment_padding = match row.alignment {
                    Alignment::Right => remaining,
                    Alignment::Center => remaining / 2.0,
                    Alignment::Left => 0.0,
                };
                let font_size = match row.size {
                    Sizing::Sm => 10 as f32,
                    Sizing::Md => 12 as f32,
                    Sizing::Lg => 14 as f32,
                    Sizing::Xl => 24 as f32,
                };
                let mut render_x = page_layout.config.left_margin
                    + x
                    + row_format_padding
                    + alignment_padding;
                let render_y = page_layout.config.top_margin
                    + y
                    + row_count as f32 * row.height
                    + font_size;
                if row.row_style == RowStyle::BulletRow {
                    surface.draw_text(
                        Point::from_xy(render_x - BULLET_INDENT, render_y),
                        nerd_font_reg.clone(),
                        font_size,
                        "•",
                        true,
                        TextDirection::Auto,
                    );
                }
                for text in &row.row {
                    let (text_string, font) = match &text.content {
                        TextType::Text(s) => (s, &serif_font_reg),
                        TextType::Bold(s) => (s, &serif_font_bold),
                        TextType::Italic(s) => (s, &serif_font_italic),
                        TextType::Nerd(s) => (s, &nerd_font_reg),
                        TextType::Link { label, href } => {
                            let target = Target::Action(
                                LinkAction::new(href.clone()).into(),
                            );
                            if text.width > 0.0 {
                                let annotation = LinkAnnotation::new(
                                    Rect::from_xywh(
                                        render_x,
                                        render_y - font_size,
                                        text.width,
                                        font_size,
                                    )
                                    .unwrap(),
                                    target,
                                );
                                annotations.push(annotation.into());
                            }
                            (label, &serif_font_reg)
                        }
                    };
                    surface.draw_glyphs(
                        Point::from_xy(render_x, render_y),
                        &text.glyphs,
                        font.to_owned(),
                        text_string,
                        font_size,
                        false,
                    );
                    render_x += text.width;
                }
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
