use crate::hir::HIRPass;

use crate::hir::ir_types::{Global, GlobalId, Id, Literal, Op, Type, ValueId};

impl HIRPass {
    pub fn assign_global(
        &mut self,
        name: &String,
        value: &crate::ast::Expression,
        id: Id,
        mutable: bool,
    ) -> Global {
        let global = match &value.node {
            crate::ast::ExpressionKind::StringLiteral(s) => Global {
                id: id,
                name: name.clone(),
                ty: Type::String,
                init: Literal::String(s.clone()),
                mutable: mutable,
            },
            crate::ast::ExpressionKind::Int(n) => Global {
                id: id,
                name: name.clone(),
                ty: Type::Int,
                init: Literal::Int(*n),
                mutable: mutable,
            },
            crate::ast::ExpressionKind::Float(n) => Global {
                id: id,
                name: name.clone(),
                ty: Type::Float,
                init: Literal::Float(*n),
                mutable: mutable,
            },
            crate::ast::ExpressionKind::InterpolatedString(parts) => {
                // For globals with interpolated strings, we evaluate at initialization time
                // by converting to a string immediately (since globals are evaluated once)
                let result = self.eval_interpolated_string_to_literal(&parts);
                Global {
                    id,
                    name: name.clone(),
                    ty: Type::String,
                    init: result,
                    mutable: mutable,
                }
            }
            _ => {
                todo!("implement other expression types")
            }
        };
        global
    }

    pub fn assign_local(
        &mut self,
        name: String,
        value: crate::ast::Expression,
        id: Id,
        _mutable: bool,
    ) -> Op {
        let op = match &value.node {
            crate::ast::ExpressionKind::StringLiteral(s) => Op::Const {
                result: id,
                literal: Literal::String(s.clone()),
                ty: Type::String,
            },
            crate::ast::ExpressionKind::Int(n) => Op::Const {
                result: id,
                literal: Literal::Int(*n),
                ty: Type::Int,
            },
            crate::ast::ExpressionKind::Float(n) => Op::Const {
                result: id,
                literal: Literal::Float(*n),
                ty: Type::Float,
            },
            crate::ast::ExpressionKind::InterpolatedString(parts) => {
                // For simplicity in local assignment, we convert to a literal string
                // In a full implementation, this would generate ops to build the string at runtime
                let result = self.eval_interpolated_string_to_literal(&parts);
                Op::Const {
                    result: id,
                    literal: result,
                    ty: Type::String,
                }
            }
            _ => {
                todo!("implement other expression types")
            }
        };

        // add variable to symbol table
        let len = self.symbol_table.len();
        let scope = self.symbol_table.get_mut(len - 1).unwrap(); // most recent scope
        scope.insert(name.clone(), id); // add to known symbols

        op
    }

    fn eval_interpolated_string_to_literal(&self, parts: &[crate::ast::InterpPart]) -> Literal {
        let mut result = String::new();
        for part in parts {
            match part {
                crate::ast::InterpPart::Text(text) => result.push_str(text),
                crate::ast::InterpPart::Expression(expr_kind) => {
                    // Try to evaluate the expression to a constant
                    match expr_kind {
                        crate::ast::ExpressionKind::StringLiteral(s) => result.push_str(s),
                        crate::ast::ExpressionKind::Int(n) => result.push_str(&n.to_string()),
                        crate::ast::ExpressionKind::Float(f) => result.push_str(&f.to_string()),
                        crate::ast::ExpressionKind::Identifier(name) => {
                            // For identifiers, we can't resolve at compile time without
                            // more sophisticated constant propagation, so we keep the placeholder
                            result.push_str(&format!("{{{}}}", name));
                        }
                        _ => {
                            // For other expressions, we use a placeholder
                            result.push_str(&format!("{{{}}}", expr_kind.to_string()));
                        }
                    }
                }
            }
        }
        Literal::String(result)
    }
}
