use crate::ast::DocElem;
use crate::ast::Expr;
use crate::ast::FuncDeclStmt;
use crate::util::Spanned;

pub type Stmt = Spanned<StmtKind>;

/// Thin wrapper types for statement kinds.
/// Each statement type can be parsed independently, then converted to StmtKind.
#[derive(Debug, Clone)]
pub struct DefaultSetStmt {
    pub key: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct VarAssignStmt {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstAssignStmt {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
    pub else_body: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub iterator: String,
    pub iterable: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum ReturnStmt {
    Expr(Expr),
    DocElem(DocElem),
}

#[derive(Debug, Clone)]
pub struct ChildrenStmt {
    pub children: bool,
}

/// The kind of statement - used within Spanned<StmtKind>
#[derive(Debug, Clone)]
pub enum StmtKind {
    DefaultSet(DefaultSetStmt),
    VarAssign(VarAssignStmt),
    ConstAssign(ConstAssignStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    FuncDecl(FuncDeclStmt),
    Children(ChildrenStmt),
}

// Conversions from wrapper types to StmtKind
impl From<DefaultSetStmt> for StmtKind {
    fn from(s: DefaultSetStmt) -> Self {
        StmtKind::DefaultSet(s)
    }
}

impl From<VarAssignStmt> for StmtKind {
    fn from(s: VarAssignStmt) -> Self {
        StmtKind::VarAssign(s)
    }
}

impl From<ConstAssignStmt> for StmtKind {
    fn from(s: ConstAssignStmt) -> Self {
        StmtKind::ConstAssign(s)
    }
}

impl From<IfStmt> for StmtKind {
    fn from(s: IfStmt) -> Self {
        StmtKind::If(s)
    }
}

impl From<WhileStmt> for StmtKind {
    fn from(s: WhileStmt) -> Self {
        StmtKind::While(s)
    }
}

impl From<ForStmt> for StmtKind {
    fn from(s: ForStmt) -> Self {
        StmtKind::For(s)
    }
}

impl From<ReturnStmt> for StmtKind {
    fn from(s: ReturnStmt) -> Self {
        StmtKind::Return(s)
    }
}

impl From<FuncDeclStmt> for StmtKind {
    fn from(s: FuncDeclStmt) -> Self {
        StmtKind::FuncDecl(s)
    }
}

impl From<ChildrenStmt> for StmtKind {
    fn from(s: ChildrenStmt) -> Self {
        StmtKind::Children(s)
    }
}
