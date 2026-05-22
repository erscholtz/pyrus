//! Tests for CSS style resolution in HLIR

use pyrus::ast::Ast;
use pyrus::hir::{
    hir_types::{HIRModule, HirElementOp},
    lower,
};
use pyrus::layout::setup_layout;
use pyrus::lexer::{TokenStream, lex};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {:?}", errors))
}

// ============================================================================
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
    let tokens = lex(source, "test_id_selector").expect("Lexing failed");
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
    let tokens = lex(source, "test_class_selector").expect("Lexing failed");
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
    let tokens = lex(source, "test_selector_applies_to_element").expect("Lexing failed");
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
// Specificity Tests
// ============================================================================

#[test]
fn test_specificity_inline_wins() {
    let source = r#"
document {
    @text(id="mytext", style="font-size: 10")[Text]
}
style {
    #mytext {
        font-size: 24;
    }
}
"#;
    let tokens = lex(source, "test_specificity_inline_wins").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

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
    @text(id="mytext", class="intro")[Text]
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
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // ID selector (specificity 100) should win over class (specificity 10)
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"24".to_string())
    );
}

#[test]
fn test_specificity_class_over_less_specific_rule() {
    let source = r#"
document {
    @text(id="copy", class="intro")[Text]
}
style {
    #copy {
        font-size: 12;
    }
    .intro {
        font-size: 24;
    }
}
"#;
    let tokens =
        lex(source, "test_specificity_class_over_less_specific_rule").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // Higher specificity should win even when the lower-specificity rule appears later.
    assert_eq!(
        node.computed.style.get("font-size"),
        Some(&"12".to_string())
    );
}

#[test]
fn test_grouped_selector_uses_matched_selector_specificity() {
    let source = r#"
document {
    @text(class="intro")[Text]
}
style {
    .intro, #unmatched {
        font-size: 12;
    }
    .intro {
        font-size: 24;
    }
}
"#;
    let tokens = lex(
        source,
        "test_grouped_selector_uses_matched_selector_specificity",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

    let metadata = &hlir.element_metadata[0];
    let node = hlir.attributes.find_node(metadata.attributes_ref).unwrap();

    // The first rule matches through `.intro`, not `#unmatched`, so both
    // declarations have class specificity and the later rule should win.
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
    @text(class="intro")[Text]
}
style {
    .intro {
        font-size: 24;
    }
    .intro {
        color: red;
    }
}
"#;
    let tokens = lex(source, "test_multiple_rules_combine").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

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
    @text(class="intro")[Text]
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
    let hlir = lower_ast(&ast);

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
    @text(class="header")[Header]
    @text(class="footer")[Footer]
}
style {
    .header, .footer {
        font-weight: bold;
    }
}
"#;
    let tokens = lex(source, "test_multiple_selectors_in_rule").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);

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

#[test]
fn test_document_flow_uses_body_margin_and_element_spacing() {
    let source = r#"
document {
    @section(class="spaced")[
        @text[Hello]
    ]
}
style {
    body {
        margin: 0.5in;
        font-size: 10pt;
    }

    .spaced {
        margin-top: 6pt;
        margin-bottom: 4pt;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_uses_body_margin_and_element_spacing",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let text_layout = computed
        .iter()
        .find(|layout| hlir.element_metadata[layout.element_index].element_type == "text")
        .expect("Text should have a computed layout");

    assert!((text_layout.x - 36.0).abs() < 0.001);
    assert!((text_layout.y - 42.0).abs() < 0.001);
    assert!((text_layout.width - 523.0).abs() < 0.001);
    assert!((text_layout.height - 12.0).abs() < 0.001);
}

#[test]
fn test_document_flow_adds_unordered_list_markers() {
    let source = r#"
document {
    @list[
        - @text[First]
        - @text[Second]
    ]
}
"#;
    let tokens =
        lex(source, "test_document_flow_adds_unordered_list_markers").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let markers: Vec<_> = computed
        .iter()
        .filter_map(|layout| layout.marker.as_deref())
        .collect();

    assert_eq!(markers, vec!["-", "-"]);
}

#[test]
fn test_document_flow_adds_decimal_list_markers() {
    let source = r#"
document {
    @list(class="steps")[
        - @text[First]
        - @text[Second]
    ]
}
style {
    .steps {
        list-style-type: decimal;
    }
}
"#;
    let tokens =
        lex(source, "test_document_flow_adds_decimal_list_markers").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let markers: Vec<_> = computed
        .iter()
        .filter_map(|layout| layout.marker.as_deref())
        .collect();

    assert_eq!(markers, vec!["1.", "2."]);
}

#[test]
fn test_document_flow_lays_out_separator() {
    let source = r#"
document {
    @text[Before]
    @separator(class="rule")
    @text[After]
}
style {
    .rule {
        height: 2pt;
        margin-top: 3pt;
        margin-bottom: 5pt;
    }
}
"#;
    let tokens = lex(source, "test_document_flow_lays_out_separator").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let separator_layout = computed
        .iter()
        .find(|layout| hlir.element_metadata[layout.element_index].element_type == "separator")
        .expect("Separator should have a computed layout");
    let following_text_layout = computed
        .iter()
        .filter(|layout| hlir.element_metadata[layout.element_index].element_type == "text")
        .nth(1)
        .expect("Text after separator should have a computed layout");

    assert!((separator_layout.height - 2.0).abs() < 0.001);
    assert!(following_text_layout.y > separator_layout.y + separator_layout.height);
}

#[test]
fn test_document_flow_separator_matches_body_content_width() {
    let source = r#"
document {
    @separator(class="rule")
}
style {
    body {
        margin: 36pt;
    }

    .rule {
        height: 0.45pt;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_separator_matches_body_content_width",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let separator_layout = computed
        .iter()
        .find(|layout| hlir.element_metadata[layout.element_index].element_type == "separator")
        .expect("Separator should have a computed layout");

    assert!((separator_layout.x - 36.0).abs() < 0.001);
    assert!((separator_layout.width - 523.0).abs() < 0.001);
}

#[test]
fn test_document_flow_row_uses_gap_and_nowrap_side_metadata() {
    let source = r#"
document {
    @section(class="row")[
        @text(class="title")[Project Title]
        @text(class="side")[September 2025 - Present]
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 20pt;
    }

    .side {
        width: 130pt;
        white-space: nowrap;
        text-align: right;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_uses_gap_and_nowrap_side_metadata",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let side_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"side".to_string())
        })
        .expect("Side metadata should have layout");

    assert!(side_layout.nowrap);
    assert!(side_layout.x > title_layout.x + title_layout.width);
    assert!((side_layout.x - (title_layout.x + title_layout.width)) >= 19.9);
    assert!((side_layout.box_x + side_layout.box_width - 595.0).abs() < 0.001);
    assert!(side_layout.x > side_layout.box_x);
    assert!(side_layout.x + side_layout.width <= side_layout.box_x + side_layout.box_width);
}

#[test]
fn test_document_flow_row_lays_out_wrapped_link_component_on_right() {
    let source = r#"
template {
    func side_link(url: String) {
        return @link(class="badge")["${url}", "GitHub"]
    }
}

document {
    @section(class="row")[
        @text(class="title")[Project Title]
        @side_link("github.com/example/project")
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 10pt;
    }

    .badge {
        white-space: nowrap;
        padding-top: 1pt;
        padding-right: 3pt;
        padding-bottom: 1pt;
        padding-left: 3pt;
        border: "0.55pt solid #1f4e79";
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_lays_out_wrapped_link_component_on_right",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let badge_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"badge".to_string())
        })
        .expect("Wrapped badge link should have layout");

    assert!(badge_layout.nowrap);
    assert!(badge_layout.box_width > badge_layout.width);
    assert!(badge_layout.x > title_layout.x + title_layout.width);
    assert!(title_layout.x + title_layout.width <= badge_layout.box_x - 9.9);
}

#[test]
fn test_document_flow_row_lays_out_grouped_project_links_on_right() {
    let source = r#"
template {
    func project_link(url: String, label: String) {
        return @link(class="project_link")["${url}", "${label}"]
    }

    func project_links(paper_url: String, github_url: String, demo_url: String) {
        return @section(class="project_links")[
            @project_link("${paper_url}", "[paper")
            @text(class="project_link_sep")[|]
            @project_link("${github_url}", "github")
            @text(class="project_link_sep")[|]
            @project_link("${demo_url}", "demo]")
        ]
    }
}

document {
    @section(class="row")[
        @text(class="title")[Project Title]
        @project_links(
            "example.com/paper",
            "github.com/example/project",
            "example.com/demo"
        )
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 10pt;
    }

    .project_links {
        width: 130pt;
        display: flex;
        flex-direction: row;
        justify-content: flex-end;
        column-gap: 3pt;
        white-space: nowrap;
    }

    .project_link,
    .project_link_sep {
        white-space: nowrap;
        font-size: 8pt;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_lays_out_grouped_project_links_on_right",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let link_layout = |content: &str| {
        computed
            .iter()
            .find(|layout| {
                matches!(
                    &hlir.elements[layout.element_index],
                    HirElementOp::Link { content: link_content, .. } if link_content == content
                )
            })
            .expect("Project link should have layout")
    };

    let paper_layout = link_layout("[paper");
    let github_layout = link_layout("github");
    let demo_layout = link_layout("demo]");

    assert!(paper_layout.nowrap);
    assert!(github_layout.nowrap);
    assert!(demo_layout.nowrap);
    assert!(paper_layout.x > title_layout.x + title_layout.width);
    assert!(github_layout.x > paper_layout.x + paper_layout.width);
    assert!(demo_layout.x > github_layout.x + github_layout.width);
    assert!((demo_layout.box_x + demo_layout.box_width - 595.0).abs() < 0.001);
}

#[test]
fn test_document_flow_row_wraps_left_before_nowrap_side_metadata() {
    let source = r#"
document {
    @section(class="row")[
        @text(class="title")[This is a deliberately long project title that needs to wrap before it reaches the right aligned metadata and it keeps going with more descriptive project words to force another line]
        @text(class="side")[GitHub]
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 10pt;
    }

    .side {
        width: 70pt;
        white-space: nowrap;
        text-align: right;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_wraps_left_before_nowrap_side_metadata",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let side_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"side".to_string())
        })
        .expect("Side metadata should have layout");

    assert!(title_layout.height > 12.0);
    assert!(side_layout.nowrap);
    assert!((side_layout.box_x + side_layout.box_width - 595.0).abs() < 0.001);
    assert!(title_layout.x + title_layout.width <= side_layout.box_x - 9.9);
}

#[test]
fn test_document_flow_list_uses_configurable_hanging_indent() {
    let source = r#"
document {
    @list(class="bullets")[
        - @text[This is a long bullet item that should wrap while keeping the continuation aligned with the text content and it keeps adding implementation detail about layout engines renderers annotations spacing and typography until the line has no choice but to wrap]
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .bullets {
        padding-left: 11pt;
        marker-width: 5pt;
        marker-gap: 3pt;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_list_uses_configurable_hanging_indent",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let item_layout = computed
        .iter()
        .find(|layout| layout.marker.is_some())
        .expect("List item should have marker layout");

    assert_eq!(item_layout.marker.as_deref(), Some("-"));
    assert!((item_layout.marker_x.unwrap() - 11.0).abs() < 0.001);
    assert!((item_layout.x - 19.0).abs() < 0.001);
    assert!(item_layout.height > 12.0);
}

#[test]
fn test_document_flow_link_badge_uses_padding_for_box_geometry() {
    let source = r#"
document {
    @link(class="badge")["github.com/example/project", "GitHub"]
}
style {
    body {
        font-size: 10pt;
    }

    .badge {
        white-space: nowrap;
        padding-top: 1pt;
        padding-right: 3pt;
        padding-bottom: 1pt;
        padding-left: 3pt;
        border: "0.55pt solid #1f4e79";
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_link_badge_uses_padding_for_box_geometry",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let badge_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"badge".to_string())
        })
        .expect("Badge link should have layout");

    assert!(badge_layout.nowrap);
    assert!((badge_layout.x - 3.0).abs() < 0.001);
    assert!((badge_layout.box_x - 0.0).abs() < 0.001);
    assert!((badge_layout.box_width - (badge_layout.width + 6.0)).abs() < 0.001);
    assert!((badge_layout.box_height - (badge_layout.height + 2.0)).abs() < 0.001);
}
