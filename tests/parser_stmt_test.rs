mod common;

use common::template_statements;
use pyrus::ast::{DocElemKind, ExprKind, ReturnStmt, StmtKind};

#[test]
fn test_parse_variable_assignment() {
    let statements = template_statements(r#"template { let x = "hello" }"#);
    assert_eq!(statements.len(), 1);

    match &statements[0].node {
        StmtKind::VarAssign(stmt) => {
            assert_eq!(stmt.name, "x");
            match &stmt.value.node {
                ExprKind::StringLiteral(value) => assert_eq!(value, "hello"),
                other => panic!("Expected StringLiteral expr, got {other:?}"),
            }
        }
        other => panic!("Expected VarAssign statement, got {other:?}"),
    }
}

#[test]
fn test_parse_const_assignment() {
    let statements = template_statements(r#"template { const PI = "3.14" }"#);
    assert_eq!(statements.len(), 1);

    match &statements[0].node {
        StmtKind::ConstAssign(stmt) => {
            assert_eq!(stmt.name, "PI");
            match &stmt.value.node {
                ExprKind::StringLiteral(value) => assert_eq!(value, "3.14"),
                other => panic!("Expected StringLiteral expr, got {other:?}"),
            }
        }
        other => panic!("Expected ConstAssign statement, got {other:?}"),
    }
}

#[test]
fn test_parse_default_set() {
    let statements = template_statements("template { width = 100 }");
    assert_eq!(statements.len(), 1);

    match &statements[0].node {
        StmtKind::DefaultSet(stmt) => {
            assert_eq!(stmt.key, "width");
            match &stmt.value.node {
                ExprKind::Int(value) => assert_eq!(*value, 100),
                other => panic!("Expected Int expr, got {other:?}"),
            }
        }
        other => panic!("Expected DefaultSet statement, got {other:?}"),
    }
}

#[test]
fn test_parse_return_doc_element() {
    let statements = template_statements("template { return @text[done] }");
    assert_eq!(statements.len(), 1);

    match &statements[0].node {
        StmtKind::Return(ReturnStmt::DocElem(doc_elem)) => match &doc_elem.node {
            DocElemKind::Text(text) => match &text.content.node {
                ExprKind::StringLiteral(value) => assert_eq!(value, "done"),
                other => panic!("Expected StringLiteral content, got {other:?}"),
            },
            other => panic!("Expected text element, got {other:?}"),
        },
        other => panic!("Expected return doc element, got {other:?}"),
    }
}

#[test]
fn test_parse_children_statement() {
    let statements = template_statements("template { children }");
    assert_eq!(statements.len(), 1);

    match &statements[0].node {
        StmtKind::Children(stmt) => assert!(stmt.children),
        other => panic!("Expected children statement, got {other:?}"),
    }
}

#[test]
fn test_parse_mixed_statements() {
    let statements = template_statements("template { let x = 10 const MAX = 100 width = 50 }");
    assert_eq!(statements.len(), 3);

    assert!(matches!(statements[0].node, StmtKind::VarAssign(_)));
    assert!(matches!(statements[1].node, StmtKind::ConstAssign(_)));
    assert!(matches!(statements[2].node, StmtKind::DefaultSet(_)));
}
