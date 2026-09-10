#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // Single-char symbols
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Dot,          // .
    Semicolon,    // ;
    Colon,        // :
    Plus,         // +
    Dash,         // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Assign,       // =
    DollarSign,   // $
    Hash,         // #
    Bang,         // !
    Question,     // ?
    At,           // @
    Tilde,        // ~
    Greater,      // >
    Less,         // <
    Pipe,         // |
    Backtick,     // `
    Backslash,    // \
    Whitespace,
    Newline,

    // Multi-char tokens
    Identifier,
    Number,
    TextFragment, // catch for wierd text objects
    LineComment,

    // End of file
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub range: std::ops::Range<usize>,
    pub line: usize,
    pub col: usize,
}
