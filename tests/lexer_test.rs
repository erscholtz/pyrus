use std::fs;

use pyrus::lexer;

fn lexeme(ts: &lexer::TokenStream, idx: usize) -> &str {
    let r = &ts.ranges[idx];
    &ts.source[r.start..r.end]
}

#[test]
fn lexes_sample_file_tokens() {
    let data = fs::read_to_string("tests/input/lexer_test.ink").expect("read sample file");
    let tokens = lexer::lex(&data).expect("Lexing failed");

    // basic structure: first tokens should be `template` and `{`
    assert!(!tokens.kinds.is_empty());
    assert_eq!(tokens.kinds[0], lexer::TokenKind::Template);
    assert_eq!(tokens.kinds[1], lexer::TokenKind::LeftBrace);

    // there should be at least one string literal ("My Document")
    let string_indices: Vec<usize> = tokens
        .kinds
        .iter()
        .enumerate()
        .filter_map(|(i, k)| {
            if *k == lexer::TokenKind::StringLiteral {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    assert!(
        !string_indices.is_empty(),
        "expected at least one string literal"
    );

    // ensure one of the string literals contains the title text
    let found_title = string_indices
        .iter()
        .any(|&i| lexeme(&tokens, i).contains("My Document"));
    assert!(
        found_title,
        "expected a string literal containing 'My Document'"
    );

    // final token should be EOF
    assert_eq!(*tokens.kinds.last().unwrap(), lexer::TokenKind::Eof);
}
