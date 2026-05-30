use pyrus::ast::Ast;
use pyrus::hir::{hir_types::HIRModule, lower};
use pyrus::lexer::{TokenStream, lex};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
}
// Inheritance Tests
// ============================================================================

#[test]
fn test_style_inheritance() {
    let source = r#"
document {
    @section(class="container")[
        @text[Nested text]
    ]
}
style {
    .container {
        color: blue;
    }
}
"#;
    let tokens = lex(source, "test_style_inheritance").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Find the nested text element (should be index 1, section is index 0)
    let text_metadata = hlir
        .element_metadata
        .iter()
        .find(|m| m.element_type == "text")
        .expect("Should have a text element");
    let node = hlir
        .attributes
        .find_node(text_metadata.attributes_ref)
        .unwrap();

    // The text element should inherit color from its parent section
    assert_eq!(
        node.computed.style.get("color"),
        Some(&"blue".to_string()),
        "Text should inherit color from section"
    );
}

#[test]
fn test_non_inherited_properties() {
    let source = r#"
document {
    @section(class="container")[
        @text[Nested text]
    ]
}
style {
    .container {
        margin: 20pt;
    }
}
"#;
    let tokens = lex(source, "test_non_inherited_properties").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let section_metadata = hlir
        .element_metadata
        .iter()
        .find(|m| m.element_type == "section")
        .unwrap();
    let section_node = hlir
        .attributes
        .find_node(section_metadata.attributes_ref)
        .unwrap();

    // Section should have margin
    assert_eq!(section_node.computed.margin, Some(20.0));

    let text_metadata = hlir
        .element_metadata
        .iter()
        .find(|m| m.element_type == "text")
        .unwrap();
    let text_node = hlir
        .attributes
        .find_node(text_metadata.attributes_ref)
        .unwrap();

    // Text should NOT inherit margin (it's not an inherited property)
    assert_eq!(text_node.computed.margin, None);
}

// ============================================================================
// Multiple Classes Tests
// ============================================================================

#[test]
fn test_multiple_classes_on_element() {
    let source = r#"
document {
    @text(class="large bold")[Text]
}
style {
    .large {
        font-size: 24;
    }
    .bold {
        font-weight: bold;
    }
}
"#;
    let tokens = lex(source, "test_multiple_classes_on_element").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    assert_eq!(metadata.classes, vec!["large", "bold"]);

    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // Both class rules should apply
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"24".to_string())
    );
    assert_eq!(
        node.computed.style.get("font-weight"),
        Some(&"bold".to_string())
    );
}

// ============================================================================
