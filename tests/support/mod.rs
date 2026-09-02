#![allow(dead_code)]

use pyrus::{
    ast::{Ast, DocElem, Stmt},
    diagnostic::SyntaxError,
    hir::{hir_types::HIRModule, lower},
    lexer::lex_all,
    parser::Parser,
};

pub fn parse_ast(source: &str) -> Ast {
    let tokens = lex_all(source, "test.ink").expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    parser.parse::<Ast>().expect("Parsing failed")
}

pub fn parse_errors(source: &str) -> Vec<SyntaxError> {
    let tokens = lex_all(source, "test.ink").expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    parser.parse::<Ast>().expect_err("Parsing should fail")
}

pub fn template_statements(source: &str) -> Vec<Stmt> {
    parse_ast(source)
        .template
        .expect("Expected template block")
        .statements
}

pub fn document_elements(source: &str) -> Vec<DocElem> {
    parse_ast(source)
        .document
        .expect("Expected document block")
        .elements
}

pub fn lower_source(source: &str) -> HIRModule {
    let ast = parse_ast(source);
    lower(&ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
}
