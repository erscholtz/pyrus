use std::collections::HashMap;
use std::fmt::Display;

use crate::ast::LayoutAlignment;
use crate::ast::LayoutRow;
use crate::diagnostic::Diagnostic;
use crate::hir::hir_types::Invoke;
use crate::hir::hir_types::Layout;
use crate::hir::hir_types::Sizing;
use crate::hir::hir_types::TextOp;
use crate::layout::inscribe::GlyphRun;

#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub struct AllocatedField {
    pub glyph_run: Vec<GlyphRun>,
    pub content: Vec<TextOp>,
    pub size: Sizing,
    pub alignment: Alignment,
}

impl Display for AllocatedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.content)
    }
}

#[derive(Debug)]
pub enum ContentRow {
    Single(AllocatedField),
    Split {
        left: AllocatedField,
        right: AllocatedField,
    },
}

impl Display for ContentRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentRow::Single(field) => write!(f, "{}", field),
            ContentRow::Split { left, right } => {
                write!(f, "{} / {}", left, right)
            }
        }
    }
}

pub fn allocate(
    invokes: &Vec<Invoke>,
    layouts: &HashMap<String, Layout>,
) -> Result<Vec<ContentRow>, Diagnostic> {
    let mut content_rows = Vec::new();
    for invoke in invokes {
        let layout = layouts.get(&invoke.name.clone()).unwrap();
        for layout_row in &layout.layouts {
            match layout_row {
                LayoutRow::Single { field, alignment } => {
                    let elem = &invoke.elements.get(&field.text).unwrap();
                    let size = layout.sizing.get(&field.text).unwrap();
                    let allocated_field = AllocatedField {
                        glyph_run: Vec::new(),
                        content: elem.iter().map(|op| op.clone()).collect(),
                        size: size.clone(),
                        alignment: convert_layout(alignment),
                    };
                    content_rows.push(ContentRow::Single(allocated_field));
                }
                LayoutRow::Split {
                    left,
                    right,
                    left_alignment,
                    right_alignment,
                } => {
                    let l_elem = &invoke.elements.get(&left.text).unwrap();
                    let l_size = layout.sizing.get(&left.text).unwrap();
                    let left_line = Some(AllocatedField {
                        glyph_run: Vec::new(),
                        content: l_elem.iter().map(|op| op.clone()).collect(),
                        size: l_size.clone(),
                        alignment: convert_layout(left_alignment),
                    });
                    let r_elem = &invoke.elements.get(&right.text).unwrap();
                    let r_size = layout.sizing.get(&right.text).unwrap();
                    let right_line = Some(AllocatedField {
                        glyph_run: Vec::new(),
                        content: r_elem.iter().map(|op| op.clone()).collect(),
                        size: r_size.clone(),
                        alignment: convert_layout(right_alignment),
                    });
                    if let (Some(left_line), Some(right_line)) =
                        (left_line, right_line)
                    {
                        content_rows.push(ContentRow::Split {
                            left: left_line,
                            right: right_line,
                        });
                    } else {
                        // TODO Err here
                    }
                }
            }
        }
    }

    Ok(content_rows)
}

fn convert_layout(alignment: &LayoutAlignment) -> Alignment {
    match alignment {
        LayoutAlignment::Left => Alignment::Left,
        LayoutAlignment::Right => Alignment::Right,
        LayoutAlignment::Centre => Alignment::Center,
    }
}
