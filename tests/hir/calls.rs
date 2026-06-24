use pyrus::ast::Ast;
use pyrus::hir::{
    hir_types::{HIRModule, Op},
    lower,
};
use pyrus::lexer::{TokenStream, lex_all};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {:?}", errors))
}

fn text_contents(hlir: &HIRModule) -> Vec<&str> {
    hlir.elements
        .iter()
        .filter_map(|element| match element {
            pyrus::hir::hir_types::HirElementOp::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn test_template_call_substitutes_text_arguments() {
    let source = r#"
template {
    func header(name: String, linkedin: String) {
        return @section[
            @text[${name}]
            @text[linkedin.com/in/${linkedin}]
        ]
    }
}
document {
    @header("Erik Scholtz", "erikscholtz")
}
"#;
    let tokens =
        lex_all(source, "test_template_call_substitutes_text_arguments").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let contents = text_contents(&hlir);

    assert!(contents.contains(&"Erik Scholtz"));
    assert!(contents.contains(&"linkedin.com/in/erikscholtz"));
}

#[test]
fn test_each_template_call_gets_its_own_substitutions() {
    let source = r#"
template {
    func item(name: String) {
        return @text[${name}]
    }
}
document {
    @item("First")
    @item("Second")
}
"#;
    let tokens = lex_all(source, "test_each_template_call_gets_its_own_substitutions")
        .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let contents = text_contents(&hlir);

    assert!(contents.contains(&"First"));
    assert!(contents.contains(&"Second"));
}
// Function Call Lowering Tests
// ============================================================================

#[test]
fn test_lower_function_call_in_document() {
    let source = r#"
template {
    func header() {
        return @text[Header]
    }
}
document {
    @header()
}
"#;
    let tokens = lex_all(source, "test_lower_function_call_in_document").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Find __document function body
    let doc_func = hlir
        .functions
        .values()
        .find(|f| f.name == "__document")
        .expect("Should have __document function");

    // Should have an ElementCall operation (element calls generate ElementCall, not FuncCall)
    let has_call = doc_func
        .body
        .items
        .iter()
        .any(|op| matches!(op, Op::ElementCall { .. }));
    assert!(has_call, "Should generate ElementCall op for element call");
}

#[test]
fn test_lower_function_call_with_args() {
    let source = r#"
template {
    func greet(name: String) {
        return @text[name]
    }
}
document {
    @greet("World")
}
"#;
    let tokens = lex_all(source, "test_lower_function_call_with_args").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let doc_func = hlir
        .functions
        .values()
        .find(|f| f.name == "__document")
        .unwrap();

    // Find the call operation and check it has args (element calls use ElementCall)
    let call_op = doc_func.body.items.iter().find_map(|op| match op {
        Op::ElementCall { element, args, .. } => Some((element, args)),
        _ => None,
    });

    assert!(call_op.is_some(), "Should have ElementCall op");
    let (_, args) = call_op.unwrap();
    assert_eq!(args.len(), 1, "Call should have 1 argument");
}

// ============================================================================
