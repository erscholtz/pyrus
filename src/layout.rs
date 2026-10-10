mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use std::cmp::max;
use std::cmp::min;
use std::ops::Range;

use crate::diagnostic::Diagnostic;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;
use crate::hir::hir_types::HIR;
use crate::hir::hir_types::TextOp;
use crate::hir::hir_types::TextType;
use crate::layout::allocate::Alignment;
use crate::layout::allocate::AllocatedField;
use crate::layout::allocate::ContentRow;
use crate::layout::inscribe::GlyphRow;

// spacing out what rows we have given the top down margins and remaing rows we
// got on a page

#[derive(Debug, Clone)]
pub struct Page {
    pub config: PageConfig,
    pub content: Vec<ContentBox>,
}

#[derive(Debug, Clone)]
pub struct PageConfig {
    pub grid_width: usize,
    pub grid_height: usize,
    pub top_margin: usize,
    pub bottom_margin: usize,
    pub left_margin: usize,
    pub right_margin: usize,
    orientation: Orientation,
    page_type: PageType,
}

#[derive(Debug, Clone)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone)]
pub enum PageType {
    A4,
}

pub enum FieldType {
    Single,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum RowStyle {
    None,
    BulletRow,
    WrappedBullet,
}

#[derive(Debug, Clone)]
pub struct WrappedRowContents {
    pub row: Vec<TextType>,
    pub row_style: RowStyle,
}

#[derive(Debug, Clone)]
pub enum WrappedRow {
    Single {
        content: Vec<WrappedRowContents>,
    },
    Split {
        left: Vec<WrappedRowContents>,
        right: Vec<WrappedRowContents>,
    },
}

#[derive(Debug, Clone)]
pub struct ContentBox {
    pub rows: Vec<WrappedRowContents>, // cut down to size
    pub coord: (usize, usize),
    width: usize,
    height: usize,
}

pub fn calculate_grid(
    page_type: &PageType,
    orientation: &Orientation,
) -> Result<(usize, usize), Diagnostic> {
    let (x, y) = match page_type {
        PageType::A4 => (90usize, 44usize),
    };
    match orientation {
        Orientation::Portrait => Ok((x, y)),
        Orientation::Landscape => Ok((y, x)),
    }
}

pub fn wrap_lines(
    field: &AllocatedField,
    split: usize,
) -> Vec<WrappedRowContents> {
    let mut wrapped_contents = Vec::new();

    for item in &field.content {
        let mut row = Vec::new();
        let mut row_len = 0;
        for text_type in &item.content {
            let text = match text_type {
                TextType::Text(s) => s,
                TextType::Bold(s) => s,
                TextType::Italic(s) => s,
                TextType::Nerd(s) => s,
                TextType::Link { label, href } => label,
            };
            if row_len + text.len() > split {
                // TODO extra work to split on words and all that
                let mut pos = text.len() - (row_len + text.len() - split);
                for i in (0..pos).rev() {
                    pos = i;
                    if text.chars().nth(i).unwrap().is_whitespace() {
                        break;
                    }
                    if i == 0 {
                        // fallback to splitting at the max length if no whitespace is found
                        pos = row_len + text.len() - split;
                    }
                }
                let (left, right) = text.split_at(pos);
                row.push(match text_type {
                    TextType::Text(_) => TextType::Text(left.to_string()),
                    TextType::Bold(_) => TextType::Bold(left.to_string()),
                    TextType::Italic(_) => TextType::Italic(left.to_string()),
                    TextType::Nerd(_) => TextType::Nerd(left.to_string()),
                    TextType::Link { label, href } => TextType::Link {
                        label: left.to_string(),
                        href: href.clone(),
                    },
                });
                wrapped_contents.push(WrappedRowContents {
                    row: row.clone(),
                    row_style: RowStyle::None, // FIXME this needs to get figured out in extra work here
                });
                row.clear();
                row.push(match text_type {
                    TextType::Text(_) => TextType::Text(right.to_string()),
                    TextType::Bold(_) => TextType::Bold(right.to_string()),
                    TextType::Italic(_) => TextType::Italic(right.to_string()),
                    TextType::Nerd(_) => TextType::Nerd(right.to_string()),
                    TextType::Link { label, href } => TextType::Link {
                        label: right.to_string(),
                        href: href.clone(),
                    },
                });
                row_len = right.len();
            } else {
                row_len += text.len();
                row.push(text_type.clone());
            }
        }
        wrapped_contents.push(WrappedRowContents {
            row: row.clone(),
            row_style: RowStyle::None, // FIXME this needs to get figured out in extra work here
        });
    }

    wrapped_contents
}

pub fn layout(hir: &HIR) -> Result<Page, Diagnostic> {
    let page_type = match hir.config.doc_type {
        DocType::A4 => PageType::A4,
    };
    let orientation = match hir.config.orientation {
        DocOrientation::Portrait => Orientation::Portrait,
    };

    let (width, height) = calculate_grid(&page_type, &orientation)?;

    let mut page = Page {
        config: PageConfig {
            grid_width: width,
            grid_height: height,
            top_margin: hir.config.top_margin,
            bottom_margin: hir.config.bottom_margin,
            left_margin: hir.config.left_margin,
            right_margin: hir.config.right_margin,
            orientation: orientation,
            page_type: page_type,
        },
        content: Vec::new(),
    };

    println!("Allocating rows \n_____________________");
    let rows = allocate::allocate(&hir.invokes, &hir.layout).unwrap();
    for row in &rows {
        println!("{}", row);
    }

    println!("Measuring rows \n_____________________");
    let mut split_line = Vec::new();
    for row in &rows {
        let width = match row {
            ContentRow::Single(_) => page.config.grid_width,
            ContentRow::Split { left, right } => {
                let left_len: usize = left
                    .content
                    .iter()
                    .flat_map(|text_op| text_op.content.iter())
                    .map(|item| match item {
                        TextType::Text(s)
                        | TextType::Bold(s)
                        | TextType::Italic(s)
                        | TextType::Nerd(s) => s.len(),
                        TextType::Link { label, .. } => label.len(),
                    })
                    .sum();
                let right_len: usize = right
                    .content
                    .iter()
                    .flat_map(|text_op| text_op.content.iter())
                    .map(|item| match item {
                        TextType::Text(s)
                        | TextType::Bold(s)
                        | TextType::Italic(s)
                        | TextType::Nerd(s) => s.len(),
                        TextType::Link { label, .. } => label.len(),
                    })
                    .sum();
                let max = max(left_len, right_len);
                let min = min(left_len, right_len);
                if max > page.config.grid_width / 2
                    && min > page.config.grid_width / 2
                {
                    page.config.grid_width / 2
                } else if max > page.config.grid_width / 2 {
                    page.config.grid_width / 2 - min
                } else {
                    max
                }
            }
        };
        split_line.push(width);
        println!("Width: {}", width);
    }

    println!("Wrapping rows \n_____________________");
    let mut wrapped_rows = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let row_content = match row {
            ContentRow::Single(field) => {
                let content = wrap_lines(&field, split_line[i]);
                WrappedRow::Single { content }
            }
            ContentRow::Split { left, right } => {
                let left_content = wrap_lines(left, split_line[i]);
                let right_content =
                    wrap_lines(right, page.config.grid_width - split_line[i]);
                WrappedRow::Split {
                    left: left_content,
                    right: right_content,
                }
            }
        };
        wrapped_rows.push(row_content);
    }

    println!("Split line: {:?}", wrapped_rows);

    // TODO have to do alignment in the laying out step, krilla only takes in coords

    println!("Laying out rows \n_____________________");
    let mut text_boxes = Vec::new();
    let mut len = 0;
    for (i, row) in wrapped_rows.iter().enumerate() {
        println!("Row {}: {:?}", i, row);

        match row {
            WrappedRow::Single { content } => {
                text_boxes.push(ContentBox {
                    rows: content.clone(),
                    coord: (0, len),
                    width: split_line[i],
                    height: content.len(),
                });
                len += content.len();
            }
            WrappedRow::Split { left, right } => {
                text_boxes.push(ContentBox {
                    rows: left.clone(),
                    coord: (0, len),
                    width: split_line[i],
                    height: left.len(),
                });
                len += left.len();

                let right_len = page.config.grid_width - split_line[i];
                text_boxes.push(ContentBox {
                    rows: right.clone(),
                    coord: (page.config.grid_width - right_len, len),
                    width: right_len,
                    height: right.len(),
                });
                len += right.len();
            }
        }
    }

    page.content = text_boxes;

    Ok(page)
}
