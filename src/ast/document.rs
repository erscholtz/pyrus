use crate::{ast::Ident, util::Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentConfig {
    pub entries: Vec<DocumentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEntry {
    pub name: Ident,
    pub value: Spanned<String>,
}
