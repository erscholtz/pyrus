use std::fs::{self, File};
use std::io::BufWriter;
use std::io::Write;

use printpdf::{
    Actions, BuiltinFont, Color, FontId, Line, LinePoint, LinkAnnotation, Mm, Op, PdfDocument,
    PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, Rect, Rgb, TextItem, font::ParsedFont,
};

use crate::hir::hir_types::{HIRModule, HirElementOp, StyleAttributes};
use crate::layout::{ComputedLayout, LayoutEngine};

pub struct PdfRenderer;

#[derive(Clone)]
struct BorderStyle {
    width: f32,
    color: Color,
}

struct FontFace {
    id: FontId,
    parsed: ParsedFont,
}

#[derive(Default)]
struct FontRegistry {
    georgia: Option<FontFace>,
    georgia_bold: Option<FontFace>,
}

struct TextRenderParams<'a> {
    point: Point,
    font: PdfFontHandle,
    parsed_font: Option<&'a ParsedFont>,
    font_size: f32,
    line_height: f32,
    fill_color: Option<Color>,
    y_mm: f32,
    sanitize_text: bool,
    anchor_right: bool,
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
        let fonts = Self::load_external_fonts(&mut doc);

        let pages = self.setup_pages(hlir, computed_layouts, &fonts);
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
        hlir: HIRModule,
        computed_layouts: &[ComputedLayout],
        fonts: &FontRegistry,
    ) -> Vec<PdfPage> {
        let mut pages = Vec::new();

        let ops = self.setup_ops(hlir, computed_layouts, fonts);
        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
        pages.push(page);

        pages
    }

    fn setup_ops(
        &self,
        hlir: HIRModule,
        computed_layouts: &[ComputedLayout],
        fonts: &FontRegistry,
    ) -> Vec<Op> {
        let mut pdf_ops = Vec::new();

        // Render each element with its computed layout
        for layout in computed_layouts {
            if let Some(element) = hlir.elements.get(layout.element_index) {
                self.format_hlir_to_pdf_op(element.clone(), &hlir, &mut pdf_ops, layout, fonts);
            }
        }

        pdf_ops
    }

    fn format_hlir_to_pdf_op(
        &self,
        element: HirElementOp,
        hlir: &HIRModule,
        pdf_ops: &mut Vec<Op>,
        layout: &ComputedLayout,
        fonts: &FontRegistry,
    ) {
        // Convert layout coordinates (points) to PDF coordinates (Mm)
        // Note: PDF y-coordinate starts from bottom, layout y starts from top
        let page_height_pt = 842.0; // A4 height in points

        // Get font size first so we can adjust for baseline
        let default_attrs = StyleAttributes::default();
        let attrs = match &element {
            HirElementOp::Text { attributes, .. }
            | HirElementOp::Link { attributes, .. }
            | HirElementOp::Separator { attributes }
            | HirElementOp::List { attributes, .. }
            | HirElementOp::Section { attributes, .. }
            | HirElementOp::Image { attributes, .. }
            | HirElementOp::Table { attributes, .. } => hlir
                .attributes
                .find_node(*attributes)
                .map(|n| &n.computed)
                .unwrap_or(&default_attrs),
        };
        let font_size = Self::parse_font_size(attrs);
        let line_height = Self::parse_line_height(attrs, font_size);
        let font_face = Self::get_font_face(attrs, fonts);
        let font = Self::get_font(attrs, font_face);
        let fill_color = Self::parse_color(attrs);
        let sanitize_text = matches!(font, PdfFontHandle::Builtin(_));

        // Convert x from points to mm
        let x_mm = layout.x / 2.83465;

        // Convert y from points (top-down) to mm (bottom-up)
        // PDF text cursor sets the baseline, not the top of the text
        // We need to adjust y downward by the font ascent (approx 0.8 * font_size for most fonts)
        // to make the text appear at the top of the allocated space
        let ascent_pt = font_size * 0.8; // Approximate ascent (80% of font size)
        let baseline_y_pt = page_height_pt - layout.y - ascent_pt;
        let y_mm = baseline_y_pt / 2.83465;

        let point = Point::new(Mm(x_mm), Mm(y_mm));
        self.push_border_ops(pdf_ops, attrs, layout, page_height_pt);

        match element {
            HirElementOp::Text { content, .. } => {
                self.push_text_ops(
                    pdf_ops,
                    &content,
                    layout,
                    TextRenderParams {
                        point,
                        font,
                        parsed_font: font_face.map(|face| &face.parsed),
                        font_size,
                        line_height,
                        fill_color,
                        y_mm,
                        sanitize_text,
                        anchor_right: Self::is_text_align_right(attrs),
                    },
                );
                self.push_autolink_annotations(pdf_ops, &content, layout, font_size, line_height);
            }
            HirElementOp::Link { href, content, .. } => {
                self.push_text_ops(
                    pdf_ops,
                    &content,
                    layout,
                    TextRenderParams {
                        point,
                        font,
                        parsed_font: font_face.map(|face| &face.parsed),
                        font_size,
                        line_height,
                        fill_color,
                        y_mm,
                        sanitize_text,
                        anchor_right: layout.nowrap,
                    },
                );
                pdf_ops.push(Op::LinkAnnotation {
                    link: LinkAnnotation::new(
                        Self::annotation_rect(layout, page_height_pt),
                        Actions::uri(Self::normalize_url(&href).unwrap_or(href)),
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
                let line_y_pt = page_height_pt - layout.y - (layout.height / 2.0);
                let color = fill_color.unwrap_or_else(|| Self::rgb(0.5, 0.5, 0.5));

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
        }
    }

    fn push_text_ops(
        &self,
        pdf_ops: &mut Vec<Op>,
        content: &str,
        layout: &ComputedLayout,
        params: TextRenderParams<'_>,
    ) {
        let lines = LayoutEngine::wrap_text_with_mode(
            content,
            layout.width,
            params.font_size,
            layout.nowrap,
        );

        if let Some(marker) = &layout.marker {
            let marker_x_mm = layout.marker_x.unwrap_or((layout.x - 14.0).max(0.0)) / 2.83465;
            let marker_y_mm = layout
                .marker_y
                .map(|marker_y| {
                    let ascent_pt = params.font_size * 0.8;
                    (842.0 - marker_y - ascent_pt) / 2.83465
                })
                .unwrap_or(params.y_mm);
            let marker_point = Point::new(Mm(marker_x_mm), Mm(marker_y_mm));

            pdf_ops.push(Op::StartTextSection);
            pdf_ops.push(Op::SetTextCursor { pos: marker_point });
            pdf_ops.push(Op::SetFont {
                font: params.font.clone(),
                size: Pt(params.font_size),
            });
            pdf_ops.push(Op::SetLineHeight {
                lh: Pt(params.line_height),
            });
            if let Some(col) = params.fill_color.clone() {
                pdf_ops.push(Op::SetFillColor { col });
            }
            let marker_text = if params.sanitize_text {
                Self::sanitize_builtin_text(marker)
            } else {
                marker.clone()
            };
            pdf_ops.push(Op::ShowText {
                items: vec![TextItem::Text(marker_text)],
            });
            pdf_ops.push(Op::EndTextSection);
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let line_width = Self::measure_text_width(line, params.font_size, params.parsed_font);
            let line_x = if params.anchor_right {
                layout.x + layout.width - line_width
            } else {
                layout.x
            };
            let line_y = params.point.y.0 - (line_idx as f32 * params.line_height);
            let line_point = Point {
                x: Pt(line_x),
                y: Pt(line_y),
            };

            pdf_ops.push(Op::StartTextSection);
            pdf_ops.push(Op::SetTextCursor { pos: line_point });
            pdf_ops.push(Op::SetFont {
                font: params.font.clone(),
                size: Pt(params.font_size),
            });
            pdf_ops.push(Op::SetLineHeight {
                lh: Pt(params.line_height),
            });
            if let Some(col) = params.fill_color.clone() {
                pdf_ops.push(Op::SetFillColor { col });
            }
            let text = if params.sanitize_text {
                Self::sanitize_builtin_text(line)
            } else {
                line.clone()
            };
            pdf_ops.push(Op::ShowText {
                items: vec![TextItem::Text(text)],
            });
            pdf_ops.push(Op::EndTextSection);
        }
    }

    fn push_autolink_annotations(
        &self,
        pdf_ops: &mut Vec<Op>,
        content: &str,
        layout: &ComputedLayout,
        font_size: f32,
        line_height: f32,
    ) {
        let lines =
            LayoutEngine::wrap_text_with_mode(content, layout.width, font_size, layout.nowrap);

        for (line_idx, line) in lines.iter().enumerate() {
            for range in Self::url_ranges(line) {
                let display_url = &line[range.clone()];
                let Some(href) = Self::normalize_url(display_url) else {
                    continue;
                };

                let prefix_width =
                    LayoutEngine::estimate_text_width(&line[..range.start], font_size);
                let url_width = LayoutEngine::estimate_text_width(display_url, font_size);
                let rect_y = 842.0 - layout.y - ((line_idx as f32 + 1.0) * line_height);

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

    fn annotation_rect(layout: &ComputedLayout, page_height_pt: f32) -> Rect {
        Rect::from_xywh(
            Pt(layout.box_x),
            Pt(page_height_pt - layout.box_y - layout.box_height),
            Pt(layout.box_width),
            Pt(layout.box_height),
        )
    }

    fn push_border_ops(
        &self,
        pdf_ops: &mut Vec<Op>,
        attrs: &StyleAttributes,
        layout: &ComputedLayout,
        page_height_pt: f32,
    ) {
        if let Some(border) =
            Self::parse_border(attrs.style.get("border").map(String::as_str), attrs)
        {
            self.push_rect_border(pdf_ops, layout, page_height_pt, border);
        }

        if let Some(border) =
            Self::parse_border(attrs.style.get("border-bottom").map(String::as_str), attrs)
        {
            self.push_bottom_border(pdf_ops, layout, page_height_pt, border);
        }
    }

    fn push_rect_border(
        &self,
        pdf_ops: &mut Vec<Op>,
        layout: &ComputedLayout,
        page_height_pt: f32,
        border: BorderStyle,
    ) {
        let left = layout.box_x;
        let right = layout.box_x + layout.box_width;
        let top = page_height_pt - layout.box_y;
        let bottom = page_height_pt - layout.box_y - layout.box_height;

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
        page_height_pt: f32,
        border: BorderStyle,
    ) {
        let y = page_height_pt - layout.box_y - layout.box_height;

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
            georgia: Self::load_font_face(doc, "C:\\Windows\\Fonts\\georgia.ttf"),
            georgia_bold: Self::load_font_face(doc, "C:\\Windows\\Fonts\\georgiab.ttf"),
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
    fn get_font(attrs: &StyleAttributes, font_face: Option<&FontFace>) -> PdfFontHandle {
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
            "times" | "times new roman" | "serif" if is_bold => BuiltinFont::TimesBold,
            "times" | "times new roman" | "serif" => BuiltinFont::TimesRoman,
            "courier" | "courier new" | "monospace" if is_bold => BuiltinFont::CourierBold,
            "courier" | "courier new" | "monospace" => BuiltinFont::Courier,
            _ if is_bold => BuiltinFont::HelveticaBold,
            _ => BuiltinFont::Helvetica,
        };

        PdfFontHandle::Builtin(font)
    }

    fn get_font_face<'a>(attrs: &StyleAttributes, fonts: &'a FontRegistry) -> Option<&'a FontFace> {
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
                    || normalized.parse::<u16>().is_ok_and(|weight| weight >= 600)
            })
            .unwrap_or(false)
    }

    fn measure_text_width(text: &str, font_size: f32, parsed_font: Option<&ParsedFont>) -> f32 {
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

    fn parse_color(attrs: &StyleAttributes) -> Option<Color> {
        let value = attrs.style.get("color")?.trim().trim_matches('"');
        let color = match value.to_lowercase().as_str() {
            "black" => Self::rgb(0.0, 0.0, 0.0),
            "white" => Self::rgb(1.0, 1.0, 1.0),
            "red" => Self::rgb(1.0, 0.0, 0.0),
            "green" => Self::rgb(0.0, 0.5, 0.0),
            "blue" => Self::rgb(0.0, 0.0, 1.0),
            "gray" | "grey" => Self::rgb(0.5, 0.5, 0.5),
            _ => return Self::parse_hex_color(value),
        };

        Some(color)
    }

    fn parse_border(value: Option<&str>, attrs: &StyleAttributes) -> Option<BorderStyle> {
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

        Some(color)
    }

    fn parse_hex_color(value: &str) -> Option<Color> {
        let hex = value.strip_prefix('#')?;
        if hex.len() != 6 {
            return None;
        }

        let red = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let green = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let blue = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Self::rgb(red, green, blue))
    }

    fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color::Rgb(Rgb {
            r,
            g,
            b,
            icc_profile: None,
        })
    }

    fn parse_css_length(value: &str) -> Option<f32> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }

        let num_end = value
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(value.len());

        let num_str = &value[..num_end];
        let unit_str = &value[num_end..].trim().to_lowercase();
        let num: f32 = num_str.parse().ok()?;

        match unit_str.as_str() {
            "pt" => Some(num),
            "px" => Some(num * 0.75),
            "mm" => Some(num * 2.83465),
            "cm" => Some(num * 28.3465),
            "in" => Some(num * 72.0),
            "" => Some(num),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PdfRenderer;

    #[test]
    fn sanitize_builtin_text_replaces_unicode_punctuation() {
        assert_eq!(
            PdfRenderer::sanitize_builtin_text("September 2025 – Present · Toronto"),
            "September 2025 - Present | Toronto"
        );
    }
}
