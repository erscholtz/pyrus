use crate::ast::{Ast, FuncDeclStmt, ReturnStmt, StmtKind};
use crate::diagnostic::SemanticError;
use crate::hir::{
    hir_passes::HIRPass,
    hir_types::{FuncBlock, FuncDecl, FuncId, HIRModule, Op, Type, ValueId},
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

                        let body = match self.parse_body(body.as_slice(), hir) {
                            Ok(body) => body,
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

impl FuncPass {
    fn parse_body(
        &self,
        body: &[Spanned<StmtKind>],
        hir: &mut HIRModule,
    ) -> Result<FuncBlock, Vec<SemanticError>> {
        let mut ir_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        };
        let mut errors = Vec::new();
        for stmt in body {
            let loc = stmt.location.clone();
            match &stmt.node {
                StmtKind::DefaultSet(_) => {
                    // NOTE default sets are only allowed at the top level
                    errors.push(SemanticError::DefaultSetAtInvalidLocation { location: loc });
                }
                StmtKind::ConstAssign(stmt) => {
                    let id = ValueId(ir_body.ops.len());
                    let op = assign_local(stmt.name.clone(), &stmt.value, id, false);
                    ir_body.ops.push(op);
                }
                StmtKind::VarAssign(stmt) => {
                    let id = ValueId(ir_body.ops.len());
                    let op = assign_local(stmt.name.clone(), &stmt.value, id, true);
                    ir_body.ops.push(op);
                }
                StmtKind::Return(ReturnStmt::DocElem(doc_element)) => {
                    let element_id =
                        match lower_document_element(doc_element, hir, &mut ir_body, None) {
                            Ok(id) => id,
                            Err(err) => {
                                errors.extend(err);
                                continue;
                            }
                        };
                    ir_body.ops.push(Op::Return {
                        doc_element_ref: element_id,
                    });
                    ir_body.returned_element_ref = Some(element_id);
                }
                StmtKind::Return(ReturnStmt::Expr(expr)) => {
                    let id = ValueId(ir_body.ops.len());
                    let op = assign_local("__return".to_string(), expr, id, false);
                    ir_body.ops.push(op);
                }
                _ => {}
            }
        }

        if errors.is_empty() {
            return Ok(ir_body);
        } else {
            return Err(errors);
        }
    }
}
