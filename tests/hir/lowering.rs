use pyrus::ast::Ast;
use pyrus::hir::{
    hir_types::{HIRModule, Literal, Op, Type},
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
// Basic Lowering Tests
// ============================================================================

#[test]
fn test_lower_empty_document() {
    let source = r#"
document {
}
"#;
    let tokens = lex(source, "test_lower_empty_document").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Should have the implicit __document function
    assert_eq!(
        hlir.functions.len(),
        1,
        "Should have one function (__document)"
    );
    assert!(hlir.functions.values().any(|f| f.name == "__document"));
    assert!(hlir.globals.is_empty());
    assert!(hlir.elements.is_empty());
}

#[test]
fn test_lower_global_const() {
    let source = r#"
template {
    const PI = 3.14
}
document {
}
"#;
    let tokens = lex(source, "test_lower_global_const").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert!(hlir.globals.values().any(|g| g.name == "PI"));
}

#[test]
fn test_lower_global_var() {
    let source = r#"
template {
    let counter = 0
}
document {
}
"#;
    let tokens = lex(source, "test_lower_global_var").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert!(hlir.globals.values().any(|g| g.name == "counter"));
}

#[test]
fn test_lower_multiple_globals() {
    let source = r#"
template {
    const TITLE = "My Doc"
    const AUTHOR = "Me"
    let page_num = 1
}
document {
}
"#;
    let tokens = lex(source, "test_lower_multiple_globals").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    for name in ["TITLE", "AUTHOR", "page_num"] {
        assert!(
            hlir.globals.values().any(|g| g.name == name),
            "Should have global {name}"
        );
    }
}

#[test]
fn test_lower_simple_function() {
    let source = r#"
template {
    func greeting() {
        return @text[Hello]
    }
}
document {
}
"#;
    let tokens = lex(source, "test_lower_simple_function").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Should have the lowered template function + the implicit __document function
    assert_eq!(hlir.functions.len(), 2);
    assert!(hlir.functions.values().any(|f| f.name == "greeting"));

    let greeting = hlir
        .functions
        .values()
        .find(|f| f.name == "greeting")
        .unwrap();
    assert_eq!(greeting.args.len(), 0);
}

#[test]
fn test_lower_function_with_args() {
    let source = r#"
template {
    func section_with_title(title: String) {
        return @section[
            @text[title]
        ]
    }
}
document {
}
"#;
    let tokens = lex(source, "test_lower_function_with_args").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let elem = hlir
        .functions
        .values()
        .find(|f| f.name == "section_with_title")
        .unwrap();
    assert_eq!(elem.args.len(), 1);
    assert_eq!(elem.args[0], Type::String);
}

#[test]
fn test_lower_function_with_multiple_args() {
    let source = r#"
template {
    func formatted_number(value: Int, prefix: String) {
        return @text[prefix]
    }
}
document {
}
"#;
    let tokens = lex(source, "test_lower_function_with_multiple_args").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let elem = hlir
        .functions
        .values()
        .find(|f| f.name == "formatted_number")
        .unwrap();
    assert_eq!(elem.args.len(), 2);
    assert_eq!(elem.args[0], Type::Int);
    assert_eq!(elem.args[1], Type::String);
}

// ============================================================================
// Op Sequence Tests
// ============================================================================

#[test]
fn test_lower_generates_doc_element_emit_ops() {
    let source = r#"
document {
    @text[First]
    @text[Second]
}
"#;
    let tokens = lex(source, "test_lower_generates_doc_element_emit_ops").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let doc_func = hlir
        .functions
        .values()
        .find(|f| f.name == "__document")
        .unwrap();

    let emit_ops: Vec<_> = doc_func
        .body
        .items
        .iter()
        .filter(|op| matches!(op, Op::HirElementEmit { .. }))
        .collect();

    assert_eq!(emit_ops.len(), 2, "Should have 2 DocElementEmit ops");
}

#[test]
fn test_lower_const_generates_const_op() {
    let source = r#"
template {
    const VALUE = 42
}
document {
}
"#;
    let tokens = lex(source, "test_lower_const_generates_const_op").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // The global should have been created with the right value
    let global = hlir
        .globals
        .values()
        .find(|g| g.name == "VALUE")
        .expect("Should have VALUE global");

    // Check the init literal
    match &global.literal {
        Literal::Int(42) => {}
        _ => panic!("Expected Int(42), got {:?}", global.literal),
    }
}
