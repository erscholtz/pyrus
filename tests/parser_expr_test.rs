mod common;

use common::template_statements;
use pyrus::ast::{BinOp, ExprKind, StmtKind, UnaryOp};
use pyrus::lexer::lex;

fn assigned_expr(source: &str) -> ExprKind {
    let statements = template_statements(source);
    match &statements[0].node {
        StmtKind::VarAssign(stmt) => stmt.value.node.clone(),
        other => panic!("Expected VarAssign statement, got {other:?}"),
    }
}

#[test]
fn test_parse_unary_negation() {
    match assigned_expr("template { let x = -42 }") {
        ExprKind::Unary(expr) => {
            assert!(matches!(expr.op, UnaryOp::Negate));
            assert!(matches!(expr.expr.as_ref(), ExprKind::Int(42)));
        }
        other => panic!("Expected Unary expr, got {other:?}"),
    }
}

#[test]
fn test_parse_binary_addition() {
    match assigned_expr("template { let sum = x + y }") {
        ExprKind::Binary(expr) => {
            assert!(matches!(expr.op, BinOp::Add));
            assert!(matches!(expr.left.as_ref(), ExprKind::Identifier(name) if name == "x"));
            assert!(matches!(expr.right.as_ref(), ExprKind::Identifier(name) if name == "y"));
        }
        other => panic!("Expected Binary expr, got {other:?}"),
    }
}

#[test]
fn test_parse_binary_subtraction() {
    match assigned_expr("template { let diff = a - b }") {
        ExprKind::Binary(expr) => assert!(matches!(expr.op, BinOp::Subtract)),
        other => panic!("Expected Binary expr, got {other:?}"),
    }
}

#[test]
fn test_parse_binary_multiplication() {
    match assigned_expr("template { let product = a * b }") {
        ExprKind::Binary(expr) => assert!(matches!(expr.op, BinOp::Multiply)),
        other => panic!("Expected Binary expr, got {other:?}"),
    }
}

#[test]
fn test_parse_binary_division() {
    match assigned_expr("template { let quotient = a / b }") {
        ExprKind::Binary(expr) => assert!(matches!(expr.op, BinOp::Divide)),
        other => panic!("Expected Binary expr, got {other:?}"),
    }
}

#[test]
fn test_parse_binary_equals() {
    match assigned_expr("template { let result = a = b }") {
        ExprKind::Binary(expr) => assert!(matches!(expr.op, BinOp::Equals)),
        other => panic!("Expected Binary expr, got {other:?}"),
    }
}

#[test]
fn test_parse_string_literal() {
    match assigned_expr(r#"template { let msg = "Hello, World!" }"#) {
        ExprKind::StringLiteral(value) => assert_eq!(value, "Hello, World!"),
        other => panic!("Expected StringLiteral expr, got {other:?}"),
    }
}

#[test]
fn test_parse_string_with_escaped_quote() {
    match assigned_expr(r#"template { let msg = "foo\"bar" }"#) {
        ExprKind::StringLiteral(value) => assert_eq!(value, "foo\\\"bar"),
        other => panic!("Expected StringLiteral expr, got {other:?}"),
    }
}

#[test]
fn test_lex_unterminated_string() {
    let tokens = lex(r#"template { let msg = "unterminated }"#, "test.ink").expect("Lexing failed");
    assert!(
        !tokens.errors.is_empty(),
        "Should report an unterminated string"
    );
    assert_eq!(tokens.errors[0].message, "Unterminated string literal");
}

#[test]
fn test_parse_integer_literal() {
    match assigned_expr("template { let num = 42 }") {
        ExprKind::Int(value) => assert_eq!(value, 42),
        other => panic!("Expected Int expr, got {other:?}"),
    }
}

#[test]
fn test_parse_float_literal() {
    match assigned_expr("template { let pi = 3.14 }") {
        ExprKind::Float(value) => assert!((value - 3.14).abs() < 0.001),
        other => panic!("Expected Float expr, got {other:?}"),
    }
}
