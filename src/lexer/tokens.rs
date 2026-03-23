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
    StringLiteral,
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
    Element,
    Let,
    Const,
    Var,
    If,
    Else,
    For,
    While,
    Return,

    // End
    Eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
