use std::fs::{self, File};
use std::io::BufWriter;
use std::io::Write;

use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};

use crate::hir::{HIRModule, HirElement, StyleAttributes};
use crate::layout::ComputedLayout;

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
        element: HirElement,
        hlir: &HIRModule,
        pdf_ops: &mut Vec<Op>,
        layout: &ComputedLayout,
    ) {
        // Convert layout coordinates (points) to PDF coordinates (Mm)
        // Note: PDF y-coordinate starts from bottom, layout y starts from top
        let page_height_pt = 842.0; // A4 height in points

        // Get font size first so we can adjust for baseline
        let (font_size, font) = match &element {
            HirElement::Text { attributes, .. } => {
                let default_attrs = StyleAttributes::default();
                let attrs = hlir
                    .attributes
                    .find_node(*attributes)
                    .map(|n| &n.computed)
                    .unwrap_or(&default_attrs);
                (
                    Self::parse_font_size(attrs),
                    Self::get_font_from_family(attrs),
                )
            }
            _ => (12.0, PdfFontHandle::Builtin(BuiltinFont::Helvetica)),
        };

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
            HirElement::Text { content, .. } => {
                pdf_ops.push(Op::StartTextSection);
                pdf_ops.push(Op::SetTextCursor { pos: point });
                pdf_ops.push(Op::SetFont {
                    font,
                    size: Pt(font_size),
                });
                pdf_ops.push(Op::ShowText {
                    items: vec![TextItem::Text(content.clone())],
                });
                pdf_ops.push(Op::EndTextSection);
            }
            HirElement::List { .. } => {
                // Container elements don't render directly
            }
            HirElement::Section { .. } => {
                // Container elements don't render directly
            }
        }
    }

    /// Parse font-size from computed styles, defaulting to 12pt
    fn parse_font_size(attrs: &StyleAttributes) -> f32 {
        attrs
            .style
            .get("font-size")
            .and_then(|v| {
                // Parse value like "24pt" or "12px"
                let v = v.trim();
                let num_end = v
                    .find(|c: char| !c.is_ascii_digit() && c != '.')
                    .unwrap_or(v.len());
                v[..num_end].parse::<f32>().ok()
            })
            .unwrap_or(12.0)
    }

    /// Get PDF font from font-family CSS property
    fn get_font_from_family(attrs: &StyleAttributes) -> PdfFontHandle {
        let font_family = attrs
            .style
            .get("font-family")
            .map(|v| v.as_str())
            .unwrap_or("Helvetica");

        match font_family.to_lowercase().as_str() {
            "times" | "times new roman" | "serif" => {
                PdfFontHandle::Builtin(BuiltinFont::TimesRoman)
            }
            "courier" | "courier new" | "monospace" => PdfFontHandle::Builtin(BuiltinFont::Courier),
            "helvetica" | "sans-serif" | _ => PdfFontHandle::Builtin(BuiltinFont::Helvetica),
        }
    }
}
