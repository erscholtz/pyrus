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
// Typed Properties Tests (margin, padding, etc.)
// ============================================================================

#[test]
fn test_typed_margin_property() {
    let source = r#"
document {
    @text(class="box")[Text]
}
style {
    .box {
        margin: 15pt;
    }
}
"#;
    let tokens = lex(source, "test_typed_margin_property").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // margin should be parsed as f32
    assert_eq!(node.computed.margin, Some(15.0));
}

#[test]
fn test_typed_padding_property() {
    let source = r#"
document {
    @text(class="box")[Text]
}
style {
    .box {
        padding: 10pt;
    }
}
"#;
    let tokens = lex(source, "test_typed_padding_property").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    assert_eq!(node.computed.padding, Some(10.0));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_no_matching_rules() {
    let source = r#"
document {
    @text(id="mytext")[Text]
}
style {
    .nomatch {
        font-size: 24;
    }
}
"#;
    let tokens = lex(source, "test_no_matching_rules").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // No styles should be applied
    assert!(node.computed.style.is_empty());
    assert_eq!(node.computed.margin, None);
}

#[test]
fn test_empty_style_block() {
    let source = r#"
document {
    @text[Text]
}
style {
}
"#;
    let tokens = lex(source, "test_empty_style_block").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // No styles should be applied
    assert!(node.computed.style.is_empty());
}

#[test]
fn test_no_style_block() {
    let source = r#"
document {
    @text[Text]
}
"#;
    let tokens = lex(source, "test_no_style_block").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert!(hlir.css_rules.is_empty());

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // No styles should be applied
    assert!(node.computed.style.is_empty());
}

// ============================================================================
// Complex Integration Test
// ============================================================================

#[test]
fn test_complex_css_scenario() {
    let source = r#"
document {
    @text(id="header", class="title")[Header]
    @section(class="content")[
        @text(class="body")[Body text]
    ]
    @text(class="footer")[Footer]
}
style {
    #header {
        font-size: 32pt;
        color: blue;
    }
    .title {
        font-weight: bold;
    }
    .content {
        margin: 20pt;
        padding: 10pt;
        border: "1px solid";
    }
    .body {
        font-size: 14pt;
    }
    .footer {
        font-size: 10pt;
        color: gray;
    }
}
"#;
    let tokens = lex(source, "test_complex_css_scenario").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Header: id + class
    let header = hlir
        .element_metadata
        .iter()
        .find(|m| m.id == Some("header".to_string()))
        .unwrap();
    let header_node = hlir.attributes.find_node(header.attributes_ref).unwrap();
    assert_eq!(
        header_node.computed.style.get("font-size"),
        Some(&"32pt".to_string())
    ); // From #header
    assert_eq!(
        header_node.computed.style.get("font-weight"),
        Some(&"bold".to_string())
    ); // From .title
    assert_eq!(
        header_node.computed.style.get("color"),
        Some(&"blue".to_string())
    ); // From #header

    // Section: type + class
    let section = hlir
        .element_metadata
        .iter()
        .find(|m| m.element_type == "section")
        .unwrap();
    let section_node = hlir.attributes.find_node(section.attributes_ref).unwrap();
    assert_eq!(section_node.computed.margin, Some(20.0)); // From section
    assert_eq!(section_node.computed.padding, Some(10.0)); // From section
    assert_eq!(
        section_node.computed.style.get("border"),
        Some(&"1px solid".to_string())
    ); // From .content

    // Body text: class (nested)
    let body = hlir
        .element_metadata
        .iter()
        .find(|m| m.classes.contains(&"body".to_string()))
        .unwrap();
    let body_node = hlir.attributes.find_node(body.attributes_ref).unwrap();
    assert_eq!(
        body_node.computed.style.get("font-size"),
        Some(&"14pt".to_string())
    );

    // Footer: class
    let footer = hlir
        .element_metadata
        .iter()
        .find(|m| m.classes.contains(&"footer".to_string()))
        .unwrap();
    let footer_node = hlir.attributes.find_node(footer.attributes_ref).unwrap();
    assert_eq!(
        footer_node.computed.style.get("font-size"),
        Some(&"10pt".to_string())
    );
    assert_eq!(
        footer_node.computed.style.get("color"),
        Some(&"gray".to_string())
    );
}

// ============================================================================
// Test with File Input
// ============================================================================

#[test]
fn test_css_from_file() {
    use std::fs;

    let data =
        fs::read_to_string("tests/input/css_test.ink").expect("Should be able to read test file");
    let tokens = lex(&data, "test_css_from_file").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    // Check that we have the expected structure
    assert!(
        hlir.element_metadata.len() >= 3,
        "Should have at least 3 elements"
    );
    assert!(!hlir.css_rules.is_empty(), "Should have CSS rules");

    // Check that styles were applied
    let header = hlir
        .element_metadata
        .iter()
        .find(|m| m.id == Some("header".to_string()));
    assert!(header.is_some(), "Should have header element");

    let header_node = hlir
        .attributes
        .find_node(header.unwrap().attributes_ref)
        .unwrap();
    assert!(
        !header_node.computed.style.is_empty(),
        "Header should have computed styles"
    );
}

#[test]
fn test_body_styles_are_document_styles_and_inherit() {
    let source = r#"
document {
    @text[Hello]
}
style {
    body {
        margin: 0.4in;
        font-size: 10pt;
        color: black;
    }
}
"#;
    let tokens =
        lex(source, "test_body_styles_are_document_styles_and_inherit").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    assert!((hlir.document_styles.margin.unwrap() - 28.8).abs() < 0.001);

    let text_metadata = &hlir.element_metadata[0];
    let text_node = hlir
        .attributes
        .find_node(text_metadata.attributes_ref)
        .unwrap();

    assert_eq!(
        text_node.computed.style.get("font-size"),
        Some(&"10pt".to_string())
    );
    assert_eq!(
        text_node.computed.style.get("color"),
        Some(&"black".to_string())
    );
}
