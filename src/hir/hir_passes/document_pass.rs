use crate::ast::Ast;
use crate::diagnostic::SemanticError;
use crate::hir::{
    HIRModule,
    hir_passes::HIRPass,
    hir_types::{FuncBlock, FuncDecl, FuncId, Op},
    hir_util::handle_elem::lower_document_element,
};

pub struct DocumentPass;

impl HIRPass for DocumentPass {
    fn run(&mut self, hir: &mut HIRModule, ast: &Ast) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();
        let mut document_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        };
        if let Some(document) = &ast.document {
            for element in &document.elements {
                match lower_document_element(element, hir, &mut document_body, None) {
                    Ok(index) => {
                        document_body.ops.push(Op::HirElementEmit { index });
                    }
                    Err(err) => errors.extend(err),
                }
            }
        }

        hir.functions.insert(
            FuncId(hir.functions.len()),
            FuncDecl {
                name: "__document".to_string(),
                args: Vec::new(),
                return_type: None,
                body: document_body,
            },
        );

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn name(&self) -> &'static str {
        "document"
    }
}

impl Default for DocumentPass {
    fn default() -> Self {
        Self {}
    }
}
