use std::fs::{self, File};
use std::io::{BufWriter, Write};

use printpdf::{
    Actions, Color, Line, LinePoint, LinkAnnotation, Mm, Op, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Pt, Rect,
};

use crate::backend::render::pdf::fonts::FontRegistry;
use crate::backend::render::pdf::layout::{PdfLayoutEngine, PdfTextMeasure};
use crate::backend::render::pdf::style::{StyleLookup, rgb};
use crate::backend::render::pdf::text::{self, TextRun};
use crate::hir::hir_types::{HIRModule, HirElementOp, StyleAttributes};
use crate::layout::ComputedLayout;

const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const PAGE_HEIGHT_PT: f32 = 842.0;
const PAGE_TEXT_SAFETY_PT: f32 = 14.0;

pub struct PdfRenderer;

struct TextRenderParams<'a> {
    point: Point,
    font: PdfFontHandle,
    parsed_font: Option<&'a printpdf::font::ParsedFont>,
    font_size: f32,
    line_height: f32,
    fill_color: Option<Color>,
    anchor_right: bool,
}

struct FontMeasure<'a> {
    doc: &'a mut PdfDocument,
    fonts: &'a mut FontRegistry,
}

#[derive(Clone, Copy)]
struct PageMetrics {
    top_margin: f32,
    bottom_margin: f32,
    usable_height: f32,
}

impl PdfRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        hlir: HIRModule,
        computed_layouts: &[ComputedLayout],
    ) -> Result<(), std::io::Error> {
        let mut doc = PdfDocument::new("Document");
        let mut fonts = FontRegistry::new();

        let pages = self.setup_pages(&mut doc, &hlir, computed_layouts, &mut fonts);
        let pdf_bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        fs::create_dir_all("generated")?;
        let file = File::create("generated/output.pdf")?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&pdf_bytes)?;

        Ok(())
    }

    fn setup_pages(
        &self,
        doc: &mut PdfDocument,
        hlir: &HIRModule,
        computed_layouts: &[ComputedLayout],
        fonts: &mut FontRegistry,
    ) -> Vec<PdfPage> {
        self.setup_page_ops(doc, hlir, computed_layouts, fonts)
            .into_iter()
            .map(|ops| PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), ops))
            .collect()
    }

    fn setup_page_ops(
        &self,
        doc: &mut PdfDocument,
        hlir: &HIRModule,
        computed_layouts: &[ComputedLayout],
        fonts: &mut FontRegistry,
    ) -> Vec<Vec<Op>> {
        let layouts = {
            let mut measure = FontMeasure { doc, fonts };
            PdfLayoutEngine::new(&mut measure).compute_document_flow(hlir)
        };
        let mut layouts = if layouts.is_empty() {
            computed_layouts.to_vec()
        } else {
            layouts
        };

        let page_metrics = Self::page_metrics(hlir);
        Self::avoid_page_boundary_crossing(&mut layouts, page_metrics);
        let page_count = Self::page_count(&layouts, page_metrics);
        let mut page_ops = vec![Vec::new(); page_count];

        for layout in &layouts {
            if let Some(element) = hlir.elements.get(layout.element_index) {
                let page_index = Self::page_index(layout, page_metrics);
                let page_layout = Self::layout_for_page(layout, page_index, page_metrics);
                self.format_hlir_to_pdf_op(
                    element,
                    hlir,
                    &mut page_ops[page_index],
                    &page_layout,
                    doc,
                    fonts,
                );
            }
        }

        page_ops
    }

    fn page_count(layouts: &[ComputedLayout], metrics: PageMetrics) -> usize {
        layouts
            .iter()
            .map(|layout| Self::page_index_for_y(layout.box_y + layout.box_height, metrics))
            .max()
            .unwrap_or(0)
            + 1
    }

    fn avoid_page_boundary_crossing(layouts: &mut [ComputedLayout], metrics: PageMetrics) {
        let mut accumulated_shift = 0.0;

        for layout in layouts {
            if accumulated_shift != 0.0 {
                Self::shift_layout(layout, accumulated_shift);
            }

            if layout.box_height >= metrics.usable_height {
                continue;
            }

            let page_index = Self::page_index(layout, metrics);
            let page_bottom =
                metrics.top_margin + ((page_index + 1) as f32 * metrics.usable_height);
            let page_limit = page_bottom - metrics.bottom_margin - PAGE_TEXT_SAFETY_PT;
            let layout_bottom = (layout.box_y + layout.box_height)
                .max(layout.y + layout.height + metrics.bottom_margin);
            if layout.box_y < page_limit && layout_bottom > page_limit {
                let shift = page_bottom - layout.box_y;
                Self::shift_layout(layout, shift);
                accumulated_shift += shift;
            }

            let page_index = Self::page_index(layout, metrics);
            let page_offset = page_index as f32 * metrics.usable_height;
            let min_y = metrics.top_margin + PAGE_TEXT_SAFETY_PT;
            let local_y = layout.y - page_offset;
            if page_index > 0 && local_y < min_y {
                let shift = min_y - local_y;
                Self::shift_layout(layout, shift);
                accumulated_shift += shift;
            }
        }
    }

    fn page_index(layout: &ComputedLayout, metrics: PageMetrics) -> usize {
        Self::page_index_for_y(layout.box_y, metrics)
    }

    fn page_index_for_y(y: f32, metrics: PageMetrics) -> usize {
        ((y - metrics.top_margin).max(0.0) / metrics.usable_height).floor() as usize
    }

    fn layout_for_page(
        layout: &ComputedLayout,
        page_index: usize,
        metrics: PageMetrics,
    ) -> ComputedLayout {
        let page_offset = page_index as f32 * metrics.usable_height;
        let mut page_layout = layout.clone();
        page_layout.y -= page_offset;
        page_layout.box_y -= page_offset;
        if let Some(marker_y) = page_layout.marker_y.as_mut() {
            *marker_y -= page_offset;
        }
        page_layout
    }

    fn shift_layout(layout: &mut ComputedLayout, amount: f32) {
        layout.y += amount;
        layout.box_y += amount;
        if let Some(marker_y) = layout.marker_y.as_mut() {
            *marker_y += amount;
        }
    }

    fn page_metrics(hlir: &HIRModule) -> PageMetrics {
        let style = StyleLookup::new(&hlir.document_styles);
        let margins = Self::document_margins(style);
        let usable_height = (PAGE_HEIGHT_PT - margins.0 - margins.1).max(1.0);
        PageMetrics {
            top_margin: margins.0,
            bottom_margin: margins.1,
            usable_height,
        }
    }

    fn document_margins(style: StyleLookup<'_>) -> (f32, f32) {
        let mut top = 0.0;
        let mut bottom = 0.0;

        if let Some(value) = style.raw("margin") {
            let parts: Vec<_> = value
                .split_whitespace()
                .filter_map(crate::backend::render::pdf::style::parse_css_length)
                .collect();
            match parts.as_slice() {
                [all] => {
                    top = *all;
                    bottom = *all;
                }
                [vertical, _horizontal] => {
                    top = *vertical;
                    bottom = *vertical;
                }
                [top_value, _horizontal, bottom_value] => {
                    top = *top_value;
                    bottom = *bottom_value;
                }
                [top_value, _right, bottom_value, _left, ..] => {
                    top = *top_value;
                    bottom = *bottom_value;
                }
                [] => {}
            }
        }

        if let Some(value) = style.length("margin-top") {
            top = value;
        }
        if let Some(value) = style.length("margin-bottom") {
            bottom = value;
        }

        (top, bottom)
    }

    fn format_hlir_to_pdf_op(
        &self,
        element: &HirElementOp,
        hlir: &HIRModule,
        pdf_ops: &mut Vec<Op>,
        layout: &ComputedLayout,
        doc: &mut PdfDocument,
        fonts: &mut FontRegistry,
    ) {
        let default_attrs = StyleAttributes::default();
        let attrs = hlir
            .attributes
            .find_node(element.attributes_ref())
            .map(|node| &node.computed)
            .unwrap_or(&default_attrs);
        let style = StyleLookup::with_fallback(attrs, &hlir.document_styles);
        let font_size = style.font_size();
        let line_height = style.line_height(font_size);
        let resolved_font = fonts.resolve(doc, style);
        let ascent_pt = text::ascent_pt(font_size, resolved_font.face.map(|face| &face.parsed));
        let baseline_y_pt = PAGE_HEIGHT_PT - layout.y - ascent_pt;
        let point = Point {
            x: Pt(layout.x),
            y: Pt(baseline_y_pt),
        };
        let fill_color = style.color();

        self.push_border_ops(pdf_ops, style, layout);

        match element {
            HirElementOp::Text { content, .. } => {
                self.push_text_ops(
                    pdf_ops,
                    content,
                    layout,
                    TextRenderParams {
                        point,
                        font: resolved_font.handle,
                        parsed_font: resolved_font.face.map(|face| &face.parsed),
                        font_size,
                        line_height,
                        fill_color,
                        anchor_right: style.is_text_align_right(),
                    },
                );
                self.push_autolink_annotations(
                    pdf_ops,
                    content,
                    layout,
                    font_size,
                    line_height,
                    resolved_font.face.map(|face| &face.parsed),
                );
            }
            HirElementOp::Link { href, content, .. } => {
                self.push_text_ops(
                    pdf_ops,
                    content,
                    layout,
                    TextRenderParams {
                        point,
                        font: resolved_font.handle,
                        parsed_font: resolved_font.face.map(|face| &face.parsed),
                        font_size,
                        line_height,
                        fill_color,
                        anchor_right: layout.nowrap,
                    },
                );
                if let Some(href) = Self::link_href(href) {
                    pdf_ops.push(Op::LinkAnnotation {
                        link: LinkAnnotation::new(
                            Self::annotation_rect(layout),
                            Actions::uri(href),
                            None,
                            None,
                            None,
                        ),
                    });
                }
            }
            HirElementOp::Separator { .. } => {
                self.push_separator_ops(pdf_ops, layout, fill_color);
            }
            HirElementOp::List { .. }
            | HirElementOp::Section { .. }
            | HirElementOp::Image { .. }
            | HirElementOp::Table { .. } => {}
        }
    }

    fn push_text_ops(
        &self,
        pdf_ops: &mut Vec<Op>,
        content: &str,
        layout: &ComputedLayout,
        params: TextRenderParams<'_>,
    ) {
        let lines = text::wrap_text_with_measure(
            content,
            layout.width,
            params.font_size,
            layout.nowrap,
            |candidate, size| text::measure_text_width(candidate, size, params.parsed_font),
        );

        if let Some(marker) = &layout.marker {
            let marker_x = layout.marker_x.unwrap_or((layout.x - 14.0).max(0.0));
            let marker_y = layout
                .marker_y
                .map(|marker_y| {
                    PAGE_HEIGHT_PT
                        - marker_y
                        - text::ascent_pt(params.font_size, params.parsed_font)
                })
                .unwrap_or(params.point.y.0);

            self.push_single_text_line(
                pdf_ops,
                marker,
                Point {
                    x: Pt(marker_x),
                    y: Pt(marker_y),
                },
                &params,
            );
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let line_width = text::measure_text_width(line, params.font_size, params.parsed_font);
            let line_x = if params.anchor_right {
                layout.x + layout.width - line_width
            } else {
                layout.x
            };
            let line_y = params.point.y.0 - (line_idx as f32 * params.line_height);

            self.push_single_text_line(
                pdf_ops,
                line,
                Point {
                    x: Pt(line_x),
                    y: Pt(line_y),
                },
                &params,
            );
        }
    }

    fn push_single_text_line(
        &self,
        pdf_ops: &mut Vec<Op>,
        line: &str,
        point: Point,
        params: &TextRenderParams<'_>,
    ) {
        pdf_ops.push(Op::StartTextSection);
        pdf_ops.push(Op::SetTextCursor { pos: point });
        text::set_font_ops(
            pdf_ops,
            params.font.clone(),
            params.font_size,
            params.line_height,
        );
        if let Some(col) = params.fill_color.clone() {
            pdf_ops.push(Op::SetFillColor { col });
        }
        let run = TextRun::new(line, params.font.clone(), params.parsed_font);
        pdf_ops.push(Op::ShowText {
            items: vec![run.show_text_item()],
        });
        pdf_ops.push(Op::EndTextSection);
    }

    fn push_autolink_annotations(
        &self,
        pdf_ops: &mut Vec<Op>,
        content: &str,
        layout: &ComputedLayout,
        font_size: f32,
        line_height: f32,
        parsed_font: Option<&printpdf::font::ParsedFont>,
    ) {
        let lines = text::wrap_text_with_measure(
            content,
            layout.width,
            font_size,
            layout.nowrap,
            |candidate, size| text::measure_text_width(candidate, size, parsed_font),
        );

        for (line_idx, line) in lines.iter().enumerate() {
            for range in Self::url_ranges(line) {
                let display_url = &line[range.clone()];
                let Some(href) = Self::normalize_url(display_url) else {
                    continue;
                };

                let prefix_width =
                    text::measure_text_width(&line[..range.start], font_size, parsed_font);
                let url_width = text::measure_text_width(display_url, font_size, parsed_font);
                let rect_y = PAGE_HEIGHT_PT - layout.y - ((line_idx as f32 + 1.0) * line_height);

                pdf_ops.push(Op::LinkAnnotation {
                    link: LinkAnnotation::new(
                        Rect::from_xywh(
                            Pt(layout.x + prefix_width),
                            Pt(rect_y),
                            Pt(url_width),
                            Pt(line_height),
                        ),
                        Actions::uri(href),
                        None,
                        None,
                        None,
                    ),
                });
            }
        }
    }

    fn push_separator_ops(
        &self,
        pdf_ops: &mut Vec<Op>,
        layout: &ComputedLayout,
        fill_color: Option<Color>,
    ) {
        let line_y_pt = PAGE_HEIGHT_PT - layout.y - (layout.height / 2.0);
        let color = fill_color.unwrap_or_else(|| rgb(0.5, 0.5, 0.5));

        pdf_ops.push(Op::SetOutlineColor { col: color });
        pdf_ops.push(Op::SetOutlineThickness {
            pt: Pt(layout.height.max(0.1)),
        });
        pdf_ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Pt(layout.x),
                            y: Pt(line_y_pt),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(layout.x + layout.width),
                            y: Pt(line_y_pt),
                        },
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }

    fn annotation_rect(layout: &ComputedLayout) -> Rect {
        Rect::from_xywh(
            Pt(layout.box_x),
            Pt(PAGE_HEIGHT_PT - layout.box_y - layout.box_height),
            Pt(layout.box_width),
            Pt(layout.box_height),
        )
    }

    fn push_border_ops(
        &self,
        pdf_ops: &mut Vec<Op>,
        style: StyleLookup<'_>,
        layout: &ComputedLayout,
    ) {
        if let Some(border) = style.border("border") {
            self.push_rect_border(pdf_ops, layout, border);
        }

        if let Some(border) = style.border("border-bottom") {
            self.push_bottom_border(pdf_ops, layout, border);
        }
    }

    fn push_rect_border(
        &self,
        pdf_ops: &mut Vec<Op>,
        layout: &ComputedLayout,
        border: crate::backend::render::pdf::style::BorderStyle,
    ) {
        let left = layout.box_x;
        let right = layout.box_x + layout.box_width;
        let top = PAGE_HEIGHT_PT - layout.box_y;
        let bottom = PAGE_HEIGHT_PT - layout.box_y - layout.box_height;

        pdf_ops.push(Op::SetOutlineColor { col: border.color });
        pdf_ops.push(Op::SetOutlineThickness {
            pt: Pt(border.width),
        });
        pdf_ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Pt(left),
                            y: Pt(top),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(right),
                            y: Pt(top),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(right),
                            y: Pt(bottom),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(left),
                            y: Pt(bottom),
                        },
                        bezier: false,
                    },
                ],
                is_closed: true,
            },
        });
    }

    fn push_bottom_border(
        &self,
        pdf_ops: &mut Vec<Op>,
        layout: &ComputedLayout,
        border: crate::backend::render::pdf::style::BorderStyle,
    ) {
        let y = PAGE_HEIGHT_PT - layout.box_y - layout.box_height;

        pdf_ops.push(Op::SetOutlineColor { col: border.color });
        pdf_ops.push(Op::SetOutlineThickness {
            pt: Pt(border.width),
        });
        pdf_ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point {
                            x: Pt(layout.box_x),
                            y: Pt(y),
                        },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point {
                            x: Pt(layout.box_x + layout.box_width),
                            y: Pt(y),
                        },
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }

    fn url_ranges(line: &str) -> Vec<std::ops::Range<usize>> {
        let mut ranges = Vec::new();
        let mut token_start = None;

        for (idx, ch) in line.char_indices() {
            if ch.is_whitespace() {
                if let Some(start) = token_start.take() {
                    Self::push_url_range(line, start, idx, &mut ranges);
                }
            } else if token_start.is_none() {
                token_start = Some(idx);
            }
        }

        if let Some(start) = token_start {
            Self::push_url_range(line, start, line.len(), &mut ranges);
        }

        ranges
    }

    fn push_url_range(
        line: &str,
        start: usize,
        end: usize,
        ranges: &mut Vec<std::ops::Range<usize>>,
    ) {
        let token = &line[start..end];
        let trimmed_start = token
            .char_indices()
            .find(|(_, ch)| ch.is_ascii_alphanumeric())
            .map(|(idx, _)| idx)
            .unwrap_or(token.len());
        let trimmed_end = token
            .char_indices()
            .rev()
            .find(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '/')
            .map(|(idx, ch)| idx + ch.len_utf8())
            .unwrap_or(trimmed_start);

        if trimmed_start >= trimmed_end {
            return;
        }

        let candidate = &token[trimmed_start..trimmed_end];
        if Self::normalize_url(candidate).is_some() {
            ranges.push((start + trimmed_start)..(start + trimmed_end));
        }
    }

    fn normalize_url(candidate: &str) -> Option<String> {
        if candidate.starts_with("http://") || candidate.starts_with("https://") {
            return Some(candidate.to_string());
        }

        let lower = candidate.to_ascii_lowercase();
        if lower.starts_with("github.com/")
            || lower.starts_with("linkedin.com/")
            || lower.starts_with("www.")
        {
            return Some(format!("https://{candidate}"));
        }

        None
    }

    fn link_href(candidate: &str) -> Option<String> {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            return None;
        }

        Self::normalize_url(candidate).or_else(|| Some(candidate.to_string()))
    }
}

impl PdfTextMeasure for FontMeasure<'_> {
    fn measure_element_text(
        &mut self,
        hlir: &HIRModule,
        element_index: usize,
        text_value: &str,
        font_size: f32,
    ) -> f32 {
        let Some(element) = hlir.elements.get(element_index) else {
            return text::measure_text_width(text_value, font_size, None);
        };
        let Some(attrs) = hlir
            .attributes
            .find_node(element.attributes_ref())
            .map(|node| &node.computed)
        else {
            return text::measure_text_width(text_value, font_size, None);
        };

        let resolved = self.fonts.resolve(
            self.doc,
            StyleLookup::with_fallback(attrs, &hlir.document_styles),
        );
        text::measure_text_width(
            text_value,
            font_size,
            resolved.face.map(|face| &face.parsed),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::render::pdf::text::sanitize_builtin_text;

    use super::PdfRenderer;

    #[test]
    fn url_ranges_trim_punctuation() {
        assert_eq!(
            PdfRenderer::url_ranges("(github.com/example),"),
            vec![1..19]
        );
    }

    #[test]
    fn builtin_sanitization_still_replaces_unicode_punctuation() {
        assert_eq!(
            sanitize_builtin_text("September 2025 – Present · Toronto"),
            "September 2025 - Present | Toronto"
        );
    }

    #[test]
    fn empty_link_href_is_not_annotated() {
        assert_eq!(PdfRenderer::link_href(""), None);
    }
}
