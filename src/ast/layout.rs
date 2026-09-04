use crate::ast::Ident;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutDecl {
    pub element: Ident,
    pub items: Vec<LayoutItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutItem {
    Row(LayoutRow),
    Property(LayoutProperty),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutRow {
    Single { field: Ident },
    Split { left: Ident, right: Ident },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutProperty {
    pub field: Ident,
    pub value: Ident,
}
