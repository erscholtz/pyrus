use pyrus::ast::Ast;
use pyrus::hir::{hir_types::HIRModule, lower};
use pyrus::layout::setup_layout;
use pyrus::lexer::{TokenStream, lex_all};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
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
    let tokens = lex_all(
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
