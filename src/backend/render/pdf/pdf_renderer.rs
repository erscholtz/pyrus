use std::fs::{self, File};
use std::io::BufWriter;
use std::io::Write;

use printpdf::{
    Actions, BuiltinFont, Color, ColorArray, Line, LinePoint, LinkAnnotation, Mm, Op, PdfDocument,
    PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, Rect, Rgb, TextItem,
};

use crate::hir::hir_types::{HIRModule, HirElementOp, StyleAttributes};
use crate::layout::{ComputedLayout, LayoutEngine};

pub struct PdfRenderer;

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

        let pages = self.setup_pages(hlir, computed_layouts);
        let pdf_bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        fs::create_dir_all("generated")?;
        let file = File::create("generated/output.pdf")?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&pdf_bytes)?;

        Ok(())
    }

    fn setup_pages(&self, hlir: HIRModule, computed_layouts: &[ComputedLayout]) -> Vec<PdfPage> {
        let mut pages = Vec::new();

        let ops = self.setup_ops(hlir, computed_layouts);
        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
        pages.push(page);

        pages
    }

    fn setup_ops(&self, hlir: HIRModule, computed_layouts: &[ComputedLayout]) -> Vec<Op> {
        let mut pdf_ops = Vec::new();

        // Render each element with its computed layout
        for layout in computed_layouts {
            if let Some(element) = hlir.elements.get(layout.element_index) {
                self.format_hlir_to_pdf_op(element.clone(), &hlir, &mut pdf_ops, layout);
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
        let font = Self::get_font(attrs);
        let fill_color = Self::parse_color(attrs);

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

        match element {
            HirElementOp::Text { content, .. } => {
                self.push_text_ops(
                    pdf_ops,
                    &content,
                    layout,
                    point,
                    font,
                    font_size,
                    line_height,
                    fill_color,
                    y_mm,
                );
                self.push_autolink_annotations(pdf_ops, &content, layout, font_size, line_height);
            }
            HirElementOp::Link { href, content, .. } => {
                self.push_text_ops(
                    pdf_ops,
                    &content,
                    layout,
                    point,
                    font,
                    font_size,
                    line_height,
                    fill_color,
                    y_mm,
                );
                pdf_ops.push(Op::LinkAnnotation {
                    link: LinkAnnotation::new(
                        Rect::from_xywh(
                            Pt(layout.x),
                            Pt(page_height_pt - layout.y - layout.height),
                            Pt(layout.width),
                            Pt(layout.height),
                        ),
                        Actions::uri(Self::normalize_url(&href).unwrap_or(href)),
                        None,
                        Some(ColorArray::Rgb([0.0, 0.0, 1.0])),
                        None,
                    ),
                });
            }
            HirElementOp::List { .. } => {
                // Container elements don't render directly
            }
            HirElementOp::Section { .. } => {
                // Container elements don't render directly
            }
            HirElementOp::Image { .. } => {
                // Render image
            }
            HirElementOp::Table { .. } => {
                // Render table
            }
            HirElementOp::Separator { .. } => {
                let line_y_pt = page_height_pt - layout.y - (layout.height / 2.0);
                let color = fill_color.unwrap_or_else(|| Self::rgb(0.5, 0.5, 0.5));

                pdf_ops.push(Op::SetOutlineColor { col: color });
                pdf_ops.push(Op::SetOutlineThickness {
                    pt: Pt(layout.height.max(1.0)),
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
        point: Point,
        font: PdfFontHandle,
        font_size: f32,
        line_height: f32,
        fill_color: Option<Color>,
        y_mm: f32,
    ) {
        let lines = LayoutEngine::wrap_text(content, layout.width, font_size);

        if let Some(marker) = &layout.marker {
            let marker_x_mm = (layout.x - 14.0).max(0.0) / 2.83465;
            let marker_point = Point::new(Mm(marker_x_mm), Mm(y_mm));

            pdf_ops.push(Op::StartTextSection);
            pdf_ops.push(Op::SetTextCursor { pos: marker_point });
            pdf_ops.push(Op::SetFont {
                font: font.clone(),
                size: Pt(font_size),
            });
            pdf_ops.push(Op::SetLineHeight {
                lh: Pt(line_height),
            });
            if let Some(col) = fill_color.clone() {
                pdf_ops.push(Op::SetFillColor { col });
            }
            pdf_ops.push(Op::ShowText {
                items: vec![TextItem::Text(marker.clone())],
            });
            pdf_ops.push(Op::EndTextSection);
        }

        pdf_ops.push(Op::StartTextSection);
        pdf_ops.push(Op::SetTextCursor { pos: point });
        pdf_ops.push(Op::SetFont {
            font,
            size: Pt(font_size),
        });
        pdf_ops.push(Op::SetLineHeight {
            lh: Pt(line_height),
        });
        if let Some(col) = fill_color {
            pdf_ops.push(Op::SetFillColor { col });
        }

        for (line_idx, line) in lines.iter().enumerate() {
            if line_idx > 0 {
                pdf_ops.push(Op::AddLineBreak);
            }
            pdf_ops.push(Op::ShowText {
                items: vec![TextItem::Text(line.clone())],
            });
        }
        pdf_ops.push(Op::EndTextSection);
    }

    fn push_autolink_annotations(
        &self,
        pdf_ops: &mut Vec<Op>,
        content: &str,
        layout: &ComputedLayout,
        font_size: f32,
        line_height: f32,
    ) {
        let lines = LayoutEngine::wrap_text(content, layout.width, font_size);

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
                        Some(ColorArray::Rgb([0.0, 0.0, 1.0])),
                        None,
                    ),
                });
            }
        }
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

    /// Get PDF font from font-family and font-weight CSS properties.
    fn get_font(attrs: &StyleAttributes) -> PdfFontHandle {
        let font_family = attrs
            .style
            .get("font-family")
            .map(|v| v.as_str())
            .unwrap_or("Helvetica");
        let is_bold = attrs
            .style
            .get("font-weight")
            .map(|value| {
                let normalized = value.trim().trim_matches('"').to_lowercase();
                normalized == "bold"
                    || normalized == "bolder"
                    || normalized.parse::<u16>().is_ok_and(|weight| weight >= 600)
            })
            .unwrap_or(false);

        let family = font_family.trim().trim_matches('"').to_lowercase();
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
