mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use crate::diagnostic::CompilerDiagnostic;
use crate::hir::hir_types::Config;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;
use crate::hir::hir_types::HIR;
use crate::layout::allocate::ContentRow;
use crate::layout::inscribe::GlyphRow;

// spacing out what rows we have given the top down margins and remaing rows we
// got on a page

#[derive(Debug, Clone)]
pub struct Page {
    config: PageConfig,
    content: Vec<ContentBox>,
}

#[derive(Debug, Clone)]
pub struct PageConfig {
    grid_width: usize,
    grid_height: usize,
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
    content: ContentRow,
    coord: Coordinate,
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
    page: Page,
    content: Vec<ContentRow>,
}

impl Composer {
    pub fn new(hir: &HIR) -> Result<Self, CompilerDiagnostic> {
        Ok(Self {
            page: Composer::setup_pages(&hir.config)?,
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
}

pub fn layout(hir: &HIR) -> Result<Composer, CompilerDiagnostic> {
    let composer = Composer::new(hir)?;
    Ok(composer)
}
