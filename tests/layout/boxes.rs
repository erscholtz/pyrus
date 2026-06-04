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
    let tokens = lex_all(
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
