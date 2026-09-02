use std::fs::{self, File};
use std::io::{BufWriter, Write};

use printpdf::{
    Actions, BuiltinFont, Color, FontId, Line, LinePoint, LinkAnnotation, Mm,
    Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, Rect,
    Rgb, TextItem, font::ParsedFont,
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
                self.format_hlir_to_pdf_op(
                    element.clone(),
                    &hlir,
                    &mut pdf_ops,
                    layout,
                    fonts,
                );
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
                    &content,
                    layout,
                    font_size,
                    line_height,
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
                pdf_ops.push(Op::LinkAnnotation {
                    link: LinkAnnotation::new(
                        Self::annotation_rect(layout, page_height_pt),
                        Actions::uri(
                            Self::normalize_url(&href).unwrap_or(href),
                        ),
                        None,
                        None,
                        None,
                    ),
                });
            }
            HirElementOp::List { .. } => {
                // TODO: Container elements don't render directly
            }
            HirElementOp::Section { .. } => {
                // TODO: Container elements don't render directly
            }
            HirElementOp::Image { .. } => {
                // TODO: Render image
            }
            HirElementOp::Table { .. } => {
                // TODO: Render table
            }
            HirElementOp::Separator { .. } => {
                let line_y_pt =
                    page_height_pt - layout.y - (layout.height / 2.0);
                let color =
                    fill_color.unwrap_or_else(|| Self::rgb(0.5, 0.5, 0.5));

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
            let marker_x_mm =
                layout.marker_x.unwrap_or((layout.x - 14.0).max(0.0)) / 2.83465;
            let marker_y_mm = layout
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
            let line_width = Self::measure_text_width(
                line,
                params.font_size,
                params.parsed_font,
            );
            let line_x = if params.anchor_right {
                layout.x + layout.width - line_width
            } else {
                layout.x
            };
            let line_y =
                params.point.y.0 - (line_idx as f32 * params.line_height);
            let line_point = Point {
                x: Pt(line_x),
                y: Pt(line_y),
            };

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
        let lines = LayoutEngine::wrap_text_with_mode(
            content,
            layout.width,
            font_size,
            layout.nowrap,
        );

        for (line_idx, line) in lines.iter().enumerate() {
            for range in Self::url_ranges(line) {
                let display_url = &line[range.clone()];
                let Some(href) = Self::normalize_url(display_url) else {
                    continue;
                };

                let prefix_width = LayoutEngine::estimate_text_width(
                    &line[..range.start],
                    font_size,
                );
                let url_width =
                    LayoutEngine::estimate_text_width(display_url, font_size);
                let rect_y =
                    842.0 - layout.y - ((line_idx as f32 + 1.0) * line_height);

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
        if let Some(border) = Self::parse_border(
            attrs.style.get("border").map(String::as_str),
            attrs,
        ) {
            self.push_rect_border(pdf_ops, layout, page_height_pt, border);
        }

        if let Some(border) = Self::parse_border(
            attrs.style.get("border-bottom").map(String::as_str),
            attrs,
        ) {
            self.push_bottom_border(pdf_ops, layout, page_height_pt, border);
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
        if candidate.starts_with("http://") || candidate.starts_with("https://")
        {
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

    /// Parse font-size from computed styles, defaulting to 12pt
    fn parse_font_size(attrs: &StyleAttributes) -> f32 {
        attrs
            .style
            .get("font-size")
            .and_then(|v| Self::parse_css_length(v))
            .unwrap_or(12.0)
    }

    fn parse_line_height(attrs: &StyleAttributes, font_size: f32) -> f32 {
        attrs
            .style
            .get("line-height")
            .and_then(|value| {
                let value = value.trim();
                let num_end = value
                    .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                    .unwrap_or(value.len());

                if num_end == value.len() {
                    value
                        .parse::<f32>()
                        .ok()
                        .map(|multiple| multiple * font_size)
                } else {
                    Self::parse_css_length(value)
                }
            })
            .unwrap_or(font_size * 1.2)
    }

    fn load_external_fonts(doc: &mut PdfDocument) -> FontRegistry {
        FontRegistry {
            georgia: Self::load_font_face(
                doc,
                "C:\\Windows\\Fonts\\georgia.ttf",
            ),
            georgia_bold: Self::load_font_face(
                doc,
                "C:\\Windows\\Fonts\\georgiab.ttf",
            ),
        }
    }

    fn load_font_face(doc: &mut PdfDocument, path: &str) -> Option<FontFace> {
        let bytes = std::fs::read(path).ok()?;
        let mut warnings = Vec::new();
        let parsed = ParsedFont::from_bytes(&bytes, 0, &mut warnings)?;
        let id = doc.add_font(&parsed);
        Some(FontFace { id, parsed })
    }

    /// Get PDF font from font-family and font-weight CSS properties.
    fn get_font(
        attrs: &StyleAttributes,
        font_face: Option<&FontFace>,
    ) -> PdfFontHandle {
        if let Some(face) = font_face {
            return PdfFontHandle::External(face.id.clone());
        }

        let family = Self::font_family(attrs);
        let is_bold = Self::is_bold(attrs);
        if matches!(family.as_str(), "georgia") {
            return if is_bold {
                PdfFontHandle::Builtin(BuiltinFont::TimesBold)
            } else {
                PdfFontHandle::Builtin(BuiltinFont::TimesRoman)
            };
        }

        let font = match family.as_str() {
            "times" | "times new roman" | "serif" if is_bold => {
                BuiltinFont::TimesBold
            }
            "times" | "times new roman" | "serif" => BuiltinFont::TimesRoman,
            "courier" | "courier new" | "monospace" if is_bold => {
                BuiltinFont::CourierBold
            }
            "courier" | "courier new" | "monospace" => BuiltinFont::Courier,
            _ if is_bold => BuiltinFont::HelveticaBold,
            _ => BuiltinFont::Helvetica,
        };

        PdfFontHandle::Builtin(font)
    }

    fn get_font_face<'a>(
        attrs: &StyleAttributes,
        fonts: &'a FontRegistry,
    ) -> Option<&'a FontFace> {
        let family = Self::font_family(attrs);
        if family != "georgia" {
            return None;
        }

        if Self::is_bold(attrs) {
            fonts.georgia_bold.as_ref().or(fonts.georgia.as_ref())
        } else {
            fonts.georgia.as_ref()
        }
    }

    fn font_family(attrs: &StyleAttributes) -> String {
        attrs
            .style
            .get("font-family")
            .map(|v| v.trim().trim_matches('"').to_lowercase())
            .unwrap_or_else(|| "helvetica".to_string())
    }

    fn is_bold(attrs: &StyleAttributes) -> bool {
        attrs
            .style
            .get("font-weight")
            .map(|value| {
                let normalized = value.trim().trim_matches('"').to_lowercase();
                normalized == "bold"
                    || normalized == "bolder"
                    || normalized
                        .parse::<u16>()
                        .is_ok_and(|weight| weight >= 600)
            })
            .unwrap_or(false)
    }

    fn measure_text_width(
        text: &str,
        font_size: f32,
        parsed_font: Option<&ParsedFont>,
    ) -> f32 {
        let Some(font) = parsed_font else {
            return LayoutEngine::estimate_text_width(text, font_size);
        };

        let units_per_em = font.font_metrics.units_per_em as f32;
        if units_per_em <= 0.0 {
            return LayoutEngine::estimate_text_width(text, font_size);
        }

        let width_units = text
            .chars()
            .filter_map(|ch| font.lookup_glyph_index(ch as u32))
            .map(|glyph_id| font.get_horizontal_advance(glyph_id) as f32)
            .sum::<f32>();

        width_units * font_size / units_per_em
    }

    fn is_text_align_right(attrs: &StyleAttributes) -> bool {
        attrs
            .style
            .get("text-align")
            .is_some_and(|value| value.trim().trim_matches('"') == "right")
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

        Some(color)
    }

    fn parse_border(
        value: Option<&str>,
        attrs: &StyleAttributes,
    ) -> Option<BorderStyle> {
        let value = value?.trim().trim_matches('"');
        if value.is_empty() || value.eq_ignore_ascii_case("none") {
            return None;
        }

        let mut width = None;
        let mut color = None;

        for part in value.split_whitespace() {
            if width.is_none() {
                width = Self::parse_css_length(part);
            }

            if color.is_none() {
                color = Self::parse_color_token(part);
            }
        }

        Some(BorderStyle {
            width: width.unwrap_or(1.0),
            color: color
                .or_else(|| Self::parse_color(attrs))
                .unwrap_or_else(|| Self::rgb(0.0, 0.0, 0.0)),
        })
    }

    fn sanitize_builtin_text(value: &str) -> String {
        value
            .chars()
            .map(|ch| match ch {
                '\u{2013}' | '\u{2014}' | '\u{2011}' | '\u{2010}' => '-',
                '\u{00b7}' | '\u{2022}' => '|',
                '\u{2018}' | '\u{2019}' => '\'',
                '\u{201c}' | '\u{201d}' => '"',
                '\u{00a0}' => ' ',
                _ => ch,
            })
            .collect()
    }

    fn parse_color_token(value: &str) -> Option<Color> {
        let value = value.trim().trim_matches('"');
        let color = match value.to_lowercase().as_str() {
            "black" => Self::rgb(0.0, 0.0, 0.0),
            "white" => Self::rgb(1.0, 1.0, 1.0),
            "red" => Self::rgb(1.0, 0.0, 0.0),
            "green" => Self::rgb(0.0, 0.5, 0.0),
            "blue" => Self::rgb(0.0, 0.0, 1.0),
            "gray" | "grey" => Self::rgb(0.5, 0.5, 0.5),
            _ => return Self::parse_hex_color(value),
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
            PdfRenderer::sanitize_builtin_text(
                "September 2025 – Present · Toronto"
            ),
            "September 2025 - Present | Toronto"
        );
    }

    #[test]
    fn empty_link_href_is_not_annotated() {
        assert_eq!(PdfRenderer::link_href(""), None);
    }
}
