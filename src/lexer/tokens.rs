use std::fmt::Write as _;

#[derive(Debug, Clone)]
pub struct StringEntry {
    pub content: String,
    pub has_interpolation: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // Single-char symbols
    LeftParen,
    RightParen, // ()
    LeftBrace,
    RightBrace, // {}
    LeftBracket,
    RightBracket, // []
    Comma,
    Dot,
    At, // @
    Semicolon,
    Colon,
    Pipe, // |
    Bang, // !
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,    // =
    Equals,    // ==
    NotEquals, // !=
    Dollarsign,
    Hash,
    Greater,
    GreaterEquals, // >=
    Less,
    LessEquals, // <=

    // Literals
    Identifier(usize),
    Int,
    Float,
    String,
    StringLiteral(usize),

    // Document elements
    Text,
    Image,
    List,
    Table,
    Section,
    Link,
    Separator,

    // Keywords
    Template,
    Document,
    Style,
    Func,
    Children,
    Let,
    Const,
    If,
    Else,
    For,
    While,
    Return,

    // Whitespace
    Whitespace,

    // End
    Eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// token representing a specific or group of keywords. holds the kind of token
/// it is and position data.
///
/// NOTE: this is currently in AOS format instead of previous SOA format due to
/// wanting a pull configuration for the lexer simplifying logic
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub range: std::ops::Range<usize>,
    pub line: usize,
    pub col: usize,
}

/// token stream of the current source file.
#[derive(Debug)]
pub struct TokenStream {
    pub file: String,
    pub source: String,
    pub tokens: Vec<Token>,
    pub identifier_table: Vec<String>,
    pub string_table: Vec<StringEntry>,
}

impl TokenStream {
    pub fn new(file: String) -> Self {
        Self {
            file,
            source: String::new(),
            tokens: Vec::new(),
            identifier_table: Vec::new(),
            string_table: Vec::new(),
        }
    }

    pub fn debug_tokens(&self) -> String {
        let mut out = String::new();

        writeln!(&mut out, "tokens for {}", self.file).unwrap();
        writeln!(
            &mut out,
            "{:>4}  {:<24} {:>9}  {:<12} text",
            "idx", "kind", "line:col", "range"
        )
        .unwrap();

        for (idx, token) in self.tokens.iter().enumerate() {
            let kind = format!("{:?}", token.kind);
            let location = format!("{}:{}", token.line, token.col);
            let range = format!("{}..{}", token.range.start, token.range.end);

            writeln!(
                &mut out,
                "{idx:>4}  {kind:<24} {location:>9}  {range:<12} {}",
                self.debug_text(token)
            )
            .unwrap();
        }

        out
    }

    fn debug_text(&self, token: &Token) -> String {
        let text = match token.kind {
            TokenKind::Identifier(idx) => self
                .identifier_table
                .get(idx)
                .map(String::as_str)
                .unwrap_or("<missing identifier>"),
            TokenKind::StringLiteral(idx) => self
                .string_table
                .get(idx)
                .map(|entry| entry.content.as_str())
                .unwrap_or("<missing string>"),
            _ => self.source.get(token.range.clone()).unwrap_or(""),
        };

        let mut text = format!("{text:?}");
        if let TokenKind::StringLiteral(idx) = token.kind {
            if self
                .string_table
                .get(idx)
                .is_some_and(|entry| entry.has_interpolation)
            {
                text.push_str(" (interpolated)");
            }
        }

        text
    }
}
