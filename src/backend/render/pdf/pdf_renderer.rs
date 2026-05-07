use std::fs::{self, File};
use std::io::BufWriter;
use std::io::Write;

use printpdf::{
    BuiltinFont, Color, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt,
    Rgb, TextItem,
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
            HirElementOp::Text { attributes, .. } => hlir
                .attributes
                .find_node(*attributes)
                .map(|n| &n.computed)
                .unwrap_or(&default_attrs),
            _ => &default_attrs,
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
                let lines = LayoutEngine::wrap_text(&content, layout.width, font_size);

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
        }
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
