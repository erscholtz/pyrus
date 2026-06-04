use pyrus::diagnostic::{CompilerDiagnostic, SyntaxError};
use pyrus::lexer::{self, tokens::TokenKind};

fn kind(tokens: &lexer::TokenStream, idx: usize) -> TokenKind {
    tokens.tokens[idx].kind
}

#[test]
fn lexes_text_body_as_single_string_literal() {
    let tokens = lexer::lex_all("@text[${price} * quantity]", "text_body").unwrap();

    assert_eq!(kind(&tokens, 0), TokenKind::At);
    assert_eq!(kind(&tokens, 1), TokenKind::Text);
    assert_eq!(kind(&tokens, 2), TokenKind::LeftBracket);
    let body_idx = match kind(&tokens, 3) {
        TokenKind::StringLiteral(idx) => idx,
        ref other => panic!("Expected text body string, got {other:?}"),
    };
    assert_eq!(tokens.string_table[body_idx].content, "${price} * quantity");
    assert!(tokens.string_table[body_idx].has_interpolation);
    assert_eq!(kind(&tokens, 4), TokenKind::RightBracket);
    assert_eq!(kind(&tokens, 5), TokenKind::Eof);
}

#[test]
fn lexes_text_body_after_attributes_without_space() {
    let tokens = lexer::lex_all(r#"@text(class="hero")[Hello]"#, "attrs").unwrap();
    let body_idx = match kind(&tokens, 8) {
        TokenKind::StringLiteral(idx) => idx,
        ref other => panic!("Expected text body string, got {other:?}"),
    };

    assert_eq!(kind(&tokens, 7), TokenKind::LeftBracket);
    assert_eq!(tokens.string_table[body_idx].content, "Hello");
    assert_eq!(kind(&tokens, 9), TokenKind::RightBracket);
}

#[test]
fn lexes_text_body_after_attributes_with_space() {
    let tokens = lexer::lex_all(r#"@text(class="hero") [ text ]"#, "attrs").unwrap();
    let body_idx = match kind(&tokens, 8) {
        TokenKind::StringLiteral(idx) => idx,
        ref other => panic!("Expected text body string, got {other:?}"),
    };

    assert_eq!(tokens.string_table[body_idx].content, " text ");
    assert_eq!(kind(&tokens, 10), TokenKind::Eof);
}

#[test]
fn records_unterminated_text_body_diagnostic() {
    let errors = lexer::lex_all("@text[unfinished", "text_body").unwrap_err();
    assert!(matches!(
        errors.first(),
        Some(CompilerDiagnostic::Syntax(SyntaxError::UnterminatedDelimiter {
            delimiter,
            ..
        })) if delimiter == "]"
    ));
}
