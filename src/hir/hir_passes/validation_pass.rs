use std::collections::HashMap;

use crate::ast::Ast;
use crate::diagnostic::SemanticError;
use crate::hir::{
    hir_passes::HIRPass,
    hir_types::{FuncDecl, FuncId, Global, GlobalId, HIRModule},
};

pub struct ValidationPass;

impl HIRPass for ValidationPass {
    fn run(&mut self, hir: &mut HIRModule, _ast: &Ast) -> Result<(), Vec<SemanticError>> {
        self.validate_globals(&hir.globals);
        self.validate_functions(&hir.functions);

        Ok(())
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
    fn validate_globals(&self, globals: &HashMap<GlobalId, Global>) {}

    fn validate_functions(&self, functions: &HashMap<FuncId, FuncDecl>) {}
}
