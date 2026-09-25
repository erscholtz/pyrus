use std::collections::HashMap;

use crate::ast::LayoutAlignment;
use crate::ast::LayoutRow;
use crate::diagnostic::CompilerDiagnostic;
use crate::hir::hir_types::Invoke;
use crate::hir::hir_types::Layout;
use crate::layout::GlyphRow;
use crate::layout::inscribe;

#[derive(Debug, Clone)]
pub enum ContentRow {
    Single(AllocatedField),
    Split {
        left: AllocatedField,
        right: AllocatedField,
    },
}

#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct AllocatedField {
    pub content: GlyphRow,
    pub alignment: Alignment,
}

pub fn allocate(
    invokes: &Vec<Invoke>,
    layouts: &HashMap<String, Layout>,
) -> Result<Vec<ContentRow>, CompilerDiagnostic> {
    let mut content_rows = Vec::new();
    for invoke in invokes {
        let layout = layouts.get(&invoke.name.clone()).unwrap();
        for layout_row in &layout.layouts {
            match layout_row {
                LayoutRow::Single { field, alignment } => {
                    for elem in &invoke.elements {
                        if field.text == *elem.0 {
                            let allocated_field = AllocatedField {
                                content: inscribe::inscribe(elem, layout)?,
                                alignment: convert_layout(alignment),
                            };
                            content_rows
                                .push(ContentRow::Single(allocated_field));
                        }
                    }
                }
                LayoutRow::Split {
                    left,
                    right,
                    left_alignment,
                    right_alignment,
                } => {
                    let mut left_line = None;
                    for elem in &invoke.elements {
                        if left.text == *elem.0 {
                            left_line = Some(AllocatedField {
                                content: inscribe::inscribe(elem, layout)?,
                                alignment: convert_layout(left_alignment),
                            });
                        }
                    }
                    let mut right_line = None;
                    for elem in &invoke.elements {
                        if right.text == *elem.0 {
                            right_line = Some(AllocatedField {
                                content: inscribe::inscribe(elem, layout)?,
                                alignment: convert_layout(right_alignment),
                            });
                        }
                    }
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
