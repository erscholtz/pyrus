mod common;

use common::template_statements;
use pyrus::ast::{ExprKind, ReturnStmt, StmtKind, Type};

#[test]
fn test_parse_function_declaration_with_parameter() {
    let statements = template_statements("template { func greet(name: String) { return 42 } }");
    assert_eq!(statements.len(), 1);

    match &statements[0].node {
        StmtKind::FuncDecl(func) => {
            assert_eq!(func.name, "greet");
            assert_eq!(func.args.len(), 1);
            assert!(matches!(func.args[0].ty, Type::String));
            assert!(
                matches!(func.args[0].value.node, ExprKind::Identifier(ref name) if name == "name")
            );
            assert_eq!(func.body.len(), 1);
            assert!(matches!(
                func.body[0].node,
                StmtKind::Return(ReturnStmt::Expr(ref expr)) if matches!(expr.node, ExprKind::Int(42))
            ));
        }
        other => panic!("Expected FuncDecl statement, got {other:?}"),
    }
}

#[test]
fn test_parse_function_declaration_with_doc_element_return() {
    let statements = template_statements("template { func render() { return @text[Hello] } }");
    assert_eq!(statements.len(), 1);

    match &statements[0].node {
        StmtKind::FuncDecl(func) => {
            assert_eq!(func.name, "render");
            assert!(func.args.is_empty());
            assert_eq!(func.body.len(), 1);
            assert!(matches!(
                func.body[0].node,
                StmtKind::Return(ReturnStmt::DocElem(_))
            ));
        }
        other => panic!("Expected FuncDecl statement, got {other:?}"),
    }
}
