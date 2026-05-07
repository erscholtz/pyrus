use pyrus::ast::Ast;
use pyrus::diagnostic::DiagnosticManager;
use pyrus::hir::{hir_types::HIRModule, lower};
use pyrus::lexer::{TokenStream, lex};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    let mut diagnostics = DiagnosticManager::new();
    lower(ast, &mut diagnostics)
        .unwrap_or_else(|_| panic!("Lowering failed: {:?}", diagnostics.diagnostics()))
}

fn text_contents(hlir: &HIRModule) -> Vec<&str> {
    hlir.elements
        .iter()
        .filter_map(|element| match element {
            pyrus::hir::hir_types::HirElementOp::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn test_template_call_substitutes_text_arguments() {
    let source = r#"
template {
    func header(name: String, linkedin: String) {
        return @section[
            @text[${name}]
            @text[linkedin.com/in/${linkedin}]
        ]
    }
}
document {
    @header("Erik Scholtz", "erikscholtz")
}
"#;
    let tokens =
        lex(source, "test_template_call_substitutes_text_arguments").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let contents = text_contents(&hlir);

    assert!(contents.contains(&"Erik Scholtz"));
    assert!(contents.contains(&"linkedin.com/in/erikscholtz"));
}

#[test]
fn test_each_template_call_gets_its_own_substitutions() {
    let source = r#"
template {
    func item(name: String) {
        return @text[${name}]
    }
}
document {
    @item("First")
    @item("Second")
}
"#;
    let tokens =
        lex(source, "test_each_template_call_gets_its_own_substitutions").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let contents = text_contents(&hlir);

    assert!(contents.contains(&"First"));
    assert!(contents.contains(&"Second"));
}
