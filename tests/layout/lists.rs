use pyrus::ast::Ast;
use pyrus::hir::{hir_types::HIRModule, lower};
use pyrus::layout::setup_layout;
use pyrus::lexer::{TokenStream, lex};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
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
