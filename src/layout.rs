pub mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use std::cmp::max;
use std::collections::BTreeMap;
use std::ops::Range;

use crate::diagnostic::Diagnostic;
use crate::hir::hir_passes::collect_invokes::TextStyle;
use crate::hir::hir_types::DocOrientation;
use crate::hir::hir_types::DocType;
use crate::hir::hir_types::HIR;
use crate::hir::hir_types::Sizing;
use crate::hir::hir_types::TextType;
use crate::layout::allocate::Alignment;
use crate::layout::allocate::AllocatedField;
use crate::layout::allocate::ContentRow;
use krilla::text::{GlyphId, KrillaGlyph};

pub const BULLET_INDENT: f32 = 10.0;

// spacing out what rows we have given the top down margins and remaing rows we
// got on a page

#[derive(Debug, Clone)]
pub struct Page {
    pub config: PageConfig,
    pub content: Vec<ContentBox>,
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
pub struct WrappedRowContents {
    pub row: Vec<WrappedText>,
    pub row_style: RowStyle,
    pub alignment: Alignment,
    pub size: Sizing,
    pub height: f32,
    // pub width: f32, // TODO: this is kinda hard to implement right now, have to rework wrapping first
}

#[derive(Debug, Clone)]
pub struct WrappedText {
    pub content: TextType,
    pub glyphs: Vec<KrillaGlyph>,
    pub width: f32,
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
    pub coord: (f32, f32),
    pub(crate) width: f32,
    height: f32,
}

pub fn calculate_grid(
    page_type: &PageType,
    orientation: &Orientation,
) -> Result<(f32, f32), Diagnostic> {
    let (x, y) = match page_type {
        PageType::A4 => (595.28, 841.89),
    };
    match orientation {
        Orientation::Portrait => Ok((x, y)),
        Orientation::Landscape => Ok((y, x)),
    }
}

pub fn text_content(text: &TextType) -> &str {
    match text {
        TextType::Text(s)
        | TextType::Bold(s)
        | TextType::Italic(s)
        | TextType::Nerd(s) => s,
        TextType::Link { label, .. } => label,
    }
}

fn slice_text(text: &TextType, range: Range<usize>) -> TextType {
    let content = text_content(text)[range].to_string();
    match text {
        TextType::Text(_) => TextType::Text(content),
        TextType::Bold(_) => TextType::Bold(content),
        TextType::Italic(_) => TextType::Italic(content),
        TextType::Nerd(_) => TextType::Nerd(content),
        TextType::Link { href, .. } => TextType::Link {
            label: content,
            href: href.clone(),
        },
    }
}

struct Cluster {
    run: usize,
    range: Range<usize>,
    width: f32,
    whitespace: bool,
    newline: bool,
}

// Group all glyphs belonging to the same source cluster before measuring.
// Sorting source offsets also keeps UTF-8 slicing independent of glyph order.
fn run_clusters(
    run: usize,
    text: &str,
    glyphs: &inscribe::GlyphRun,
    font_size: f32,
) -> Vec<Cluster> {
    let mut advances = BTreeMap::<usize, f32>::new();
    for (info, pos) in glyphs
        .buffer
        .glyph_infos()
        .iter()
        .zip(glyphs.buffer.glyph_positions())
    {
        *advances.entry(info.cluster as usize).or_default() +=
            pos.x_advance as f32 * font_size / glyphs.units_per_em;
    }
    let mut starts: Vec<_> = advances.keys().copied().collect();
    starts.push(text.len());
    starts
        .windows(2)
        .map(|bounds| {
            let range = bounds[0]..bounds[1];
            let source = &text[range.clone()];
            Cluster {
                run,
                width: advances[&range.start],
                range,
                whitespace: source.chars().all(|c| {
                    c.is_whitespace() && c != '\u{00a0}' && c != '\u{202f}'
                }),
                newline: source.contains(['\n', '\r', '\u{2028}', '\u{2029}']),
            }
        })
        .collect()
}

fn wrapped_fragment(
    text: &TextType,
    run: &inscribe::GlyphRun,
    range: Range<usize>,
    font_size: f32,
) -> WrappedText {
    let mut boundaries: Vec<_> = run
        .buffer
        .glyph_infos()
        .iter()
        .map(|info| info.cluster as usize)
        .collect();
    boundaries.push(text_content(text).len());
    boundaries.sort_unstable();
    boundaries.dedup();
    let glyphs: Vec<_> = run
        .buffer
        .glyph_infos()
        .iter()
        .zip(run.buffer.glyph_positions())
        .filter(|(info, _)| range.contains(&(info.cluster as usize)))
        .map(|(info, pos)| {
            let start = info.cluster as usize;
            let index = boundaries.binary_search(&start).unwrap();
            let end = boundaries[index + 1];
            KrillaGlyph::new(
                GlyphId::new(info.glyph_id),
                pos.x_advance as f32 / run.units_per_em,
                pos.x_offset as f32 / run.units_per_em,
                pos.y_offset as f32 / run.units_per_em,
                pos.y_advance as f32 / run.units_per_em,
                start - range.start..end - range.start,
                None,
            )
        })
        .collect();
    let width = glyphs.iter().map(|glyph| glyph.x_advance * font_size).sum();
    WrappedText {
        content: slice_text(text, range),
        glyphs,
        width,
    }
}

pub fn wrap_lines(
    field: &AllocatedField,
    split: f32,
) -> Vec<WrappedRowContents> {
    let mut wrapped_contents = Vec::new();
    // Buffers are flattened across ALL TextOps, not indexed per paragraph.
    let mut runs = field.glyph_run.iter();
    for item in &field.content {
        let font_size = match field.size {
            Sizing::Sm => 10 as f32,
            Sizing::Md => 11 as f32,
            Sizing::Lg => 12 as f32,
            Sizing::Xl => 20 as f32,
        };
        let font_spacing = match field.size {
            Sizing::Sm => 10 as f32 + 2 as f32,
            Sizing::Md => 12 as f32 + 4 as f32,
            Sizing::Lg => 14 as f32 + 4 as f32,
            Sizing::Xl => 24 as f32 + 6 as f32,
        };
        let paragraph_runs: Vec<_> = item
            .content
            .iter()
            .map(|text| {
                (
                    text,
                    runs.next().expect("text must have a matching glyph run"),
                )
            })
            .collect();
        let clusters: Vec<_> = paragraph_runs
            .iter()
            .enumerate()
            .flat_map(|(index, (text, run))| {
                run_clusters(index, text_content(text), run, font_size)
            })
            .collect();
        let mut row_style = match item.style {
            TextStyle::Paragraph => RowStyle::None,
            TextStyle::BulletList => RowStyle::BulletRow,
        };
        // FIXME ?? strange looking code I dont like it
        let available = (split
            - if row_style == RowStyle::BulletRow {
                BULLET_INDENT
            } else {
                0.0
            })
        .max(0.0);
        let mut start = 0;
        // An empty paragraph still occupies a line.
        loop {
            let mut end = start;
            let mut width = 0.0;
            let mut last_break = None;
            while end < clusters.len() && !clusters[end].newline {
                let cluster = &clusters[end];
                // A whitespace cluster can end a line even if its own advance
                // exceeds the width; trailing whitespace is removed below.
                if width + cluster.width > available && !cluster.whitespace {
                    break;
                }
                width += cluster.width;
                end += 1;
                if cluster.whitespace {
                    last_break = Some(end);
                }
            }
            let newline = end < clusters.len() && clusters[end].newline;
            let cut = if end == clusters.len() || newline {
                end
            } else {
                // Oversized clusters get their own line, ensuring progress.
                last_break.unwrap_or(end.max(start + 1))
            };
            let mut visible_end = cut;
            while visible_end > start && clusters[visible_end - 1].whitespace {
                visible_end -= 1;
            }
            let mut row = Vec::new();
            let mut index = start;
            while index < visible_end {
                let first = &clusters[index];
                let mut end_index = index + 1;
                while end_index < visible_end
                    && clusters[end_index].run == first.run
                {
                    end_index += 1;
                }
                let (text, run) = paragraph_runs[first.run];
                row.push(wrapped_fragment(
                    text,
                    run,
                    first.range.start..clusters[end_index - 1].range.end,
                    font_size,
                ));
                index = end_index;
            }
            wrapped_contents.push(WrappedRowContents {
                row,
                row_style: row_style.clone(),
                alignment: field.alignment.clone(),
                size: field.size.clone(),
                height: font_spacing,
            });
            if row_style == RowStyle::BulletRow {
                row_style = RowStyle::TrailingBullet;
            }
            start = if newline { cut + 1 } else { cut };
            // CRLF represents one break, including across styled runs.
            if newline && start < clusters.len() {
                let previous = &clusters[cut];
                let next = &clusters[start];
                let previous_text =
                    text_content(paragraph_runs[previous.run].0);
                let next_text = text_content(paragraph_runs[next.run].0);
                if previous_text[previous.range.clone()].ends_with('\r')
                    && &next_text[next.range.clone()] == "\n"
                {
                    start += 1;
                }
            }
            // Do not carry separator whitespace onto continuation lines.
            while start < clusters.len()
                && clusters[start].whitespace
                && !clusters[start].newline
            {
                start += 1;
            }
            if start >= clusters.len() && !newline {
                break;
            }
        }
    }
    wrapped_contents
}

// FIXME ?? bad code there should be function chaining 4 layers deep that is crazy
fn field_width(field: &AllocatedField) -> f32 {
    let font_size = match field.size {
        Sizing::Sm => 10 as f32,
        Sizing::Md => 12 as f32,
        Sizing::Lg => 14 as f32,
        Sizing::Xl => 24 as f32,
    };

    let mut runs = field.glyph_run.iter();
    field
        .content
        .iter()
        .map(|item| {
            let width: f32 = item
                .content
                .iter()
                .map(|_| {
                    let run = runs
                        .next()
                        .expect("text must have a matching glyph run");
                    run.buffer
                        .glyph_positions()
                        .iter()
                        .map(|pos| {
                            pos.x_advance as f32 * font_size / run.units_per_em
                        })
                        .sum::<f32>()
                })
                .sum();
            width
                + if item.style == TextStyle::BulletList {
                    BULLET_INDENT
                } else {
                    0.0
                }
        })
        .fold(0.0, f32::max)
}

pub fn layout(hir: &HIR) -> Result<Page, Diagnostic> {
    let page_type = match hir.config.doc_type {
        DocType::A4 => PageType::A4,
    };
    let orientation = match hir.config.orientation {
        DocOrientation::Portrait => Orientation::Portrait,
    };

    let (page_width, _) = calculate_grid(&page_type, &orientation)?;
    let available_width =
        (page_width - hir.config.left_margin - hir.config.right_margin)
            .max(0.0);

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
                let right_content =
                    wrap_lines(right, available_width - split_line[i]);
                WrappedRow::Split {
                    left: left_content,
                    right: right_content,
                }
            }
        };
        wrapped_rows.push(row_content);
    }

    let mut text_boxes = Vec::new();
    let mut len = 0.0;
    for (i, row) in wrapped_rows.iter().enumerate() {
        match row {
            WrappedRow::Single { content } => {
                let total_height = content.iter().map(|h| h.height).sum();
                text_boxes.push(ContentBox {
                    rows: content.clone(),
                    coord: (0.0, len),
                    width: split_line[i],
                    height: total_height,
                });
                len += total_height;
            }
            WrappedRow::Split { left, right } => {
                let l_total_height = left.iter().map(|h| h.height).sum();
                text_boxes.push(ContentBox {
                    rows: left.clone(),
                    coord: (0.0, len),
                    width: split_line[i],
                    height: l_total_height,
                });
                let r_total_height = right.iter().map(|h| h.height).sum();
                let right_width = available_width - split_line[i];
                text_boxes.push(ContentBox {
                    rows: right.clone(),
                    coord: (split_line[i], len),
                    width: right_width,
                    height: r_total_height,
                });
                len += l_total_height.max(r_total_height);
            }
        }
    }

    page.content = text_boxes;

    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::hir_types::{Sizing, TextOp};

    fn shaped_field(content: Vec<TextOp>) -> AllocatedField {
        let mut rows = vec![ContentRow::Single(AllocatedField {
            content,
            glyph_run: Vec::new(),
            alignment: Alignment::Left,
            size: Sizing::Sm,
        })];
        inscribe::inscribe(&mut rows).unwrap();
        match rows.pop().unwrap() {
            ContentRow::Single(field) => field,
            _ => unreachable!(),
        }
    }

    fn paragraph(content: Vec<TextType>) -> TextOp {
        TextOp {
            content,
            style: TextStyle::Paragraph,
        }
    }

    fn line_text(row: &WrappedRowContents) -> String {
        row.row
            .iter()
            .map(|part| text_content(&part.content))
            .collect()
    }

    fn line_width(row: &WrappedRowContents) -> f32 {
        row.row.iter().map(|part| part.width).sum()
    }

    #[test]
    fn wrapping_uses_the_correct_run_across_paragraphs_and_duplicates() {
        let field = shaped_field(vec![
            paragraph(vec![
                TextType::Text("iiii".into()),
                TextType::Text("iiii".into()),
            ]),
            paragraph(vec![TextType::Bold("WWWW".into())]),
        ]);
        let lines = wrap_lines(&field, 1000.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(line_text(&lines[0]), "iiiiiiii");
        assert_eq!(line_text(&lines[1]), "WWWW");
        let font_size = match field.size {
            Sizing::Sm => 10 as f32,
            Sizing::Md => 12 as f32,
            Sizing::Lg => 14 as f32,
            Sizing::Xl => 24 as f32,
        };
        let expected: f32 = field.glyph_run[2]
            .buffer
            .glyph_positions()
            .iter()
            .map(|pos| {
                pos.x_advance as f32 * font_size
                    / field.glyph_run[2].units_per_em
            })
            .sum();
        assert!((line_width(&lines[1]) - expected).abs() < 0.001);
        assert!(line_width(&lines[1]) > line_width(&lines[0]));
    }

    #[test]
    fn whitespace_breaks_work_across_style_changes_and_preserve_links() {
        let field = shaped_field(vec![paragraph(vec![
            TextType::Text("one ".into()),
            TextType::Bold("two".into()),
            TextType::Link {
                label: " three".into(),
                href: "https://example.com".into(),
            },
        ])]);
        let all = wrap_lines(&field, 1000.0);
        let width = all[0].row[0].width + all[0].row[1].width + 0.01;
        let lines = wrap_lines(&field, width);
        assert_eq!(
            lines.iter().map(line_text).collect::<Vec<_>>(),
            ["one two", "three"]
        );
        assert!(
            matches!(&lines[0].row[1].content, TextType::Bold(s) if s == "two")
        );
        assert!(matches!(&lines[1].row[0].content,
            TextType::Link { label, href } if label == "three" && href == "https://example.com"));
        assert!(lines.iter().all(|line| line_width(line) <= width + 0.001));
    }

    #[test]
    fn long_unicode_words_keep_clusters_and_glyph_text_ranges_intact() {
        let source = "éofficee\u{301}officeé";
        let field =
            shaped_field(vec![paragraph(vec![TextType::Text(source.into())])]);
        let font_size = match field.size {
            Sizing::Sm => 10 as f32,
            Sizing::Md => 12 as f32,
            Sizing::Lg => 14 as f32,
            Sizing::Xl => 24 as f32,
        };
        let clusters = run_clusters(0, source, &field.glyph_run[0], font_size);
        let width = clusters
            .iter()
            .map(|cluster| cluster.width)
            .fold(0.0, f32::max)
            + 0.01;
        let lines = wrap_lines(&field, width);
        assert!(lines.len() > 1);
        assert_eq!(lines.iter().map(line_text).collect::<String>(), source);
        let boundaries: Vec<_> =
            clusters.iter().map(|cluster| cluster.range.end).collect();
        let mut end = 0;
        for line in &lines {
            end += line_text(line).len();
            assert!(boundaries.contains(&end));
            assert!(line_width(line) <= width + 0.001);
            for part in &line.row {
                let text = text_content(&part.content);
                for glyph in &part.glyphs {
                    assert!(text.get(glyph.text_range.clone()).is_some());
                }
            }
        }
    }

    #[test]
    fn empty_lines_newlines_and_zero_width_make_progress() {
        let field = shaped_field(vec![paragraph(vec![TextType::Text(
            "a\n\nb\n".into(),
        )])]);
        let lines = wrap_lines(&field, 1000.0);
        assert_eq!(
            lines.iter().map(line_text).collect::<Vec<_>>(),
            ["a", "", "b", ""]
        );
        let tiny = field.glyph_run[0].buffer.glyph_infos().len();
        assert!(wrap_lines(&field, 0.0).len() <= tiny + 1);
        let empty = shaped_field(vec![paragraph(Vec::new())]);
        assert_eq!(wrap_lines(&empty, 0.0).len(), 1);
    }

    #[test]
    fn bullets_keep_their_indent_on_continuation_lines() {
        let mut item =
            paragraph(vec![TextType::Text("one two three four".into())]);
        item.style = TextStyle::BulletList;
        let field = shaped_field(vec![item]);
        let lines = wrap_lines(&field, 35.0);
        assert!(lines.len() > 1);
        assert_eq!(lines[0].row_style, RowStyle::BulletRow);
        for line in &lines[1..] {
            assert_eq!(line.row_style, RowStyle::TrailingBullet);
        }
        assert!(
            lines
                .iter()
                .all(|line| line_width(line) <= 35.0 - BULLET_INDENT + 0.001)
        );
    }

    #[test]
    fn split_fields_are_shaped_and_wrapped() {
        let mut rows = vec![ContentRow::Split {
            left: shaped_field(vec![paragraph(vec![TextType::Text(
                "left side".into(),
            )])]),
            right: shaped_field(vec![paragraph(vec![TextType::Italic(
                "right side".into(),
            )])]),
        }];
        inscribe::inscribe(&mut rows).unwrap();
        let ContentRow::Split { left, right } = &rows[0] else {
            unreachable!()
        };
        for field in [left, right] {
            assert_eq!(field.glyph_run.len(), 1);
            let lines = wrap_lines(field, 25.0);
            assert!(lines.len() > 1);
            assert!(lines.iter().all(|line| line_width(line) <= 25.001));
        }
    }

    #[test]
    fn layout_and_renderer_use_the_same_point_measurements() {
        let source = "document {\n type: A4\n margin: 240\n}\n\
            elem card { text, date }\n\
            layout card {\n < text | > date\n}\n\
            @card {\n text: Some **bold** and *italic* words with é and a [link](https://example.com).\n date: October 2026\n}";
        let mut parser =
            crate::parser::Parser::new("wrap-test.pyr".into(), source.into())
                .unwrap();
        let ast = parser.parse::<crate::ast::Ast>().unwrap();
        let hir = crate::hir::lower(&ast).unwrap();
        let page = layout(&hir).unwrap();
        assert_eq!(page.content.len(), 2);
        let available =
            595.28 - page.config.left_margin - page.config.right_margin;
        let left = &page.content[0];
        let right = &page.content[1];
        assert!((left.width + right.width - available).abs() < 0.001);
        assert!((right.coord.0 - left.width).abs() < 0.001);
        assert!(left.rows.len() > 1);
        for content in &page.content {
            assert!(
                content
                    .rows
                    .iter()
                    .all(|row| line_width(row) <= content.width + 0.001)
            );
        }
        let output = "target/wrapping-smoke.pdf";
        crate::render::Renderer::render_pdf(&page, output);
        let pdf = std::fs::read(output).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn crlf_is_a_single_line_break() {
        let field = shaped_field(vec![paragraph(vec![TextType::Text(
            "a\r\nb".into(),
        )])]);
        let lines = wrap_lines(&field, 1000.0);
        assert_eq!(lines.iter().map(line_text).collect::<Vec<_>>(), ["a", "b"]);
    }
}
