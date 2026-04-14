#[derive(Debug, Clone)]
pub struct StringEntry {
    pub content: String,
    pub has_interpolation: bool,
}

#[repr(u8)]
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
    Bang,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equals,
    Dollarsign,
    Hash,
    Greater,
    Less,

    // Literals
    Identifier,
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
