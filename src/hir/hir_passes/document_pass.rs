use crate::ast::Ast;
use crate::diagnostic::SemanticError;
use crate::hir::{HIRModule, hir_passes::HIRPass};

pub struct DocumentPass;

impl HIRPass for DocumentPass {
    fn run(&mut self, hir: &mut HIRModule, ast: &Ast) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();

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
