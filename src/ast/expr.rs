use std::fmt;

use crate::util::Spanned;

pub type Expr = Spanned<ExprKind>;

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    StringLiteral(String),
    InterpolatedString {
        parts: Vec<ExprKind>,
    },
    Int(i64),
    Float(f64),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        operator: BinOp,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOp,
        expr: Box<Expr>,
    },
    StructDefault(String),
}

impl ExprKind {
    // TODO this is not good find a way to remove
    pub fn as_number(&self) -> Option<i64> {
        match self {
            ExprKind::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ExprKind::StringLiteral(s) => s.clone(),
            ExprKind::InterpolatedString { parts } => {
                let mut result = String::new();
                for part in parts {
                    result.push_str(&part.to_string());
                }
                result
            }
            ExprKind::Unary { operator, expr } => match operator {
                UnaryOp::Negate => format!("-{}", expr.to_string()),
                UnaryOp::Not => format!("!{}", expr.to_string()),
            },
            ExprKind::StructDefault(name) => format!("default({})", name),
            ExprKind::Int(value) => format!("{}", value),
            ExprKind::Float(value) => format!("{}", value),
            _ => "Error".to_string(),
        }
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    Mod,
}
