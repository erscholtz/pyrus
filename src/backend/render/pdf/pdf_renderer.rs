use std::fs::{self, File};
use std::io::BufWriter;
use std::io::Write;

use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};

use crate::hlir::{FuncId, HLIRModule, HlirElement, Id, Op as HlirOp};

pub struct PdfRenderer;

impl PdfRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, hlir: HLIRModule) -> Result<(), std::io::Error> {
        let mut doc = PdfDocument::new("Document");

        let pages = self.setup_pages(hlir);
        let pdf_bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        fs::create_dir_all("generated")?;
        let file = File::create("generated/output.pdf")?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&pdf_bytes)?;

        Ok(())
    }

    fn setup_pages(&self, hlir: HLIRModule) -> Vec<PdfPage> {
        let mut pages = Vec::new();

        let ops = self.setup_ops(hlir);
        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
        pages.push(page);

        pages
    }

    fn setup_ops(&self, hlir: HLIRModule) -> Vec<Op> {
        // vec![
        //     Op::StartTextSection,
        //     Op::SetTextCursor {
        //         pos: Point::new(Mm(10.0), Mm(270.0)),
        //     },
        //     Op::SetFont {
        //         font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
        //         size: Pt(48.0),
        //     },
        //     Op::ShowText {
        //         items: vec![TextItem::Text("Hello, PDF!".to_string())],
        //     },
        //     Op::EndTextSection,
        // ]
        let mut pdf_ops = Vec::new();
        let mut point = Point::new(Mm(10.0), Mm(270.0));

        let document_id = FuncId(hlir.functions.len() - 1);
        let document = hlir
            .functions
            .get(&Id::Func(document_id))
            .expect("document function not found");
        for op in &document.body.ops {
            match op {
                HlirOp::HlirElementEmit { index } => {
                    let element = hlir.elements.get(*index).expect("element not found");
                    self.format_hlir_to_pdf_op(element.clone(), &hlir, &mut pdf_ops, &mut point);
                }
                HlirOp::Call { result, func, args } => {
                    let func = hlir.functions.get(&func).expect("func not found");
                    let returned_element_ref = func.body.returned_element_ref;
                    if let Some(ref returned_element_ref) = returned_element_ref {
                        let element = hlir
                            .elements
                            .get(*returned_element_ref)
                            .expect("element not found");
                        self.format_hlir_to_pdf_op(
                            element.clone(),
                            &hlir,
                            &mut pdf_ops,
                            &mut point,
                        );
                    }
                }
                _ => {}
            }
        }
        pdf_ops
    }

    fn format_hlir_to_pdf_op(
        &self,
        element: HlirElement,
        hlir: &HLIRModule,
        pdf_ops: &mut Vec<Op>,
        point: &mut Point,
    ) {
        match element {
            HlirElement::Text { content, .. } => {
                pdf_ops.push(Op::StartTextSection);
                pdf_ops.push(Op::SetTextCursor { pos: *point });
                pdf_ops.push(Op::SetFont {
                    font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
                    size: Pt(12.0),
                });
                pdf_ops.push(Op::ShowText {
                    items: vec![TextItem::Text(content.clone())],
                });
                pdf_ops.push(Op::EndTextSection);
                point.y -= Pt(12.0);
            }
            HlirElement::List { children, .. } => {
                for child_idx in children {
                    if let Some(child) = hlir.elements.get(child_idx) {
                        self.format_hlir_to_pdf_op(child.clone(), hlir, pdf_ops, point);
                    }
                }
            }
            HlirElement::Section { children, .. } => {
                for child_idx in children {
                    if let Some(child) = hlir.elements.get(child_idx) {
                        self.format_hlir_to_pdf_op(child.clone(), hlir, pdf_ops, point);
                    }
                }
            }
        }
    }
}
