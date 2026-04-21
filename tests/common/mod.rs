use pyrus::{
    ast::{Ast, DocElem, Stmt},
    diagnostic::SyntaxError,
    lexer::lex,
    parser::Parser,
};

pub fn parse_ast(source: &str) -> Ast {
    let tokens = lex(source, "test.ink").expect("Lexing failed");
    let mut parser = Parser::new(tokens);
    parser.parse::<Ast>().expect("Parsing failed")
}

pub fn parse_errors(source: &str) -> Vec<SyntaxError> {
    let tokens = lex(source, "test.ink").expect("Lexing failed");
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
