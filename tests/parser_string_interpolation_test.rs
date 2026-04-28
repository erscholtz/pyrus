mod common;

use common::template_statements;
use pyrus::ast::{ExprKind, StmtKind};

fn assigned_expr(source: &str) -> ExprKind {
    let statements = template_statements(source);
    match &statements[0].node {
        StmtKind::VarAssign(stmt) => stmt.value.node.clone(),
        other => panic!("Expected VarAssign statement, got {other:?}"),
    }
}

#[test]
fn test_parse_string_interpolation_simple() {
    match assigned_expr(r#"template { let msg = "Hello, ${name}!" }"#) {
        ExprKind::InterpolatedString(expr) => {
            assert_eq!(expr.parts.len(), 3);
            assert!(
                matches!(expr.parts[0], ExprKind::StringLiteral(ref text) if text == "Hello, ")
            );
            assert!(matches!(expr.parts[1], ExprKind::Identifier(ref name) if name == "name"));
            assert!(matches!(expr.parts[2], ExprKind::StringLiteral(ref text) if text == "!"));
        }
        other => panic!("Expected InterpolatedString expr, got {other:?}"),
    }
}

#[test]
fn test_parse_string_interpolation_multiple() {
    match assigned_expr(r#"template { let msg = "${greeting}, ${name}!" }"#) {
        ExprKind::InterpolatedString(expr) => {
            assert_eq!(expr.parts.len(), 4);
            assert!(matches!(expr.parts[0], ExprKind::Identifier(ref name) if name == "greeting"));
            assert!(matches!(expr.parts[1], ExprKind::StringLiteral(ref text) if text == ", "));
            assert!(matches!(expr.parts[2], ExprKind::Identifier(ref name) if name == "name"));
            assert!(matches!(expr.parts[3], ExprKind::StringLiteral(ref text) if text == "!"));
        }
        other => panic!("Expected InterpolatedString expr, got {other:?}"),
    }
}

#[test]
fn test_parse_string_interpolation_with_number() {
    match assigned_expr(r#"template { let msg = "Count: ${count}" }"#) {
        ExprKind::InterpolatedString(expr) => {
            assert_eq!(expr.parts.len(), 2);
            assert!(
                matches!(expr.parts[0], ExprKind::StringLiteral(ref text) if text == "Count: ")
            );
            assert!(matches!(expr.parts[1], ExprKind::Identifier(ref name) if name == "count"));
        }
        other => panic!("Expected InterpolatedString expr, got {other:?}"),
    }
}

#[test]
fn test_parse_string_without_interpolation() {
    match assigned_expr(r#"template { let msg = "Hello, World!" }"#) {
        ExprKind::StringLiteral(value) => assert_eq!(value, "Hello, World!"),
        other => panic!("Expected StringLiteral expr, got {other:?}"),
    }
}

#[test]
fn test_parse_string_with_literal_braces() {
    match assigned_expr(r#"template { let msg = "Use {brackets} freely" }"#) {
        ExprKind::StringLiteral(value) => assert_eq!(value, "Use {brackets} freely"),
        other => panic!("Expected StringLiteral expr, got {other:?}"),
    }
}

#[test]
fn test_parse_double_dollar_preserved() {
    match assigned_expr(r#"template { let msg = "Price: $$100" }"#) {
        ExprKind::StringLiteral(value) => assert_eq!(value, "Price: $$100"),
        other => panic!("Expected StringLiteral expr, got {other:?}"),
    }
}
