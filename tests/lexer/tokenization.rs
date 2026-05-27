use std::fs;

use pyrus::lexer::{self, tokens::TokenKind};

fn lexeme(ts: &lexer::TokenStream, idx: usize) -> &str {
    let r = &ts.ranges[idx];
    &ts.source[r.clone()]
}

#[test]
fn lexes_sample_file_tokens() {
    let data = fs::read_to_string("tests/input/lexer_test.ink").expect("read sample file");
    let tokens = lexer::lex(&data, "lexer_test.ink").expect("Lexing failed");

    let non_ws_indices: Vec<usize> = tokens
        .kinds
        .iter()
        .enumerate()
        .filter_map(|(i, k)| (*k != TokenKind::Whitespace).then_some(i))
        .collect();

    assert!(!non_ws_indices.is_empty());
    assert_eq!(tokens.kinds[non_ws_indices[0]], TokenKind::Template);
    assert_eq!(tokens.kinds[non_ws_indices[1]], TokenKind::LeftBrace);

    let string_indices: Vec<usize> = tokens
        .kinds
        .iter()
        .enumerate()
        .filter_map(|(i, k)| matches!(*k, TokenKind::StringLiteral(_)).then_some(i))
        .collect();
    assert!(!string_indices.is_empty());

    let has_my_document = string_indices.iter().any(|&i| {
        let raw = lexeme(&tokens, i);
        raw.contains("My Document") || raw.trim_matches('"').contains("My Document")
    });
    assert!(has_my_document);
    assert_eq!(*tokens.kinds.last().unwrap(), TokenKind::Eof);
}

#[test]
fn lexes_single_equals_as_assignment() {
    let tokens = lexer::lex("=", "single_equals").expect("Lexing failed");
    assert_eq!(tokens.kinds[0], TokenKind::Assign);
}

#[test]
fn distinguishes_keywords_from_identifiers() {
    let tokens = lexer::lex("template template_name text text2", "keywords").unwrap();
    assert_eq!(
        &tokens.kinds[..4],
        &[
            TokenKind::Template,
            TokenKind::Identifier,
            TokenKind::Text,
            TokenKind::Identifier
        ]
    );
}

#[test]
fn records_integer_and_float_ranges() {
    let tokens = lexer::lex("42 3.14", "numbers").unwrap();
    assert_eq!(&tokens.kinds[..2], &[TokenKind::Int, TokenKind::Float]);
    assert_eq!(lexeme(&tokens, 0), "42");
    assert_eq!(lexeme(&tokens, 1), "3.14");
    assert_eq!(tokens.ranges[0], 0..2);
    assert_eq!(tokens.ranges[1], 3..7);
}

#[test]
fn skips_line_and_block_comments() {
    let tokens = lexer::lex("let // hidden\nconst /* hidden */ name", "comments").unwrap();
    assert_eq!(
        tokens.kinds,
        vec![
            TokenKind::Let,
            TokenKind::Const,
            TokenKind::Identifier,
            TokenKind::Eof
        ]
    );
}

#[test]
fn tracks_token_location_after_newline() {
    let tokens = lexer::lex("let\nname", "locations").unwrap();
    assert_eq!(tokens.kinds[1], TokenKind::Identifier);
    assert_eq!(tokens.lines[1], 2);
    assert_eq!(tokens.cols[1], 1);
}
