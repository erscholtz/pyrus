use super::Cursor;
use crate::{
    diagnostic::{SourceLocation, SyntaxError},
    lexer::{
        tokens::{StringEntry, TokenKind},
        Token, TokenStream,
    },
};

fn token(kind: TokenKind, start: usize, end: usize, line: usize, col: usize) -> Token {
    Token {
        kind,
        range: start..end,
        line,
        col,
    }
}

fn cursor(tokens: Vec<Token>) -> Cursor {
    let mut stream = TokenStream::new("cursor.ink".to_string());
    stream.tokens = tokens;
    stream.identifier_table = vec!["value".to_string()];
    Cursor::new(stream)
}

fn cursor_with_string_table(tokens: Vec<Token>, string_table: Vec<StringEntry>) -> Cursor {
    let mut stream = TokenStream::new("cursor.ink".to_string());
    stream.tokens = tokens;
    stream.identifier_table = vec!["value".to_string()];
    stream.string_table = string_table;
    Cursor::new(stream)
}

#[test]
fn exposes_initial_token_text_and_location() {
    let cursor = cursor(vec![
        token(TokenKind::Let, 0, 3, 1, 1),
        token(TokenKind::Identifier(0), 4, 9, 1, 5),
        token(TokenKind::Eof, 9, 9, 1, 10),
    ]);
    assert_eq!(cursor.cur_tok(), &TokenKind::Let);
    assert_eq!(cursor.cur_range(), Some(0..3));
    assert_eq!(cursor.location(), SourceLocation::new(1, 1, "cursor.ink"));
}

#[test]
fn advances_and_peeks_without_consuming_lookahead() {
    let mut cursor = cursor(vec![
        token(TokenKind::Let, 0, 3, 1, 1),
        token(TokenKind::Identifier(0), 4, 9, 1, 5),
        token(TokenKind::Eof, 9, 9, 1, 10),
    ]);
    assert!(matches!(cursor.peek_tok(), Some(TokenKind::Identifier(_))));
    assert_eq!(cursor.peek_text(), Some("value"));
    cursor.advance();
    assert!(matches!(cursor.cur_tok(), TokenKind::Identifier(_)));
    assert_eq!(cursor.cur_text(), "value");
    assert_eq!(cursor.cur_range(), Some(4..9));
}

#[test]
fn restores_a_checkpoint() {
    let mut cursor = cursor(vec![
        token(TokenKind::Let, 0, 3, 1, 1),
        token(TokenKind::Identifier(0), 4, 9, 1, 5),
        token(TokenKind::Eof, 9, 9, 1, 10),
    ]);
    let checkpoint = cursor.checkpoint();
    cursor.advance();
    cursor.restore(checkpoint);
    assert_eq!(cursor.cur_tok(), &TokenKind::Let);
}

#[test]
fn expect_consumes_matches_and_reports_mismatches() {
    let mut cursor = cursor(vec![
        token(TokenKind::Let, 0, 3, 1, 1),
        token(TokenKind::Identifier(0), 4, 9, 1, 5),
        token(TokenKind::Eof, 9, 9, 1, 10),
    ]);
    assert_eq!(cursor.expect(TokenKind::Let).unwrap(), TokenKind::Let);
    assert!(matches!(
        cursor.expect(TokenKind::Const),
        Err(SyntaxError::UnexpectedToken {
            found: TokenKind::Identifier(_),
            ..
        })
    ));
}

#[test]
fn accesses_string_table_entries() {
    let cursor = cursor_with_string_table(
        vec![
            token(TokenKind::StringLiteral(0), 0, 9, 1, 1),
            token(TokenKind::Eof, 9, 9, 1, 10),
        ],
        vec![StringEntry {
            content: "content".to_string(),
            has_interpolation: false,
        }],
    );
    assert_eq!(cursor.get_string(0).unwrap().content, "content");
    assert_eq!(cursor.get_string_entry(0).unwrap().content, "content");
}
