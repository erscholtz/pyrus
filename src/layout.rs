mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use std::cmp::max;
use std::cmp::min;

use crate::diagnostic::CompilerDiagnostic;
use crate::hir::hir_types::Config;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;
use crate::hir::hir_types::HIR;
use crate::layout::allocate::Alignment;
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
    top_margin: usize,
    bottom_margin: usize,
    left_margin: usize,
    right_margin: usize,
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

#[derive(Debug, Clone)]
pub struct ContentBox {
    pub content: Vec<GlyphRow>, // cut down to size
    alignment: Alignment,
    pub coord: Coordinate,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone)]
struct Coordinate {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
pub struct Composer {
    // just figure out one page for now, multiple pages in the future
    pub page: Page,
    pub cur_line: usize,
    content: Vec<ContentRow>,
}

impl Composer {
    pub fn new(hir: &HIR) -> Result<Self, CompilerDiagnostic> {
        Ok(Self {
            page: Composer::setup_pages(&hir.config)?,
            cur_line: 0,
            content: allocate::allocate(&hir.invokes, &hir.layout)?,
        })
    }

    pub fn setup_pages(config: &Config) -> Result<Page, CompilerDiagnostic> {
        let page_type = match config.doc_type {
            DocType::A4 => PageType::A4,
        };
        let orientation = match config.orientation {
            DocOrientation::Portrait => Orientation::Portrait,
        };

        let (width, height) =
            Composer::calculate_grid(&page_type, &orientation)?;

        Ok(Page {
            config: PageConfig {
                grid_width: width,
                grid_height: height,
                top_margin: config.top_margin,
                bottom_margin: config.bottom_margin,
                left_margin: config.left_margin,
                right_margin: config.right_margin,
                orientation: orientation,
                page_type: page_type,
            },
            content: Vec::new(),
        })
    }

    pub fn format(&mut self) {
        let page_width = self.page.config.grid_width;
        // find row

        for row in &self.content {
            match row {
                ContentRow::Single(field) => {
                    // wrap whats needed
                    let lines = Composer::wrap(&field.content, page_width);
                    let line_length = lines.len();
                    // create boxes with start pos, extent
                    self.page.content.push(self.draw_box(
                        lines,
                        field.alignment.clone(),
                        page_width,
                        None,
                    ));
                    self.cur_line += line_length;
                }
                ContentRow::Split { left, right } => {
                    // determine border for each side (min len)
                    let gap = min(
                        page_width / 2,
                        max(
                            left.content.glyphs.len(),
                            right.content.glyphs.len(),
                        ),
                    );
                    // wrap whats needed
                    let l_lines = Composer::wrap(&left.content, gap);
                    let r_lines =
                        Composer::wrap(&right.content, page_width - gap);
                    // create boxes with start pos, extent
                    let line_length = max(l_lines.len(), r_lines.len());
                    self.page.content.push(self.draw_box(
                        l_lines,
                        left.alignment.clone(),
                        gap,
                        None,
                    ));
                    self.page.content.push(self.draw_box(
                        r_lines,
                        right.alignment.clone(),
                        page_width - gap,
                        Some(gap),
                    ));
                    self.cur_line += line_length;
                }
            }
        }
    }

    fn calculate_grid(
        page_type: &PageType,
        orientation: &Orientation,
    ) -> Result<(usize, usize), CompilerDiagnostic> {
        let (x, y) = match page_type {
            PageType::A4 => (166usize, 44usize),
        };
        match orientation {
            Orientation::Portrait => Ok((x, y)),
            Orientation::Landscape => Ok((y, x)),
        }
    }

    fn wrap(content: &GlyphRow, gap: usize) -> Vec<GlyphRow> {
        let mut lines = Vec::new();
        let mut line = Vec::new();
        for glyph in content.glyphs.iter() {
            line.push(glyph.clone());
            if line.len() == gap {
                lines.push(GlyphRow {
                    glyphs: line.clone(),
                    name: "line".to_string() + &lines.len().to_string(),
                    offset: line.len(),
                });
                line = Vec::new();
            }
        }
        if !line.is_empty() {
            lines.push(GlyphRow {
                glyphs: line.clone(),
                name: "line".to_string() + &lines.len().to_string(),
                offset: line.len(),
            });
        }
        lines
    }

    fn draw_box(
        &self,
        lines: Vec<GlyphRow>,
        alignment: Alignment,
        width: usize,
        gap: Option<usize>,
    ) -> ContentBox {
        let coord = if gap.is_some() {
            Coordinate {
                x: gap.unwrap(),
                y: self.cur_line,
            }
        } else {
            Coordinate {
                x: 0,
                y: self.cur_line,
            }
        };

        ContentBox {
            content: lines.clone(),
            alignment,
            coord,
            width,
            height: lines.len(),
        }
    }
}

pub fn layout(hir: &HIR) -> Result<Composer, CompilerDiagnostic> {
    let mut composer = Composer::new(hir)?;
    composer.format();
    Ok(composer)
}
