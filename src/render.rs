use std::path::PathBuf;

use krilla::Document;
use krilla::action::LinkAction;
use krilla::annotation::{LinkAnnotation, Target};
use krilla::color::luma;
use krilla::paint::Fill;
use krilla::geom::{PathBuilder, Point, Rect};
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};

use crate::hir::hir_types::TextType;
use crate::layout::allocate::Alignment;
use crate::layout::{BULLET_INDENT, BlockContentType, Page, RowStyle, font_size};

// Pure black text on white reads harsh; a near-black and a light grey rule
// keep the hierarchy without the contrast shouting.
const TEXT_GREY: u8 = 40;
const RULE_GREY: u8 = 165;
const RULE_THICKNESS: f32 = 0.4;

fn fill(lightness: u8) -> Fill {
    Fill {
        paint: luma::Color::new(lightness).into(),
        ..Fill::default()
    }
}

pub struct Renderer {}

impl Renderer {
    pub fn render_pdf(page_layout: &Page, output_path: &str) {
        // TODO: add krilla::outline to specific items that are title attributes
        let mut document = Document::new();
        // FIXME super inefficient importing the fonts again, use the warm versions in shaping
        let serif_font_reg = {
            let path = PathBuf::from("fonts/newsreader/static/Newsreader_9pt-Regular.ttf");
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let serif_font_bold = {
            let path = PathBuf::from("fonts/newsreader/static/Newsreader_9pt-Medium.ttf");
            let data = std::fs::read(&path).unwrap();
            Font::new(data.into(), 0).unwrap()
        };
        let serif_font_italic = {
            let path = PathBuf::from("fonts/newsreader/static/Newsreader_9pt-Italic.ttf");
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
        let mut page = document.start_page_with(PageSettings::from_wh(width, height).unwrap());
        let mut surface = page.surface();
        surface.set_fill(Some(fill(TEXT_GREY)));
        let mut annotations = Vec::new();

        for content in &page_layout.content {
            let (x, y) = content.coord;
            let rows = match &content.content {
                BlockContentType::Text(rows) => rows,
                BlockContentType::Seperator(width) => {
                    // TEMP: full-width hairline rule, drawn lighter than the text.
                    let left = page_layout.config.left_margin + x;
                    let top = page_layout.config.top_margin + y;
                    if *width > 0.0 {
                        let mut path = PathBuilder::new();
                        path.move_to(left, top);
                        path.line_to(left + width, top);
                        path.line_to(left + width, top + RULE_THICKNESS);
                        path.line_to(left, top + RULE_THICKNESS);
                        path.close();
                        if let Some(path) = path.finish() {
                            surface.set_fill(Some(fill(RULE_GREY)));
                            surface.draw_path(&path);
                            surface.set_fill(Some(fill(TEXT_GREY)));
                        }
                    }
                    continue;
                }
            };
            for (row_count, row) in rows.iter().enumerate() {
                let row_format_padding = match row.row_style {
                    RowStyle::BulletRow | RowStyle::TrailingBullet => BULLET_INDENT,
                    RowStyle::None => 0.0,
                };
                let text_width: f32 = row.lines.iter().map(|text| text.width).sum();
                let remaining = (content.width - row_format_padding - text_width).max(0.0);
                let alignment_padding = match row.alignment {
                    Alignment::Right => remaining,
                    Alignment::Center => remaining / 2.0,
                    Alignment::Left => 0.0,
                };
                let font_size = font_size(&row.size);
                let mut render_x =
                    page_layout.config.left_margin + x + row_format_padding + alignment_padding;
                let render_y =
                    page_layout.config.top_margin + y + row_count as f32 * row.height + font_size;
                if row.row_style == RowStyle::BulletRow {
                    surface.draw_text(
                        Point::from_xy(render_x - BULLET_INDENT, render_y),
                        nerd_font_reg.clone(),
                        font_size,
                        "•",
                        false,
                        TextDirection::Auto,
                    );
                }
                for text in &row.lines {
                    let (text_string, font) = match &text.content {
                        TextType::Text(s) => (s, &serif_font_reg),
                        TextType::Bold(s) => (s, &serif_font_bold),
                        TextType::Italic(s) => (s, &serif_font_italic),
                        TextType::Nerd(s) => (s, &nerd_font_reg),
                        TextType::Link { label, href } => {
                            let target = Target::Action(LinkAction::new(href.clone()).into());
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

        let metadata = krilla::metadata::Metadata::new()
            .title("todo".to_string()) // FIXME: add metadata to the AST and pass it through to the layout
            .description("todo".to_string())
            .authors(vec!["todo".to_string()])
            .creator("Pyrus".to_string())
            .producer("Pyrus".to_string());
        document.set_metadata(metadata);

        let pdf_bytes = document.finish().unwrap();
        std::fs::write(output_path, pdf_bytes).unwrap();
    }
}
