use pyrus::diagnostic::{CompilerDiagnostic, SyntaxError};
use pyrus::lexer::{self, tokens::TokenKind};

#[test]
fn records_unknown_character_as_syntax_diagnostic() {
    let errors = lexer::lex_all("^", "unknown_character").unwrap_err();

    assert!(matches!(
        errors.first(),
        Some(CompilerDiagnostic::Syntax(SyntaxError::InvalidConstruct { construct, .. }))
            if construct == "character"
    ));
}
