use crate::ast::{Expression, ExpressionKind};
use crate::hir::HIRPass;
use crate::hir::ir_types::{Global, GlobalId, Id, Literal, Op, Type, ValueId};

impl HIRPass {
    pub fn assign_global(
        &mut self,
        name: &String,
        value: &Expression,
        id: Id,
        mutable: bool,
    ) -> Global {
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
                let result = self.eval_interpolated_string_to_literal(parts.clone());
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

    pub fn assign_local(&mut self, name: String, value: Expression, id: Id, _mutable: bool) -> Op {
        let op = match &value.node {
            ExpressionKind::StringLiteral(s) => Op::Const {
                result: id,
                literal: Literal::String(s.clone()),
                ty: Type::String,
            },
            ExpressionKind::Int(n) => Op::Const {
                result: id,
                literal: Literal::Int(*n),
                ty: Type::Int,
            },
            ExpressionKind::Float(n) => Op::Const {
                result: id,
                literal: Literal::Float(*n),
                ty: Type::Float,
            },
            ExpressionKind::InterpolatedString { parts } => {
                // For simplicity in local assignment, we convert to a literal string
                // In a full implementation, this would generate ops to build the string at runtime
                let result = self.eval_interpolated_string_to_literal(parts.clone());
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

    fn eval_interpolated_string_to_literal(&self, parts: Vec<ExpressionKind>) -> Literal {
        let mut result = String::new();
        for part in parts {
            match part {
                ExpressionKind::StringLiteral(s) => result.push_str(&s),
                ExpressionKind::Int(n) => result.push_str(&n.to_string()),
                ExpressionKind::Float(f) => result.push_str(&f.to_string()),
                ExpressionKind::Identifier(s) => result.push_str(&s),
                ExpressionKind::StructDefault(s) => result.push_str(&format!("default({})", s)),
                _ => {}
            }
        }
        Literal::String(result)
    }
}
