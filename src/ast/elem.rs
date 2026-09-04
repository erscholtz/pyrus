use crate::ast::{Content, Ident, InlineText};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElemDecl {
    pub name: Ident,
    pub fields: Vec<Ident>,
    pub content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElemInvoke {
    pub name: Ident,
    pub fields: Vec<FieldValue>,
    pub content: Option<Content>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValue {
    pub name: Ident,
    pub value: InlineText,
}
