pub mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use krilla::text::KrillaGlyph;

use crate::diagnostic::Diagnostic;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;
use crate::hir::hir_types::HIR;
use crate::hir::hir_types::Sizing;
use crate::hir::hir_types::TextType;
use crate::layout::allocate::Alignment;
use crate::layout::allocate::ContentRow;
use crate::layout::wrap::field_width;
pub use crate::layout::wrap::text_content;
use crate::layout::wrap::wrap_lines;

pub const BULLET_INDENT: f32 = 10.0;

// Vertical space added before an invoke. Section headings (layouts containing a
// `---` divider) get more room above them than ordinary entries.
pub const ENTRY_GAP: f32 = 3.0;
pub const SECTION_GAP: f32 = 6.0;
// Space between a `---` divider and the content that follows it.
pub const DIVIDER_GAP: f32 = 3.0;

pub fn font_size(size: &Sizing) -> f32 {
    match size {
        Sizing::Sm => 9.0,
        Sizing::Md => 10.0,
        Sizing::Lg => 12.0,
        Sizing::Xl => 20.0,
    }
}

pub fn line_height(size: &Sizing) -> f32 {
    match size {
        Sizing::Sm => 10.5,
        Sizing::Md => 12.5,
        Sizing::Lg => 16.0,
        Sizing::Xl => 24.0,
    }
}

// spacing out what rows we have given the top down margins and remaing rows we
// got on a page

#[derive(Debug, Clone)]
pub struct Page {
    pub config: PageConfig,
    pub content: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct PageConfig {
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub left_margin: f32,
    pub right_margin: f32,
    pub(crate) orientation: Orientation,
    pub(crate) page_type: PageType,
}

#[derive(Debug, Clone)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone)]
pub enum PageType {
    A4,
    Letter,
}

pub enum FieldType {
    Single,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RowStyle {
    None,
    BulletRow,
    TrailingBullet,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub lines: Vec<Line>,
    pub row_style: RowStyle,
    pub alignment: Alignment,
    pub size: Sizing,
    pub height: f32,
    pub width: f32,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub content: TextType,
    pub glyphs: Vec<KrillaGlyph>,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub enum WrappedRow {
    Single {
        content: Vec<Paragraph>,
    },
    Split {
        left: Vec<Paragraph>,
        right: Vec<Paragraph>,
    },
    Seperator,
    Gap(f32),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub content: BlockContentType,
    pub coord: (f32, f32),
    pub width: f32,
    pub height: f32,
}

// NOTE this is a bit of a hack, but I want to make sure that the content of a
// block is either text or a seperator, and not both. This is because I want to
// be able to render the content of a block in a single pass, and not have to
// worry about rendering text and seperators separately. This is also because I
// want to be able to render the content of a block in a single pass, and not
// have to worry about rendering text and seperators separately.
#[derive(Debug, Clone)]
pub enum BlockContentType {
    Text(Vec<Paragraph>),
    Seperator(f32),
}

pub fn calculate_grid(
    page_type: &PageType,
    orientation: &Orientation,
) -> Result<(f32, f32), Diagnostic> {
    let (x, y) = match page_type {
        PageType::A4 => (595.28, 841.89),
        // US Letter, 8.5 x 11 in
        PageType::Letter => (612.0, 792.0),
    };
    match orientation {
        Orientation::Portrait => Ok((x, y)),
        Orientation::Landscape => Ok((y, x)),
    }
}

pub fn layout(hir: &HIR) -> Result<Page, Diagnostic> {
    let page_type = match hir.config.doc_type {
        DocType::A4 => PageType::A4,
        DocType::Letter => PageType::Letter,
    };
    let orientation = match hir.config.orientation {
        DocOrientation::Portrait => Orientation::Portrait,
    };

    let (page_width, _) = calculate_grid(&page_type, &orientation)?;
    let available_width = (page_width - hir.config.left_margin - hir.config.right_margin).max(0.0);

    let mut page = Page {
        config: PageConfig {
            top_margin: hir.config.top_margin,
            bottom_margin: hir.config.bottom_margin,
            left_margin: hir.config.left_margin,
            right_margin: hir.config.right_margin,
            orientation: orientation,
            page_type: page_type,
        },
        content: Vec::new(),
    };

    let mut rows = allocate::allocate(&hir.invokes, &hir.layout)?;

    inscribe::inscribe(&mut rows)?;

    let mut split_line = Vec::new();
    for row in &rows {
        let width = match row {
            ContentRow::Single(_) => available_width,
            ContentRow::Split { left, right } => {
                let left_width = field_width(left);
                let right_width = field_width(right);
                if left_width <= available_width / 2.0 {
                    left_width
                } else if right_width <= available_width / 2.0 {
                    available_width - right_width
                } else {
                    available_width / 2.0
                }
            }
            ContentRow::Seperator | ContentRow::Gap(_) => available_width,
        };
        split_line.push(width);
    }

    let mut wrapped_rows = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let row_content = match row {
            ContentRow::Single(field) => {
                let content = wrap_lines(&field, split_line[i]);
                WrappedRow::Single { content }
            }
            ContentRow::Split { left, right } => {
                let left_content = wrap_lines(left, split_line[i]);
                let right_content = wrap_lines(right, available_width - split_line[i]);
                WrappedRow::Split {
                    left: left_content,
                    right: right_content,
                }
            }
            ContentRow::Seperator => WrappedRow::Seperator, // NOTE: confirm
            ContentRow::Gap(height) => WrappedRow::Gap(*height),
        };
        wrapped_rows.push(row_content);
    }

    let mut text_boxes = Vec::new();
    let mut len = 0.0;
    for (i, row) in wrapped_rows.iter().enumerate() {
        match row {
            WrappedRow::Single { content } => {
                let total_height = content.iter().map(|h| h.height).sum();
                text_boxes.push(Block {
                    content: BlockContentType::Text(content.clone()),
                    coord: (0.0, len),
                    width: split_line[i],
                    height: total_height,
                });
                len += total_height;
            }
            WrappedRow::Split { left, right } => {
                let l_total_height = left.iter().map(|h| h.height).sum();
                text_boxes.push(Block {
                    content: BlockContentType::Text(left.clone()),
                    coord: (0.0, len),
                    width: split_line[i],
                    height: l_total_height,
                });
                let r_total_height = right.iter().map(|h| h.height).sum();
                let right_width = available_width - split_line[i];
                text_boxes.push(Block {
                    content: BlockContentType::Text(right.clone()),
                    coord: (split_line[i], len),
                    width: right_width,
                    height: r_total_height,
                });
                len += l_total_height.max(r_total_height);
            }
            WrappedRow::Seperator => {
                text_boxes.push(Block {
                    content: BlockContentType::Seperator(split_line[i]),
                    coord: (0.0, len),
                    width: split_line[i],
                    height: 0.5,
                });
                len += 0.5 + DIVIDER_GAP;
            }
            WrappedRow::Gap(height) => {
                len += height;
            }
        }
    }

    page.content = text_boxes;

    Ok(page)
}
