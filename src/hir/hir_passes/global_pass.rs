use crate::ast::{Ast, StmtKind};
use crate::diagnostic::SemanticError;
use crate::hir::{
    HIRModule, hir_passes::HIRPass, hir_types::GlobalId, hir_util::handle_expr::assign_global,
};

pub struct GlobalPass;

impl HIRPass for GlobalPass {
    fn run(&mut self, hir: &mut HIRModule, ast: &Ast) -> Result<(), Vec<SemanticError>> {
        let errors = Vec::new();
        if let Some(template) = &ast.template {
            for statement in &template.statements {
                match &statement.node {
                    StmtKind::DefaultSet(stmt) => {
                        let global_id = GlobalId(hir.globals.len());
                        let name = format!("__{}", stmt.key);
                        let global = assign_global(&name, &stmt.value, false);
                        hir.globals.insert(global_id, global);
                    }
                    StmtKind::ConstAssign(stmt) => {
                        let global_id = GlobalId(hir.globals.len());
                        let global = assign_global(&stmt.name, &stmt.value, false);
                        hir.globals.insert(global_id, global);
                    }
                    StmtKind::VarAssign(stmt) => {
                        let global_id = GlobalId(hir.globals.len());
                        let global = assign_global(&stmt.name, &stmt.value, true);
                        hir.globals.insert(global_id, global);
                    }
                    _ => {} // TODO: add in default case
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn name(&self) -> &'static str {
        "global_pass"
    }
}

impl Default for GlobalPass {
    fn default() -> Self {
        Self {}
    }
}
