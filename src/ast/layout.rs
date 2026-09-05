use crate::ast::Ident;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutDecl {
    pub element: Ident,
    pub rows: Vec<LayoutRow>,
    pub props: Vec<LayoutProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutAlignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutRow {
    Single {
        field: Ident,
        alignment: LayoutAlignment,
    },
    Split {
        left: Ident,
        right: Ident,
        left_alignment: LayoutAlignment,
        right_alignment: LayoutAlignment,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutProperty {
    pub field: Ident,
    pub value: Ident,
}
