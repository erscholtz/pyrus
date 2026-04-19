use crate::ast::{Ast, Expression, ExpressionKind, StatementKind};
use crate::hir::HIRModule;
use crate::hir::hir_passes::HIRPass;
use crate::hir::hir_types::{Global, GlobalId, Id, Literal, Type};
use crate::hir::hir_util::hir_error::HirError;

pub struct GlobalPass;

impl HIRPass for GlobalPass {
    fn run(&mut self, hir: &mut HIRModule, ast: &Ast) -> Result<(), Vec<HirError>> {
        let errors = Vec::new();
        if let Some(template) = ast.template.clone() {
            for statement in template.statements {
                match statement.node {
                    StatementKind::DefaultSet { key, value } => {
                        let global_id = Id::Global(GlobalId(hir.globals.len()));
                        let name = "__".to_string() + &key.clone();
                        let Some(global) = self.assign_global(&name, &value, global_id, false)
                        else {
                            continue;
                        };
                        hir.globals.insert(global_id, global);
                    }
                    StatementKind::ConstAssign { name, value } => {
                        let global_id = Id::Global(GlobalId(hir.globals.len()));
                        let Some(global) = self.assign_global(&name, &value, global_id, false)
                        else {
                            continue;
                        };
                        hir.globals.insert(global_id, global);
                    }
                    StatementKind::VarAssign { name, value } => {
                        let global_id = Id::Global(GlobalId(hir.globals.len()));
                        let Some(global) = self.assign_global(&name, &value, global_id, true)
                        else {
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
        "var_pass"
    }
}

impl Default for GlobalPass {
    fn default() -> Self {
        Self {}
    }
}

impl GlobalPass {
    fn assign_global(
        &mut self,
        name: &String,
        value: &Expression,
        id: Id,
        mutable: bool,
    ) -> Option<Global> {
        let global = match &value.node {
            ExpressionKind::StringLiteral(s) => Global {
                id: id,
                name: name.clone(),
                ty: Type::String,
                init: Literal::String(s.clone()),
                mutable: mutable,
            },
            ExpressionKind::Int(n) => Global {
                id: id,
                name: name.clone(),
                ty: Type::Int,
                init: Literal::Int(*n),
                mutable: mutable,
            },
            ExpressionKind::Float(n) => Global {
                id: id,
                name: name.clone(),
                ty: Type::Float,
                init: Literal::Float(*n),
                mutable: mutable,
            },
            ExpressionKind::InterpolatedString { parts } => {
                // For globals with interpolated strings, we evaluate at initialization time
                // by converting to a string immediately (since globals are evaluated once)
                let result = self.eval_interpolated_string_to_literal(parts.clone())?; // option returns None if evaluation fails
                Global {
                    id,
                    name: name.clone(),
                    ty: Type::String,
                    init: result,
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

    fn eval_interpolated_string_to_literal(&self, parts: Vec<ExpressionKind>) -> Option<Literal> {
        let mut result = String::new();
        for part in parts {
            match part {
                ExpressionKind::StringLiteral(s) => result.push_str(&s),
                ExpressionKind::Int(n) => result.push_str(&n.to_string()),
                ExpressionKind::Float(f) => result.push_str(&f.to_string()),
                ExpressionKind::Identifier(s) => result.push_str(&s),
                ExpressionKind::StructDefault(s) => result.push_str(&format!("default({})", s)),
                _ => {
                    return None;
                }
            }
        }
        Some(Literal::String(result))
    }
}
