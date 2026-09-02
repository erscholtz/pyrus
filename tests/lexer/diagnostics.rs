use pyrus::diagnostic::{CompilerDiagnostic, SyntaxError};
use pyrus::lexer;

#[test]
fn records_unknown_character_as_syntax_diagnostic() {
    let errors = lexer::lex_all("^", "unknown_character").unwrap_err();

    assert!(matches!(
        errors.first(),
        Some(CompilerDiagnostic::Syntax(SyntaxError::InvalidConstruct { construct, .. }))
            if construct == "character"
    ));
}

#[test]
fn rejects_comment_between_at_and_element_name() {
    let result =
        lexer::lex_all("@ // comment\ntext[Hello]", "comment_after_at");

    assert!(result.is_err());
}

#[test]
fn rejects_newline_between_at_and_element_name() {
    let result = lexer::lex_all("@\ntext[Hello]", "newline_after_at");

    assert!(result.is_err());
}
