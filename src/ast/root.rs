use crate::ast::DocumentConfig;
use crate::ast::ElemDecl;
use crate::ast::ElemInvoke;
use crate::ast::LayoutDecl;
use crate::diagnostic::Span;
use crate::util::Spanned;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Document(DocumentConfig),
    ElemDecl(ElemDecl),
    LayoutDecl(LayoutDecl),
    ElemInvoke(ElemInvoke),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ast {
    pub file: String,
    pub items: Vec<Spanned<Item>>,
}
