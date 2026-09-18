use std::collections::HashMap;

use crate::ast::LayoutRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HIR {
    pub file: String,
    pub config: Config,
    pub decls: HashMap<String, Elem>,
    pub layout: HashMap<String, Layout>,
    pub invokes: Vec<Invoke>,
}

impl HIR {
    pub fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            config: Config::default(),
            decls: HashMap::new(),
            layout: HashMap::new(),
            invokes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocType {
    A4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub doc_type: DocType,
    pub top_margin: usize,
    pub bottom_margin: usize,
    pub left_margin: usize,
    pub right_margin: usize,
    pub padding: usize,
}

impl Config {
    pub fn new(
        doc_type: DocType,
        top: usize,
        bottom: usize,
        left: usize,
        right: usize,
        padding: usize,
    ) -> Self {
        Self {
            doc_type,
            top_margin: top,
            bottom_margin: bottom,
            left_margin: left,
            right_margin: right,
            padding: padding,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            doc_type: DocType::A4,
            top_margin: 4,
            bottom_margin: 4,
            left_margin: 4,
            right_margin: 4,
            padding: 1,
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
