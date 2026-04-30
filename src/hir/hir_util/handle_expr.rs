use crate::ast::{BinaryExpr, Expr, ExprKind, InterpolatedStringExpr};
use crate::hir::Op;
use crate::hir::ValueId;
use crate::hir::{Global, Literal, Type};

pub fn assign_global(name: &str, value: &Expr, mutable: bool) -> Global {
    let (literal, ty) = expr_to_literal(value);
    Global {
        name: name.to_string(),
        ty,
        literal,
        mutable,
    }
}

pub fn assign_local(name: String, value: &Expr, id: ValueId, mutable: bool) -> Op {
    let (literal, ty) = expr_to_literal(value);
    if mutable {
        Op::Var {
            result: id,
            name,
            literal,
            ty,
        }
    } else {
        Op::Const {
            result: id,
            name,
            literal,
            ty,
        }
    }
}

fn expr_to_literal(expr: &Expr) -> (Literal, Type) {
    match &expr.node {
        ExprKind::StringLiteral(value) => (Literal::String(value.clone()), Type::String),
        ExprKind::Int(value) => (Literal::Int(*value), Type::Int),
        ExprKind::Float(value) => (Literal::Float(*value), Type::Float),
        ExprKind::Identifier(value) => (Literal::String(value.clone()), Type::String),
        ExprKind::InterpolatedString(InterpolatedStringExpr { parts }) => {
            let value = eval_interpolated_string(parts);
            (Literal::String(value), Type::String)
        }
        ExprKind::StructDefault(value) => (
            Literal::String(format!("default({})", value.name)),
            Type::String,
        ),
        ExprKind::Binary(BinaryExpr { .. }) => (Literal::Int(0), Type::Int),
        _ => (Literal::Int(0), Type::Int),
    }
}

fn eval_interpolated_string(parts: &[ExprKind]) -> String {
    let mut result = String::new();
    for part in parts {
        match part {
            ExprKind::StringLiteral(s) => result.push_str(s),
            ExprKind::Int(n) => result.push_str(&n.to_string()),
            ExprKind::Float(f) => result.push_str(&f.to_string()),
            ExprKind::Identifier(s) => result.push_str(s),
            ExprKind::StructDefault(s) => {
                result.push_str(&format!("default({})", s.name));
            }
            _ => {} // TODO other types
        }
    }
    result
}
