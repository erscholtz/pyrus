use std::{collections::HashMap, ops::Range};

use crate::{ast::LayoutRow, hir::hir_passes::collect_invokes::TextStyle};

#[derive(Debug, Clone, PartialEq)]
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
pub enum DocOrientation {
    Portrait,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub doc_type: DocType,
    pub orientation: DocOrientation,
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub left_margin: f32,
    pub right_margin: f32,
    pub padding: usize,
}

impl Config {
    pub fn new(
        doc_type: DocType,
        doc_orientation: DocOrientation,
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
        padding: usize,
    ) -> Self {
        Self {
            doc_type,
            orientation: doc_orientation,
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
            orientation: DocOrientation::Portrait,
            top_margin: 40 as f32,
            bottom_margin: 40 as f32,
            left_margin: 40 as f32,
            right_margin: 40 as f32,
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
pub enum TextType {
    Text(String),
    Bold(String),
    Italic(String),
    Nerd(String),
    Link { label: String, href: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextOp {
    pub content: Vec<TextType>,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invoke {
    pub name: String,
    pub elements: HashMap<String, Vec<TextOp>>, // NOTE vec because of bullet lists
}
