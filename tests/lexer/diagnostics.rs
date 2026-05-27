use pyrus::diagnostic::{CompilerDiagnostic, SyntaxError};
use pyrus::lexer::{self, tokens::TokenKind};

#[test]
fn records_unknown_character_as_syntax_diagnostic() {
    let tokens = lexer::lex("^", "unknown_character").unwrap();

    assert!(matches!(
        tokens.errors.first(),
        Some(CompilerDiagnostic::Syntax(SyntaxError::InvalidConstruct { construct, .. }))
            if construct == "character"
    ));
    assert_eq!(tokens.kinds.last(), Some(&TokenKind::Eof));
}
