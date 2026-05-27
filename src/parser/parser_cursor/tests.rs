use super::Cursor;
use crate::{
    diagnostic::{SourceLocation, SyntaxError},
    lexer::{lex, tokens::TokenKind},
};

fn cursor(source: &str) -> Cursor {
    Cursor::new(lex(source, "cursor.ink").expect("Lexing failed"))
}

#[test]
fn exposes_initial_token_text_and_location() {
    let cursor = cursor("let value");
    assert_eq!(cursor.cur_tok(), &TokenKind::Let);
    assert_eq!(cursor.cur_text(), "let");
    assert_eq!(cursor.location(), SourceLocation::new(1, 1, "cursor.ink"));
}

#[test]
fn advances_and_peeks_without_consuming_lookahead() {
    let mut cursor = cursor("let value");
    assert_eq!(cursor.peek_tok(), Some(&TokenKind::Identifier));
    assert_eq!(cursor.peek_text(), Some("value"));
    cursor.advance();
    assert_eq!(cursor.cur_tok(), &TokenKind::Identifier);
    assert_eq!(cursor.cur_text(), "value");
}

#[test]
fn restores_a_checkpoint() {
    let mut cursor = cursor("let value");
    let checkpoint = cursor.checkpoint();
    cursor.advance();
    cursor.restore(checkpoint);
    assert_eq!(cursor.cur_tok(), &TokenKind::Let);
}

#[test]
fn expect_consumes_matches_and_reports_mismatches() {
    let mut cursor = cursor("let value");
    assert_eq!(cursor.expect(TokenKind::Let).unwrap(), TokenKind::Let);
    assert!(matches!(
        cursor.expect(TokenKind::Const),
        Err(SyntaxError::UnexpectedToken {
            found: TokenKind::Identifier,
            ..
        })
    ));
}

#[test]
fn accesses_string_table_entries() {
    let cursor = cursor("\"content\"");
    assert_eq!(cursor.get_string(0).unwrap().content, "content");
    assert_eq!(cursor.get_string_entry(0).unwrap().content, "content");
}
