mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use std::cmp::max;
use std::cmp::min;

use crate::diagnostic::Diagnostic;
use crate::hir::hir_types::Config;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;
use crate::hir::hir_types::HIR;
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
pub enum WrappedRow {
    Single(Vec<String>),
    Split(Vec<String>, Vec<String>),
}

#[derive(Debug, Clone)]
pub struct ContentBox {
    pub content: Vec<String>, // cut down to size
    pub coord: (usize, usize),
    width: usize,
    height: usize,
}

#[derive(Debug, Clone)]
pub struct Composer {
    // just figure out one page for now, multiple pages in the future
    pub page: Page,
    pub cur_line: usize,
    content: Vec<ContentRow>,
}

impl Composer {
    fn calculate_grid(
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
}

pub fn wrap_lines(field: &AllocatedField, split: usize) -> Vec<String> {
    let mut wrapped_lines = Vec::new();
    let mut wrapped_line = String::new();
    for item in &field.content {
        for char in item.chars() {
            if wrapped_line.len() == split {
                wrapped_lines.push(wrapped_line.clone());
                wrapped_line.clear();
            }
            wrapped_line.push(char);
        }
        wrapped_lines.push(wrapped_line.clone());
        wrapped_line.clear();
    }
    wrapped_lines
}

pub fn layout(hir: &HIR) -> Result<Page, Diagnostic> {
    let page_type = match hir.config.doc_type {
        DocType::A4 => PageType::A4,
    };
    let orientation = match hir.config.orientation {
        DocOrientation::Portrait => Orientation::Portrait,
    };

    let (width, height) = Composer::calculate_grid(&page_type, &orientation)?;

    println!("Setting up page \n_____________________");
    println!("Page size: {}x{}", width, height);
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
                let max = max(
                    left.content.get(0).map_or(0, |s| s.len()),
                    right.content.get(0).map_or(0, |s| s.len()),
                );
                let min = min(
                    left.content.get(0).map_or(0, |s| s.len()),
                    right.content.get(0).map_or(0, |s| s.len()),
                );
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
                let wrapped_lines = wrap_lines(field, split_line[i]);
                WrappedRow::Single(wrapped_lines)
            }
            ContentRow::Split { left, right } => {
                let left_wrapped_lines = wrap_lines(left, split_line[i]);
                let right_wrapped_lines =
                    wrap_lines(right, page.config.grid_width - split_line[i]);
                WrappedRow::Split(left_wrapped_lines, right_wrapped_lines)
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
            WrappedRow::Single(text) => {
                text_boxes.push(ContentBox {
                    content: text.clone(),
                    coord: (0, len),
                    width: page.config.grid_width,
                    height: text.len(),
                });
                len += text.len();
            }
            WrappedRow::Split(left, right) => {
                let max_width = left
                    .iter()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0);
                text_boxes.push(ContentBox {
                    content: left.clone(),
                    coord: (0, len),
                    width: max_width, // NOTE we calculate this width before we should use it
                    height: left.len(),
                });
                text_boxes.push(ContentBox {
                    content: right.clone(),
                    coord: (max_width + 3, len), // NOTE we calculate this gap before we should use it
                    width: page.config.grid_width
                        - text_boxes.last().unwrap().width, // NOTE we calculate this width before we should use it
                    height: right.len(),
                });
                len += left.len() + right.len();
            }
        }
    }

    page.content = text_boxes;

    Ok(page)
}
