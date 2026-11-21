use std::collections::HashMap;

use crate::ast::Ast;
use crate::diagnostic::SemanticError;
use crate::hir::hir_types::HirElemDecl;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::ElemId;
use crate::hir::hir_types::FuncDecl;
use crate::hir::hir_types::FuncId;
use crate::hir::hir_types::Global;
use crate::hir::hir_types::GlobalId;
use crate::hir::hir_types::HIRModule;
use crate::hir::hir_types::HirElementOp;

pub struct ValidationPass;

impl HIRPass for ValidationPass {
    fn run(
        &mut self,
        hir: &mut HIRModule,
        _ast: &Ast,
    ) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();

        errors.extend(self.validate_globals(&hir.globals));
        errors.extend(self.validate_functions(&hir.functions));
        errors.extend(self.validate_elements(&hir.element_decls));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn name(&self) -> &'static str {
        "ValidationPass"
    }
}

impl Default for ValidationPass {
    fn default() -> Self {
        Self {}
    }
}

impl ValidationPass {
    fn validate_globals(
        &self,
        globals: &HashMap<GlobalId, Global>,
    ) -> Vec<SemanticError> {
        let mut errors = Vec::new();

        errors
    }

    fn validate_functions(
        &self,
        functions: &HashMap<FuncId, FuncDecl>,
    ) -> Vec<SemanticError> {
        let mut errors = Vec::new();

        errors
    }

    fn validate_elements(
        &self,
        elements: &HashMap<ElemId, HirElemDecl>,
    ) -> Vec<SemanticError> {
        let mut errors = Vec::new();

        errors
    }
}
