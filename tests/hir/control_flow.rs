use pyrus::ast::Ast;
use pyrus::hir::{
    hir_types::{HIRModule, Op},
    lower,
};
use pyrus::lexer::{TokenStream, lex};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
}

fn function_ops<'a>(hlir: &'a HIRModule, name: &str) -> &'a [Op] {
    hlir.functions
        .values()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("Should have function {name}"))
        .body
        .items
        .as_slice()
}
// If Statement Lowering Tests
// ============================================================================

#[test]
fn test_lower_if_statement_emits_if_op() {
    let source = r#"
template {
    func choose(show: String) {
        if show {
            let y = 1
        }
    }
}
document {
}
"#;
    let tokens = lex(source, "test_lower_if_statement_emits_if_op").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let ops = function_ops(&hlir, "choose");

    assert!(
        ops.iter()
            .any(|op| matches!(op, Op::Const { name, .. } if name == "__cond")),
        "Should emit a local condition op"
    );

    let if_op = ops
        .iter()
        .find_map(|op| match op {
            Op::If { then, else_, .. } => Some((then, else_)),
            _ => None,
        })
        .expect("Should emit an If op");

    assert!(!if_op.0.items.is_empty(), "Then block should not be empty");
    assert!(
        if_op.1.is_none(),
        "If without else should not emit else block"
    );
}

#[test]
fn test_lower_if_else_statement_emits_else_block() {
    let source = r#"
template {
    func choose(show: String) {
        if show {
            let y = 1
        } else {
            let y = 2
        }
    }
}
document {
}
"#;
    let tokens =
        lex(source, "test_lower_if_else_statement_emits_else_block").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let ops = function_ops(&hlir, "choose");

    let if_op = ops
        .iter()
        .find_map(|op| match op {
            Op::If { then, else_, .. } => Some((then, else_)),
            _ => None,
        })
        .expect("Should emit an If op");

    assert!(!if_op.0.items.is_empty(), "Then block should not be empty");

    let else_block = if_op.1.as_ref().expect("Should emit an else block");
    assert!(
        !else_block.items.is_empty(),
        "Else block should not be empty"
    );
}

#[test]
fn test_lower_if_statement_preserves_multiple_then_ops() {
    let source = r#"
template {
    func choose(show: String) {
        if show {
            let a = 1
            const B = 2
        }
    }
}
document {
}
"#;
    let tokens = lex(
        source,
        "test_lower_if_statement_preserves_multiple_then_ops",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let ops = function_ops(&hlir, "choose");

    let then = ops
        .iter()
        .find_map(|op| match op {
            Op::If { then, .. } => Some(then),
            _ => None,
        })
        .expect("Should emit an If op");

    assert_eq!(
        then.items.len(),
        2,
        "Then block should preserve both lowered statements"
    );
}

// ============================================================================
