use crate::ast::{IfStmt, ReturnStmt, Stmt, StmtKind};
use crate::diagnostic::SemanticError;
use crate::hir::{
    hir_types::{Block, HIRModule, LoweredBlock, Op, ReturnSummary, ValueId},
    hir_util::{handle_elem::lower_document_element, handle_expr::assign_local},
};

pub fn lower_block(body: &[Stmt], hir: &mut HIRModule) -> Result<LoweredBlock, Vec<SemanticError>> {
    let mut ir_body = LoweredBlock {
        block: Block { items: Vec::new() },
        return_summary: ReturnSummary::None,
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
                let id = ValueId(ir_body.block.items.len());
                let op = assign_local(stmt.name.clone(), &stmt.value, id, false);
                ir_body.block.items.push(op);
            }
            StmtKind::VarAssign(stmt) => {
                let id = ValueId(ir_body.block.items.len());
                let op = assign_local(stmt.name.clone(), &stmt.value, id, true);
                ir_body.block.items.push(op);
            }
            StmtKind::Return(ReturnStmt::DocElem(doc_element)) => {
                let element_id =
                    match lower_document_element(doc_element, hir, &mut ir_body.block, None) {
                        Ok(id) => id,
                        Err(err) => {
                            errors.extend(err);
                            continue;
                        }
                    };
                ir_body.block.items.push(Op::Return {
                    doc_element_ref: element_id,
                });
                ir_body.return_summary = ir_body
                    .return_summary
                    .combine(ReturnSummary::SingleElem(element_id));
            }
            StmtKind::Return(ReturnStmt::Expr(expr)) => {
                let id = ValueId(ir_body.block.items.len());
                let op = assign_local("__return".to_string(), expr, id, false);
                ir_body.block.items.push(op);
                ir_body.return_summary = ir_body.return_summary.combine(ReturnSummary::Expr);
            }
            StmtKind::If(IfStmt {
                condition,
                body,
                else_body,
            }) => {
                let cond_id = ValueId(ir_body.block.items.len());
                let op = assign_local("__cond".to_string(), condition, cond_id, false);
                ir_body.block.items.push(op);
                let then_block = match lower_block(body, hir) {
                    Ok(ops) => ops,
                    Err(err) => {
                        errors.extend(err);
                        LoweredBlock {
                            block: Block { items: Vec::new() },
                            return_summary: ReturnSummary::None,
                        }
                    }
                };
                let mut else_block = None;
                if let Some(else_body) = else_body {
                    else_block = match lower_block(else_body, hir) {
                        Ok(ops) => Some(ops),
                        Err(err) => {
                            errors.extend(err);
                            None
                        }
                    };
                }

                let if_summary = ReturnSummary::from_if(
                    then_block.return_summary.clone(),
                    else_block
                        .as_ref()
                        .map(|lowered| lowered.return_summary.clone()),
                );
                ir_body.return_summary = ir_body.return_summary.combine(if_summary);

                ir_body.block.items.push(Op::If {
                    cond: cond_id,
                    then: then_block.block,
                    else_: else_block.map(|lowered| lowered.block),
                });
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
