use crate::ast::Expr;

#[derive(Debug, Clone)]
pub struct FuncParam {
    pub ty: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct ArgType {
    pub name: String,
    pub ty: String,
}
