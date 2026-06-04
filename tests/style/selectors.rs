use pyrus::ast::Ast;
use pyrus::hir::{hir_types::HIRModule, lower};
use pyrus::lexer::{TokenStream, lex_all};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
}
// Basic Selector Tests
// ============================================================================

#[test]
fn test_id_selector() {
    let source = r#"
document {
    @text(id="header")[Header]
}
style {
    #header {
        font-size: 24pt;
        color: blue;
    }
}
"#;
    let tokens = lex_all(source, "test_id_selector").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert_eq!(hlir.element_metadata.len(), 1);
    assert_eq!(hlir.css_rules.len(), 1);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // Check computed styles
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"24pt".to_string())
    );
    assert_eq!(node.computed.style.get("color"), Some(&"blue".to_string()));
}

#[test]
fn test_class_selector() {
    let source = r#"
document {
    @text(class="intro")[Introduction]
}
style {
    .intro {
        font-weight: bold;
    }
}
"#;
    let tokens = lex_all(source, "test_class_selector").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    assert_eq!(
        node.computed.style.get("font-weight"),
        Some(&"bold".to_string())
    );
}

#[test]
fn test_selector_applies_to_element() {
    let source = r#"
document {
    @text(class="body")[Some text]
}
style {
    .body {
        font-size: 12;
    }
}
"#;
    let tokens = lex_all(source, "test_selector_applies_to_element").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"12".to_string())
    );
}

// ============================================================================
