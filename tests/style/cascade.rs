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
