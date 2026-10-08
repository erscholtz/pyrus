mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use std::cmp::max;
use std::cmp::min;
use std::ops::Range;

use crate::diagnostic::Diagnostic;
use crate::hir::hir_types::Config;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;
use crate::hir::hir_types::HIR;
use crate::hir::hir_types::TextOp;
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
pub struct WrappedRowContents {
    pub content: Vec<String>,
    pub italics_range: Vec<Range<usize>>,
    pub bold_range: Vec<Range<usize>>,
    pub nerd_range: Vec<Range<usize>>,
    pub link_range: Vec<Range<usize>>,
    pub link_hrefs: Vec<String>,
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
    pub content: Vec<String>, // cut down to size
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
    let mut italics_range = Vec::new();
    let mut bold_range = Vec::new();
    let mut nerd_range = Vec::new();
    let mut link_range = Vec::new();
    let mut link_hrefs = Vec::new();

    let mut wrapped_lines = Vec::new();
    for item in &field.content {
        let mut wrapped_line = String::new();
        for word in item.content.split_whitespace() {
            if wrapped_line.len() + word.len() > split {
                wrapped_lines.push(wrapped_line.clone());
                wrapped_line.clear();
            }
            wrapped_line.push_str(word);
            if word != item.content.split_whitespace().last().unwrap() {
                wrapped_line.push(' ');
            }
        }
        wrapped_lines.push(wrapped_line.clone());
        italics_range.extend(item.italic_ranges.clone());
        bold_range.extend(item.bold_ranges.clone());
        nerd_range.extend(item.nerd_font_ranges.clone());
        link_range.extend(item.link_ranges.clone());
        link_hrefs.extend(item.link_hrefs.clone());
    }
    wrapped_contents.push(WrappedRowContents {
        content: wrapped_lines.clone(),
        italics_range,
        bold_range,
        nerd_range,
        link_range,
        link_hrefs,
    });
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
                let max = max(
                    left.content.get(0).map_or(0, |s| s.content.len()),
                    right.content.get(0).map_or(0, |s| s.content.len()),
                );
                let min = min(
                    left.content.get(0).map_or(0, |s| s.content.len()),
                    right.content.get(0).map_or(0, |s| s.content.len()),
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
                for item in content {
                    text_boxes.push(ContentBox {
                        content: item.content.clone(),
                        coord: (0, len),
                        width: page.config.grid_width,
                        height: item.content.len(),
                    });
                    len += item.content.len();
                }
            }
            WrappedRow::Split { left, right } => {
                for l_item in left {
                    let l_max_width = l_item
                        .content
                        .iter()
                        .map(|line| line.chars().count())
                        .max()
                        .unwrap_or(0);
                    text_boxes.push(ContentBox {
                        content: l_item.content.clone(),
                        coord: (0, len),
                        width: l_max_width,
                        height: left.len(),
                    });
                    len += left.len()
                }

                for r_item in right {
                    let r_max_width = r_item
                        .content
                        .iter()
                        .map(|line| line.chars().count())
                        .max()
                        .unwrap_or(0);
                    text_boxes.push(ContentBox {
                        content: r_item.content.clone(),
                        coord: (page.config.grid_width - r_max_width, len),
                        width: r_max_width,
                        height: right.len(),
                    });
                    len += right.len();
                }
            }
        }
    }

    page.content = text_boxes;

    Ok(page)
}
