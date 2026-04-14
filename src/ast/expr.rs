use std::fmt;

use crate::util::Spanned;

pub type Expr = Spanned<ExprKind>;

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
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

/// Thin wrapper types for expression kinds.
/// Each expression type can be parsed independently, then converted to ExprKind.
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<ExprKind>,
    pub op: BinOp,
    pub right: Box<ExprKind>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<ExprKind>,
}

#[derive(Debug, Clone)]
pub struct InterpolatedStringExpr {
    pub parts: Vec<ExprKind>,
}

#[derive(Debug, Clone)]
pub struct StructDefaultExpr {
    pub name: String,
}

/// The kind of expression - used within Spanned<ExprKind>
#[derive(Debug, Clone)]
pub enum ExprKind {
    StringLiteral(String),
    InterpolatedString(InterpolatedStringExpr),
    Int(i64),
    Float(f64),
    Identifier(String),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    StructDefault(StructDefaultExpr),
}

// Conversions from wrapper types to ExprKind
impl From<BinaryExpr> for ExprKind {
    fn from(e: BinaryExpr) -> Self {
        ExprKind::Binary(e)
    }
}

impl From<UnaryExpr> for ExprKind {
    fn from(e: UnaryExpr) -> Self {
        ExprKind::Unary(e)
    }
}

impl From<InterpolatedStringExpr> for ExprKind {
    fn from(e: InterpolatedStringExpr) -> Self {
        ExprKind::InterpolatedString(e)
    }
}

impl From<StructDefaultExpr> for ExprKind {
    fn from(e: StructDefaultExpr) -> Self {
        ExprKind::StructDefault(e)
    }
}

impl ExprKind {
    pub fn to_string(&self) -> String {
        match self {
            ExprKind::StringLiteral(s) => s.clone(),
            ExprKind::InterpolatedString(InterpolatedStringExpr { parts }) => {
                let mut result = String::new();
                for part in parts {
                    result.push_str(&part.to_string());
                }
                result
            }
            ExprKind::Unary(UnaryExpr { op, expr }) => match op {
                UnaryOp::Negate => format!("-{}", expr.to_string()),
                UnaryOp::Not => format!("!{}", expr.to_string()),
            },
            ExprKind::StructDefault(StructDefaultExpr { name }) => format!("default({})", name),
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
