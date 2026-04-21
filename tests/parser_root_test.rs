mod common;

use common::{parse_ast, parse_errors};
use pyrus::diagnostic::Diagnostic;

#[test]
fn test_parse_empty_document() {
    let ast = parse_ast("document { }");
    assert!(ast.document.is_some());
    assert!(ast.template.is_none());
    assert!(ast.style.is_none());

    let doc = ast.document.expect("Expected document block");
    assert_eq!(doc.elements.len(), 0);
}

#[test]
fn test_parse_empty_template() {
    let ast = parse_ast("template { }");
    assert!(ast.template.is_some());
    assert!(ast.document.is_none());
    assert!(ast.style.is_none());

    let template = ast.template.expect("Expected template block");
    assert_eq!(template.statements.len(), 0);
}

#[test]
fn test_parse_empty_style() {
    let ast = parse_ast("style { }");
    assert!(ast.style.is_some());
    assert!(ast.template.is_none());
    assert!(ast.document.is_none());

    let style = ast.style.expect("Expected style block");
    assert_eq!(style.statements.len(), 0);
}

#[test]
fn test_parse_all_blocks() {
    let ast = parse_ast("template { } document { } style { }");
    assert!(ast.template.is_some());
    assert!(ast.document.is_some());
    assert!(ast.style.is_some());
}

#[test]
fn test_duplicate_template_block_reports_error() {
    let errors = parse_errors("template { } template { }");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message(), "duplicate template block");
}
