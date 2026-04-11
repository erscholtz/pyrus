use std::collections::HashMap;

use crate::ast::ArgType;
use crate::ast::Expr;
use crate::util::Spanned;

pub type DocElem = Spanned<DocElemKind>;

#[derive(Debug, Clone)]
pub enum DocElemKind {
    Text {
        content: Expr,
        attributes: HashMap<String, Expr>,
    },
    Image {
        src: String,
        attributes: HashMap<String, Expr>,
    },
    Table {
        table: Vec<Vec<DocElem>>,
        attributes: HashMap<String, Expr>,
    },
    List {
        items: Vec<DocElem>,
        attributes: HashMap<String, Expr>,
        numbered: bool,
    },
    Code {
        content: String,
        attributes: HashMap<String, Expr>,
    },
    Call {
        name: String,
        args: Vec<ArgType>,
        children: Vec<DocElem>, // NOTE: maybe should be wrapped in a section
    },
    Link {
        href: String,
        content: String,
        attributes: HashMap<String, Expr>,
    },
    Section {
        elements: Vec<DocElem>,
        attributes: HashMap<String, Expr>,
    },
}
