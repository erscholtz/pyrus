use crate::ast::Expr;
use crate::ast::Stmt;
use crate::ast::Type;

#[derive(Debug, Clone)]
pub struct FuncParam {
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct ArgType {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct FuncDeclStmt {
    pub name: String,
    pub args: Vec<FuncParam>,
    pub body: Vec<Stmt>,
    pub return_type: Option<Type>,
}
