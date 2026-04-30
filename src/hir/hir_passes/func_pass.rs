use crate::ast::{Ast, FuncDeclStmt, KeyValue, ReturnStmt, StmtKind};
use crate::diagnostic::SemanticError;
use crate::hir::{
    FuncBlock, Op, Type,
    hir_passes::HIRPass,
    hir_types::{FuncDecl, FuncId, HIRModule, ValueId},
    hir_util::handle_args::parse_type,
    hir_util::handle_elem::lower_document_element,
    hir_util::handle_expr::assign_local,
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

                        let args = args
                            .iter()
                            .filter_map(|arg| parse_type(&arg.ty))
                            .collect::<Vec<Type>>();

                        let return_type = match return_type {
                            Some(ty) => parse_type(&ty),
                            None => None,
                        };

                        let body = match self.parse_body(body.as_slice()) {
                            Ok(body) => body,
                            Err(err) => {
                                errors.push(err);
                                continue;
                            }
                        };

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
    fn parse_body(&self, body: &[Spanned<StmtKind>]) -> Result<FuncBlock, SemanticError> {
        let mut ops = Vec::new();
        let mut returned_element_ref: Option<usize> = None;
        // for stmt in body {
        //     let loc = stmt.location.clone();
        //     match &stmt.node {
        //         StmtKind::DefaultSet(stmt) => {
        //             // NOTE default sets are only allowed at the top level
        //             return Err(SemanticError::DefaultSetAtInvalidLocation { location: loc });
        //         }
        //         StmtKind::ConstAssign(stmt) => {
        //             let id = ValueId(ops.len());
        //             let op = assign_local(stmt.name.clone(), &stmt.value, id, false);
        //             ops.push(op);
        //         }
        //         StmtKind::VarAssign(stmt) => {
        //             let id = ValueId(ops.len());
        //             let op = assign_local(stmt.name.clone(), &stmt.value, id, true);
        //             ops.push(op);
        //         }
        //         StmtKind::Return(ReturnStmt::DocElem(doc_element)) => {
        //             let element_id =
        //                 lower_document_element(doc_element, hirmodule, &mut body, None);
        //             ops.push(Op::Return {
        //                 doc_element_ref: element_id,
        //             });
        //             returned_element_ref = Some(element_id);
        //         }
        //         StmtKind::Return(ReturnStmt::Expr(expr)) => {
        //             let id = ValueId(ops.len());
        //             let op = assign_local("__return".to_string(), expr, id, false);
        //             ops.push(op);
        //         }
        //         _ => {}
        //     }
        // }
        Ok(FuncBlock {
            ops,
            returned_element_ref,
        })
    }
}
