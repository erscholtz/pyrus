use crate::ast::{Ast, FuncDeclStmt, StmtKind};
use crate::diagnostic::SemanticError;
use crate::hir::{
    hir_passes::HIRPass,
    hir_types::{FuncDecl, FuncId, HIRModule, Type},
    hir_util::{handle_args::parse_type, handle_block::lower_block},
};
use crate::util::Spanned;
pub struct FuncPass;

impl HIRPass for FuncPass {
    fn run(&mut self, hir: &mut HIRModule, ast: &Ast) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();
        if let Some(template) = &ast.template {
            for stmt in &template.statements {
                match stmt {
                    Spanned {
                        node:
                            StmtKind::FuncDecl(FuncDeclStmt {
                                name,
                                args,
                                body,
                                return_type,
                            }),
                        ..
                    } => {
                        let func_id = FuncId(hir.functions.len());
                        let arg_names = args
                            .iter()
                            .map(|arg| arg.value.to_string())
                            .collect::<Vec<String>>();
                        let args = args
                            .iter()
                            .filter_map(|arg| parse_type(&arg.ty))
                            .collect::<Vec<Type>>();
                        let return_type = match return_type {
                            Some(ty) => parse_type(&ty),
                            None => None,
                        };
                        let lowered = match lower_block(body, hir) {
                            Ok(lowered) => lowered,
                            Err(err) => {
                                errors.extend(err);
                                continue;
                            }
                        };

                        hir.functions.insert(
                            func_id,
                            FuncDecl {
                                name: name.clone(),
                                arg_names,
                                args,
                                return_type,
                                return_summary: lowered.return_summary,
                                body: lowered.block,
                            },
                        );
                    }
                    _ => {
                        continue;
                    }
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
        "func_pass"
    }
}

impl Default for FuncPass {
    fn default() -> Self {
        Self {}
    }
}
