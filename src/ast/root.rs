use crate::ast::DocElem;
use crate::ast::Stmt;
use crate::ast::StyleRule;

//blocks

#[derive(Debug, Clone)]
pub struct TemplateBlock {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct DocumentBlock {
    pub elements: Vec<DocElem>,
}

#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub statements: Vec<StyleRule>,
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub file: String,
    pub template: Option<TemplateBlock>,
    pub document: Option<DocumentBlock>,
    pub style: Option<StyleBlock>,
}
