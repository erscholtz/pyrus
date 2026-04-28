use crate::ast::{Ast, Expr, ExprKind, StmtKind};
use crate::diagnostic::SemanticError;
use crate::hir::HIRModule;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::{Global, GlobalId, Literal, Type, ValueId};

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
                        let Some(global) = self.assign_global(&name, &stmt.value, false) else {
                            continue;
                        };
                        hir.globals.insert(global_id, global);
                    }
                    StmtKind::ConstAssign(stmt) => {
                        let global_id = GlobalId(hir.globals.len());
                        let Some(global) = self.assign_global(&stmt.name, &stmt.value, false)
                        else {
                            continue;
                        };
                        hir.globals.insert(global_id, global);
                    }
                    StmtKind::VarAssign(stmt) => {
                        let global_id = GlobalId(hir.globals.len());
                        let Some(global) = self.assign_global(&stmt.name, &stmt.value, true) else {
                            continue;
                        };
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

impl GlobalPass {
    fn assign_global(&mut self, name: &String, value: &Expr, mutable: bool) -> Option<Global> {
        let global = match &value.node {
            ExprKind::StringLiteral(s) => Global {
                name: name.clone(),
                ty: Type::String,
                literal: Literal::String(s.clone()),
                mutable: mutable,
            },
            ExprKind::Int(n) => Global {
                name: name.clone(),
                ty: Type::Int,
                literal: Literal::Int(*n),
                mutable: mutable,
            },
            ExprKind::Float(n) => Global {
                name: name.clone(),
                ty: Type::Float,
                literal: Literal::Float(*n),
                mutable: mutable,
            },
            ExprKind::InterpolatedString(expr) => {
                // For globals with interpolated strings, we evaluate at initialization time
                // by converting to a string immediately (since globals are evaluated once)
                let result = self.eval_interpolated_string_to_literal(&expr.parts)?;
                Global {
                    name: name.clone(),
                    ty: Type::String,
                    literal: result,
                    mutable: mutable,
                }
            }
            _ => {
                // TODO: todo!("implement other expression types")
                return None;
            }
        };
        Some(global)
    }

    fn eval_interpolated_string_to_literal(&self, parts: &[ExprKind]) -> Option<Literal> {
        let mut result = String::new();
        for part in parts {
            match part {
                ExprKind::StringLiteral(s) => result.push_str(s),
                ExprKind::Int(n) => result.push_str(&n.to_string()),
                ExprKind::Float(f) => result.push_str(&f.to_string()),
                ExprKind::Identifier(s) => result.push_str(s),
                ExprKind::StructDefault(s) => result.push_str(&format!("default({})", s.name)),
                _ => {
                    return None;
                }
            }
        }
        Some(Literal::String(result))
    }
}
