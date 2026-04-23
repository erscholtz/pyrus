use crate::ast::Ast;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::HIRModule;
use crate::hir::hir_util::hir_error::HirError;

pub struct FuncPass;

impl HIRPass for FuncPass {
    fn run(&mut self, _hir: &mut HIRModule, _ast: &Ast) -> Result<(), Vec<HirError>> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "func_pass"
    }
}
