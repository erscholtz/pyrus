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

impl Default for Ast {
    fn default() -> Self {
        Self {
            file: String::new(),
            template: None,
            document: None,
            style: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    String,
    Int,
    Float,
    DocElem,
    Var, // for args
}
