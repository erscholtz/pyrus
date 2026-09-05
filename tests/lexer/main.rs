use pyrus::{
    lexer::Lexer,
    tokens::{Token, TokenKind},
};

fn lex(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new("test.pyr".to_string(), source.to_string());
    let mut tokens = Vec::new();

    loop {
        let token = lexer.pull().expect("source should lex");
        let done = token.kind == TokenKind::Eof;
        tokens.push(token);
        if done {
            return tokens;
        }
    }
}

fn kinds(tokens: &[Token]) -> Vec<TokenKind> {
    tokens.iter().map(|token| token.kind).collect()
}

fn token_text<'a>(source: &'a str, token: &Token) -> &'a str {
    &source[token.range.clone()]
}

#[test]
fn empty_input_produces_stable_eof() {
    let mut lexer = Lexer::new("empty.pyr".to_string(), String::new());

    let first = lexer.pull().expect("empty input should lex");
    let second = lexer.pull().expect("pulling after EOF should remain valid");

    assert_eq!(first.kind, TokenKind::Eof);
    assert_eq!(first.range, 0..0);
    assert_eq!((first.line, first.col), (1, 1));
    assert_eq!(second, first);
}

#[test]
fn lexes_document_configuration_losslessly() {
    let source = "document {\n    type: A4\n    margin: 4\n}\n";
    let tokens = lex(source);

    assert_eq!(
        kinds(&tokens),
        vec![
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::LeftBrace,
            TokenKind::Newline,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Newline,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Whitespace,
            TokenKind::Number,
            TokenKind::Newline,
            TokenKind::RightBrace,
            TokenKind::Newline,
            TokenKind::Eof,
        ]
    );

    let reconstructed: String = tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof)
        .map(|token| token_text(source, token))
        .collect();
    assert_eq!(reconstructed, source);
}

#[test]
fn preserves_comments_and_their_terminating_newline() {
    let source =
        "// Experience section\n@separator {\n    title: Experience\n}";
    let tokens = lex(source);

    assert_eq!(tokens[0].kind, TokenKind::LineComment);
    assert_eq!(token_text(source, &tokens[0]), "// Experience section");
    assert_eq!(tokens[1].kind, TokenKind::Newline);
    assert_eq!(tokens[2].kind, TokenKind::At);
    assert_eq!((tokens[2].line, tokens[2].col), (2, 1));
}

#[test]
fn treats_crlf_as_one_logical_newline() {
    let source = "a\r\nb";
    let tokens = lex(source);

    assert_eq!(
        kinds(&tokens),
        vec![
            TokenKind::Identifier,
            TokenKind::Newline,
            TokenKind::Identifier,
            TokenKind::Eof,
        ]
    );
    assert_eq!(tokens[1].range, 1..3);
    assert_eq!((tokens[2].line, tokens[2].col), (2, 1));
    assert_eq!(tokens[2].range, 3..4);
}

#[test]
fn preserves_unicode_resume_content() {
    let source = "date: 2025–2026\ncontact: `github` · `email`";
    let tokens = lex(source);
    let text: Vec<&str> = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::TextFragment)
        .map(|token| token_text(source, token))
        .collect();

    assert_eq!(text, vec!["–", "·"]);

    let contact = tokens
        .iter()
        .find(|token| token_text(source, token) == "contact")
        .expect("contact identifier should be present");
    assert_eq!((contact.line, contact.col), (2, 1));

    let reconstructed: String = tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof)
        .map(|token| token_text(source, token))
        .collect();
    assert_eq!(reconstructed, source);
}

#[test]
fn distinguishes_urls_from_line_comments() {
    let source = "https://example.com // comment";
    let tokens = lex(source);

    assert_eq!(
        tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Slash)
            .count(),
        2
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|token| token.kind == TokenKind::LineComment)
            .count(),
        1
    );
}

#[test]
fn lexes_inline_formatting_and_escapes() {
    let source = r"**Pyrus** `Rust` \` \\ \{ \}";
    let tokens = lex(source);

    assert_eq!(
        tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Star)
            .count(),
        4
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Backtick)
            .count(),
        3
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Backslash)
            .count(),
        5
    );
}

#[test]
fn lexes_representative_language_surface() {
    let source = r#"document {
    type: A4
    margin: 4
}

elem entry {
    company,
    role,
    date,
    content,
}

layout entry {
    company < | > date
    role
    content

    company: md
    date: sm
}

@entry {
    company: ACME
    role: Software Engineer
    date: 2025–2026

    - Built **Pyrus** in `Rust`
    - Worked on low-level PDF generation
}
"#;
    let tokens = lex(source);

    assert_eq!(tokens.last().map(|token| token.kind), Some(TokenKind::Eof));
    assert!(tokens.iter().any(|token| token.kind == TokenKind::At));
    assert!(tokens.iter().any(|token| token.kind == TokenKind::Pipe));
    assert!(tokens.iter().any(|token| token.kind == TokenKind::Dash));
    assert!(
        tokens
            .iter()
            .any(|token| token.kind == TokenKind::TextFragment)
    );

    let reconstructed: String = tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof)
        .map(|token| token_text(source, token))
        .collect();
    assert_eq!(reconstructed, source);
}
