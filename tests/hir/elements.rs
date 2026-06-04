use pyrus::ast::Ast;
use pyrus::hir::{
    hir_types::{HIRModule, HirElementOp},
    lower,
};
use pyrus::lexer::{TokenStream, lex_all};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
}
// Document Element Lowering Tests
// ============================================================================

#[test]
fn test_lower_text_element() {
    let source = r#"
document {
    @text[Hello World]
}
"#;
    let tokens = lex_all(source, "test_lower_text_element").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert_eq!(hlir.elements.len(), 1);
    assert_eq!(hlir.element_metadata.len(), 1);
    assert_eq!(hlir.element_metadata[0].element_type, "text");
}

#[test]
fn test_lower_section_with_children() {
    let source = r#"
document {
    @section[
        @text[Child 1]
        @text[Child 2]
    ]
}
"#;
    let tokens = lex_all(source, "test_lower_section_with_children").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Should have: 1 section + 2 text elements
    assert_eq!(hlir.elements.len(), 3);

    // Find section metadata
    let _section_meta = hlir
        .element_metadata
        .iter()
        .find(|m| m.element_type == "section")
        .expect("Should have section metadata");

    // Children should point to section as parent
    let children: Vec<_> = hlir
        .element_metadata
        .iter()
        .filter(|m| m.parent == Some(0)) // section is index 0
        .collect();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_lower_element_with_id_and_class() {
    let source = r#"
document {
    @text(id="header", class="large bold")[Title]
}
"#;
    let tokens = lex_all(source, "test_lower_element_with_id_and_class").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let meta = &hlir.element_metadata[0];
    assert_eq!(meta.id, Some("header".to_string()));
    assert_eq!(meta.classes, vec!["large", "bold"]);
}

#[test]
fn test_lower_separator_element() {
    let source = r#"
document {
    @separator(class="rule")
}
"#;
    let tokens = lex_all(source, "test_lower_separator_element").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert_eq!(hlir.elements.len(), 1);
    assert_eq!(hlir.element_metadata[0].element_type, "separator");
    assert_eq!(hlir.element_metadata[0].classes, vec!["rule"]);
}

#[test]
fn test_lower_link_element_preserves_href() {
    let source = r#"
document {
    @link(class="external")["https://example.com", "Example"]
}
"#;
    let tokens = lex_all(source, "test_lower_link_element_preserves_href").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert_eq!(hlir.elements.len(), 1);
    assert_eq!(hlir.element_metadata[0].element_type, "link");
    assert_eq!(hlir.element_metadata[0].classes, vec!["external"]);

    match &hlir.elements[0] {
        HirElementOp::Link { href, content, .. } => {
            assert_eq!(href, "https://example.com");
            assert_eq!(content, "Example");
        }
        other => panic!("Expected link element, got {other:?}"),
    }
}

// ============================================================================
// CSS Style Integration Tests
// ============================================================================

#[test]
fn test_lower_preserves_css_rules() {
    let source = r#"
document {
    @text(class="content")[Content]
}
style {
    .content {
        font-size: 14pt;
    }
    .highlight {
        color: red;
    }
}
"#;
    let tokens = lex_all(source, "test_lower_preserves_css_rules").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert_eq!(hlir.css_rules.len(), 2);
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_lower_preserves_element_order() {
    let source = r#"
document {
    @text[First]
    @section[
        @text[Nested]
    ]
    @text[Last]
}
"#;
    let tokens = lex_all(source, "test_lower_preserves_element_order").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Elements should be in document order
    assert_eq!(hlir.element_metadata[0].element_type, "text");
    assert_eq!(hlir.element_metadata[1].element_type, "section");
    assert_eq!(hlir.element_metadata[2].element_type, "text");
    assert_eq!(hlir.element_metadata[3].element_type, "text"); // Nested
}

#[test]
fn test_lower_empty_template_is_ok() {
    let source = r#"
template {
}
document {
}
"#;
    let tokens = lex_all(source, "test_lower_empty_template_is_ok").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert!(hlir.globals.is_empty());
    assert_eq!(hlir.functions.len(), 1); // Just __document
}

#[test]
fn test_lower_nested_sections() {
    let source = r#"
document {
    @section[
        @section[
            @text[Deeply nested]
        ]
    ]
}
"#;
    let tokens = lex_all(source, "test_lower_nested_sections").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Should have: outer section (0), inner section (1), text (2)
    assert_eq!(hlir.elements.len(), 3);

    // Inner section's parent should be outer section
    assert_eq!(hlir.element_metadata[1].parent, Some(0));
    // Text's parent should be inner section
    assert_eq!(hlir.element_metadata[2].parent, Some(1));
}
