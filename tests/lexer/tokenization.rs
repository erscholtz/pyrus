use std::fs;

use pyrus::lexer::{self, tokens::TokenKind};

fn lexeme(ts: &lexer::TokenStream, idx: usize) -> &str {
    let r = &ts.tokens[idx].range;
    &ts.source[r.clone()]
}

fn kind(ts: &lexer::TokenStream, idx: usize) -> TokenKind {
    ts.tokens[idx].kind
}

fn kinds(ts: &lexer::TokenStream) -> Vec<TokenKind> {
    ts.tokens.iter().map(|token| token.kind).collect()
}

#[test]
fn lexes_sample_file_tokens() {
    let data = fs::read_to_string("tests/input/lexer_test.ink").expect("read sample file");
    let tokens = lexer::lex_all(&data, "lexer_test.ink").expect("Lexing failed");

    let non_ws_indices: Vec<usize> = tokens
        .tokens
        .iter()
        .enumerate()
        .filter_map(|(i, token)| (token.kind != TokenKind::Whitespace).then_some(i))
        .collect();

    assert!(!non_ws_indices.is_empty());
    assert_eq!(kind(&tokens, non_ws_indices[0]), TokenKind::Template);
    assert_eq!(kind(&tokens, non_ws_indices[1]), TokenKind::LeftBrace);

    let string_indices: Vec<usize> = tokens
        .tokens
        .iter()
        .enumerate()
        .filter_map(|(i, token)| matches!(token.kind, TokenKind::StringLiteral(_)).then_some(i))
        .collect();
    assert!(!string_indices.is_empty());

    let has_my_document = string_indices.iter().any(|&i| {
        let raw = lexeme(&tokens, i);
        raw.contains("My Document") || raw.trim_matches('"').contains("My Document")
    });
    assert!(has_my_document);
    assert_eq!(tokens.tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn lexes_single_equals_as_assignment() {
    let tokens = lexer::lex_all("=", "single_equals").expect("Lexing failed");
    assert_eq!(kind(&tokens, 0), TokenKind::Assign);
}

#[test]
fn distinguishes_keywords_from_identifiers() {
    let tokens = lexer::lex_all("template template_name text text2", "keywords").unwrap();
    assert_eq!(kind(&tokens, 0), TokenKind::Template);
    assert!(matches!(kind(&tokens, 1), TokenKind::Identifier(_)));
    assert_eq!(kind(&tokens, 2), TokenKind::Text);
    assert!(matches!(kind(&tokens, 3), TokenKind::Identifier(_)));
    assert_eq!(tokens.identifier_table[0], "template_name");
    assert_eq!(tokens.identifier_table[1], "text2");
}

#[test]
fn records_integer_and_float_ranges() {
    let tokens = lexer::lex_all("42 3.14", "numbers").unwrap();
    assert_eq!(&kinds(&tokens)[..2], &[TokenKind::Int, TokenKind::Float]);
    assert_eq!(lexeme(&tokens, 0), "42");
    assert_eq!(lexeme(&tokens, 1), "3.14");
    assert_eq!(tokens.tokens[0].range, 0..2);
    assert_eq!(tokens.tokens[1].range, 3..7);
}

#[test]
fn skips_line_and_block_comments() {
    let tokens = lexer::lex_all("let // hidden\nconst /* hidden */ name", "comments").unwrap();
    assert_eq!(
        kinds(&tokens),
        vec![
            TokenKind::Let,
            TokenKind::Const,
            TokenKind::Identifier(0),
            TokenKind::Eof
        ]
    );
    assert_eq!(tokens.identifier_table[0], "name");
}

#[test]
fn tracks_token_location_after_newline() {
    let tokens = lexer::lex_all("let\nname", "locations").unwrap();
    assert!(matches!(kind(&tokens, 1), TokenKind::Identifier(_)));
    assert_eq!(tokens.tokens[1].line, 2);
    assert_eq!(tokens.tokens[1].col, 1);
}
