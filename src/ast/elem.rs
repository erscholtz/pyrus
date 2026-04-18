use std::collections::HashMap;

use crate::ast::ArgType;
use crate::ast::Expr;
use crate::util::Spanned;

/// A document element, with a location in the source file.
pub type DocElem = Spanned<DocElemKind>;
/// A map of attribute names to expressions. can be `None` if there are no attributes.
pub type Attributes = Option<HashMap<String, Expr>>;

/// Thin wrapper types for document elements.
/// Each element type can be parsed independently, then converted to DocElemKind.
#[derive(Debug, Clone)]
pub struct TextElem {
    pub content: Expr,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct ImageElem {
    pub src: String,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct TableElem {
    pub table: Vec<Vec<DocElem>>,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct ListElem {
    pub items: Vec<DocElem>,
    pub attributes: Attributes,
    pub numbered: bool,
}

#[derive(Debug, Clone)]
pub struct CodeElem {
    pub content: String,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct CallElem {
    pub name: String,
    pub args: Vec<ArgType>,
    pub children: Option<Vec<DocElem>>,
}

#[derive(Debug, Clone)]
pub struct LinkElem {
    pub href: String,
    pub content: String,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct SectionElem {
    pub elements: Vec<DocElem>,
    pub attributes: Attributes,
}

#[derive(Debug, Clone)]
pub struct ChildrenElem {
    pub render_childen: bool,
}

/// The kind of document element - used within Spanned<DocElemKind>
#[derive(Debug, Clone)]
pub enum DocElemKind {
    Text(TextElem),
    Image(ImageElem),
    Table(TableElem),
    List(ListElem),
    Code(CodeElem),
    Call(CallElem),
    Link(LinkElem),
    Section(SectionElem),
    Children(ChildrenElem),
}

// Conversions from wrapper types to DocElemKind
impl From<TextElem> for DocElemKind {
    fn from(e: TextElem) -> Self {
        DocElemKind::Text(e)
    }
}

impl From<ImageElem> for DocElemKind {
    fn from(e: ImageElem) -> Self {
        DocElemKind::Image(e)
    }
}

impl From<TableElem> for DocElemKind {
    fn from(e: TableElem) -> Self {
        DocElemKind::Table(e)
    }
}

impl From<ListElem> for DocElemKind {
    fn from(e: ListElem) -> Self {
        DocElemKind::List(e)
    }
}

impl From<CodeElem> for DocElemKind {
    fn from(e: CodeElem) -> Self {
        DocElemKind::Code(e)
    }
}

impl From<CallElem> for DocElemKind {
    fn from(e: CallElem) -> Self {
        DocElemKind::Call(e)
    }
}

impl From<LinkElem> for DocElemKind {
    fn from(e: LinkElem) -> Self {
        DocElemKind::Link(e)
    }
}

impl From<SectionElem> for DocElemKind {
    fn from(e: SectionElem) -> Self {
        DocElemKind::Section(e)
    }
}

impl From<ChildrenElem> for DocElemKind {
    fn from(e: ChildrenElem) -> Self {
        DocElemKind::Children(e)
    }
}
