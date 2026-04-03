//! Tests for CSS style resolution in HLIR

use pyrus::hir::{lower, resolve_styles};
use pyrus::lexer::lex;
use pyrus::parser::parse;

// ============================================================================
// Basic Selector Tests
// ============================================================================

#[test]
fn test_id_selector() {
    let source = r#"
document {
    text (id="header") { "Header" }
}
style {
    #header {
        font-size: 24pt;
        color: blue;
    }
}
"#;
    let tokens = lex(source, "test_id_selector").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);

    assert_eq!(hlir.element_metadata.len(), 1);
    assert_eq!(hlir.css_rules.len(), 1);

    resolve_styles(&mut hlir);

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
    text (class="intro") { "Introduction" }
}
style {
    .intro {
        font-weight: bold;
    }
}
"#;
    let tokens = lex(source, "test_class_selector").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    assert_eq!(
        node.computed.style.get("font-weight"),
        Some(&"bold".to_string())
    );
}

#[test]
fn test_type_selector() {
    let source = r#"
document {
    text { "Some text" }
}
style {
    text {
        font-size: 12;
    }
}
"#;
    let tokens = lex(source, "test_type_selector").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"12".to_string())
    );
}

// ============================================================================
// Specificity Tests
// ============================================================================

#[test]
fn test_specificity_inline_wins() {
    let source = r#"
document {
    text (id="mytext", style="font-size: 10") { "Text" }
}
style {
    #mytext {
        font-size: 24;
    }
}
"#;
    let tokens = lex(source, "test_specificity_inline_wins").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // Inline style="font-size: 10" should win over CSS
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"10".to_string())
    );
}

#[test]
fn test_specificity_id_over_class() {
    let source = r#"
document {
    text (id="mytext", class="intro") { "Text" }
}
style {
    #mytext {
        font-size: 24;
    }
    .intro {
        font-size: 12;
    }
}
"#;
    let tokens = lex(source, "test_specificity_id_over_class").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // ID selector (specificity 100) should win over class (specificity 10)
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"24".to_string())
    );
}

#[test]
fn test_specificity_class_over_type() {
    let source = r#"
document {
    text (class="intro") { "Text" }
}
style {
    text {
        font-size: 12;
    }
    .intro {
        font-size: 24;
    }
}
"#;
    let tokens = lex(source, "test_specificity_class_over_type").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // Class selector (specificity 10) should win over type (specificity 1)
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"24".to_string())
    );
}

// ============================================================================
// Multiple Rules Tests
// ============================================================================

#[test]
fn test_multiple_rules_combine() {
    let source = r#"
document {
    text (class="intro") { "Text" }
}
style {
    .intro {
        font-size: 24;
    }
    .intro {
        color: red;;
    }
}
"#;
    let tokens = lex(source, "test_multiple_rules_combine").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // Both rules should apply to the same element
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"24".to_string())
    );
    assert_eq!(node.computed.style.get("color"), Some(&"red".to_string()));
}

#[test]
fn test_same_property_last_wins() {
    let source = r#"
document {
    text (class="intro") { "Text" }
}
style {
    .intro {
        font-size: 12;
    }
    .intro {
        font-size: 24;
    }
}
"#;
    let tokens = lex(source, "test_same_property_last_wins").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // Same specificity, second rule should win
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"24".to_string())
    );
}

// ============================================================================
// Multiple Selectors in One Rule
// ============================================================================

#[test]
fn test_multiple_selectors_in_rule() {
    let source = r#"
document {
    text (class="header") { "Header" }
    text (class="footer") { "Footer" }
}
style {
    .header, .footer {
        font-weight: bold;;
    }
}
"#;
    let tokens = lex(source, "test_multiple_selectors_in_rule").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    // Both elements should get the style
    for i in 0..2 {
        let metadata = &hlir.element_metadata[i];
        let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();
        assert_eq!(
            node.computed.style.get("font-weight"),
            Some(&"bold".to_string()),
            "Element {} should have font-weight: bold",
            i
        );
    }
}

// ============================================================================
// Inheritance Tests
// ============================================================================

#[test]
fn test_style_inheritance() {
    let source = r#"
document {
    section {
        text { "Nested text" }
    }
}
style {
    section {
        color: blue;;
    }
}
"#;
    let tokens = lex(source, "test_style_inheritance").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

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
    section {
        text { "Nested text" }
    }
}
style {
    section {
        margin: 20pt;
    }
}
"#;
    let tokens = lex(source, "test_non_inherited_properties").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

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
    text (class="large bold") { "Text" }
}
style {
    .large {
        font-size: 24;
    }
    .bold {
        font-weight: bold;;
    }
}
"#;
    let tokens = lex(source, "test_multiple_classes_on_element").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

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
// Typed Properties Tests (margin, padding, etc.)
// ============================================================================

#[test]
fn test_typed_margin_property() {
    let source = r#"
document {
    text { "Text" }
}
style {
    text {
        margin: 15pt;
    }
}
"#;
    let tokens = lex(source, "test_typed_margin_property").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // margin should be parsed as f32
    assert_eq!(node.computed.margin, Some(15.0));
}

#[test]
fn test_typed_padding_property() {
    let source = r#"
document {
    text { "Text" }
}
style {
    text {
        padding: 10pt;
    }
}
"#;
    let tokens = lex(source, "test_typed_padding_property").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

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
    text (id="mytext") { "Text" }
}
style {
    .nomatch {
        font-size: 24;
    }
}
"#;
    let tokens = lex(source, "test_no_matching_rules").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

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
    text { "Text" }
}
style {
}
"#;
    let tokens = lex(source, "test_empty_style_block").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // No styles should be applied
    assert!(node.computed.style.is_empty());
}

#[test]
fn test_no_style_block() {
    let source = r#"
document {
    text { "Text" }
}
"#;
    let tokens = lex(source, "test_no_style_block").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);

    assert!(hlir.css_rules.is_empty());

    resolve_styles(&mut hlir);

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
    text (id="header", class="title") { "Header" }
    section (class="content") {
        text (class="body") { "Body text" }
    }
    text (class="footer") { "Footer" }
}
style {
    #header {
        font-size: 32pt;
        color: blue;;
    }
    .title {
        font-weight: bold;;
    }
    section {
        margin: 20pt;
        padding: 10pt;
    }
    .content {
        border: 1px solid;
    }
    .body {
        font-size: 14pt;
    }
    .footer {
        font-size: 10pt;
        color: gray;;
    }
}
"#;
    let tokens = lex(source, "test_complex_css_scenario").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let mut hlir = lower(&ast);
    resolve_styles(&mut hlir);

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
    let mut hlir = lower(&ast);

    // Check that we have the expected structure
    assert!(
        hlir.element_metadata.len() >= 3,
        "Should have at least 3 elements"
    );
    assert!(!hlir.css_rules.is_empty(), "Should have CSS rules");

    resolve_styles(&mut hlir);

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
