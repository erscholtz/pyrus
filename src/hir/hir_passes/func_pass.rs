use crate::ast::{Ast, FuncDeclStmt, StmtKind};
use crate::diagnostic::SemanticError;
use crate::hir::{
    FuncBlock,
    hir_passes::HIRPass,
    hir_types::{FuncDecl, FuncId, HIRModule},
    hir_util::handle_args::parse_type,
};
use crate::util::Spanned;
pub struct FuncPass;

impl HIRPass for FuncPass {
    fn run(&mut self, hir: &mut HIRModule, ast: &Ast) -> Result<(), Vec<SemanticError>> {
        if let Some(template) = &ast.template {
            for stmt in &template.statements {
                match stmt {
                    Spanned {
                        node:
                            StmtKind::FuncDecl(FuncDeclStmt {
                                name,
                                args,
                                body: _,
                                return_type,
                            }),
                        ..
                    } => {
                        let func_id = FuncId(hir.functions.len());

                        let args = args
                            .iter()
                            .filter_map(|arg| parse_type(&arg.ty))
                            .collect::<Vec<_>>();

                        let return_type = match return_type {
                            Some(ty) => parse_type(&ty),
                            None => None,
                        };

                        let body = self.parse_body();

                        hir.functions.insert(
                            func_id,
                            FuncDecl {
                                name: name.clone(),
                                args,
                                return_type,
                                body,
                            },
                        );
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "func_pass"
    }
}

impl Default for FuncPass {
    fn default() -> Self {
        Self {}
    }
}

impl FuncPass {
    fn parse_body(&self) -> FuncBlock {
        FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        }
    }
}
