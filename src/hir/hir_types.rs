use std::collections::HashMap;

use crate::ast::LayoutRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HIR {
    pub file: String,
    pub decls: HashMap<String, Elem>,
    pub layout: HashMap<String, Layout>,
    pub invokes: Vec<Invoke>,
}

impl HIR {
    pub fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            decls: HashMap::new(),
            layout: HashMap::new(),
            invokes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elem {
    pub elements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sizing {
    Sm,
    Md,
    Lg,
    Xl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub layouts: Vec<LayoutRow>,
    pub sizing: HashMap<String, Sizing>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
    Bold(String),
    Italic(String),
    NerdFont(String),
    Link { label: String, href: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invoke {
    pub name: String,
    pub elements: HashMap<String, Vec<Content>>,
}
