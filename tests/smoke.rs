use std::fs;
use std::path::PathBuf;

use pyrus::ast::Ast;
use pyrus::hir::lower;
use pyrus::layout::{Page, RowStyle, layout, text_content};
use pyrus::parser::Parser;
use pyrus::render::Renderer;

fn compile_pdf(name: &str, source: &str) -> (Page, Vec<u8>) {
    let mut parser = Parser::new(format!("{name}.pyr"), source.into())
        .expect("source should lex");
    let ast = parser.parse::<Ast>().expect("document should parse");
    let hir = lower(&ast).expect("document should lower");
    let page = layout(&hir).expect("document should lay out");

    let output_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/smoke-tests");
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join(format!("{name}-{}.pdf", std::process::id()));
    Renderer::render_pdf(&page, output.to_str().unwrap());
    let pdf = fs::read(&output).expect("renderer should write a PDF");
    assert!(pdf.starts_with(b"%PDF-"), "missing PDF header");
    assert!(pdf.ends_with(b"%%EOF"), "missing PDF trailer");
    fs::remove_file(output).unwrap();
    (page, pdf)
}

#[test]
fn empty_document_renders_to_pdf() {
    let (page, _) = compile_pdf("empty", "document {}\n");
    assert!(page.content.is_empty());
}

#[test]
fn styled_document_renders_to_pdf() {
    let (page, pdf) = compile_pdf(
        "styled-document",
        include_str!("input/styled-document.pyr"),
    );
    assert_eq!(page.content.len(), 5);
    let text: Vec<String> = page
        .content
        .iter()
        .map(|content| {
            content
                .rows
                .iter()
                .flat_map(|row| &row.row)
                .map(|part| text_content(&part.content))
                .collect()
        })
        .collect();
    assert_eq!(text[0], "Pyrus résumé");
    assert_eq!(text[1], "Document compiler");
    assert_eq!(text[2], "October 2026");
    assert!(text[3].contains("Built with Rust"));
    assert!(text[3].contains("Read the documentation"));
    assert_eq!(text[4], "Second entry");
    assert_eq!(
        page.content[3]
            .rows
            .iter()
            .filter(|row| row.row_style == RowStyle::BulletRow)
            .count(),
        2
    );
    assert!(page.content[4].coord.1 > page.content[3].coord.1);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(
        pdf_text.contains("/URI"),
        "PDF should contain a link action"
    );
    assert!(pdf_text.contains("https://example.com/docs"));
}

#[test]
fn wrapped_split_row_renders_to_pdf() {
    let source = "document {\n doc_type: A4\n left: 240\n right: 240\n}\n\
        elem card { text, date }\n\
        layout card {\n < text | > date\n text: sm\n date: sm\n}\n\
        @card {\n text: Some **bold** and *italic* words with é and a [link](https://example.com).\n date: October 2026\n}";
    let (page, _) = compile_pdf("wrapped-split-row", source);
    assert_eq!(page.content.len(), 2);
    assert_eq!(page.config.left_margin, 240.0);
    assert_eq!(page.config.right_margin, 240.0);
    let available = 595.28 - page.config.left_margin - page.config.right_margin;
    let left = &page.content[0];
    let right = &page.content[1];
    assert_eq!(left.coord, (0.0, 0.0));
    assert_eq!(right.coord.1, left.coord.1);
    assert!(right.coord.0 > 0.0 && right.coord.0 < available);
    assert!(left.rows.len() > 1);
    let widths = [right.coord.0, available - right.coord.0];
    for (content, width) in page.content.iter().zip(widths) {
        for row in &content.rows {
            let line_width: f32 = row.row.iter().map(|part| part.width).sum();
            assert!(line_width <= width + 0.001);
        }
    }
}
