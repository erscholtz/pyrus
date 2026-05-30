use crate::support::{parse_ast, parse_errors};
use pyrus::{
    ast::{ExprKind, Selector},
    diagnostic::SyntaxError,
    lexer::tokens::TokenKind,
};

fn rules(source: &str) -> Vec<pyrus::ast::StyleRule> {
    parse_ast(source)
        .style
        .expect("Expected style block")
        .statements
}

#[test]
fn parses_type_class_and_id_selectors() {
    let rules =
        rules("style { body { color: red; } .hero { color: blue; } #title { color: gold; } }");
    assert!(matches!(&rules[0].selector_list[0], Selector::Type(name) if name == "body"));
    assert!(matches!(&rules[1].selector_list[0], Selector::Class(name) if name == "hero"));
    assert!(matches!(&rules[2].selector_list[0], Selector::Id(name) if name == "title"));
}

#[test]
fn parses_grouped_selectors() {
    let rules = rules("style { body, .hero, #title { color: red; } }");
    assert_eq!(rules[0].selector_list.len(), 3);
}

#[test]
fn parses_declaration_units() {
    let rules = rules("style { body { font-size: 12pt; } }");
    let declaration = &rules[0].declaration_block[0];
    assert_eq!(declaration.key, "font-size");
    assert!(matches!(declaration.value.expr.node, ExprKind::Int(12)));
    assert_eq!(declaration.value.unit.as_deref(), Some("pt"));
}

#[test]
fn accepts_colon_and_assignment_declaration_separators() {
    let rules = rules("style { body { color: red; weight = bold; } }");
    assert_eq!(rules[0].declaration_block.len(), 2);
}

#[test]
fn reports_missing_declaration_semicolon() {
    let errors = parse_errors("style { body { color: red } }");
    assert!(matches!(
        &errors[0],
        SyntaxError::UnexpectedToken { expected, .. }
            if expected.contains(&TokenKind::Semicolon)
    ));
}
