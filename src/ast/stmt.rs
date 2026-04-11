use crate::ast::DocElem;
use crate::ast::Expr;
use crate::ast::FuncParam;
use crate::util::Spanned;

pub type Stmt = Spanned<StmtKind>;

#[derive(Debug, Clone)]
pub enum StmtKind {
    DefaultSet {
        key: String,
        value: Expr,
    },
    VarAssign {
        name: String,
        value: Expr,
    },
    ConstAssign {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        iterator: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Return {
        expr: Expr,
    },
    FuncDecl {
        name: String,
        args: Vec<FuncParam>,
        body: Vec<Stmt>,
        return_type: Option<String>,
    },
    DocElemEmit {
        element: DocElem,
    },
    Children {
        children: bool,
    },
}
