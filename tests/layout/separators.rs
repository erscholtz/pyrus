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
