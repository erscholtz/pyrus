use crate::lexer::TokenKind;

use super::{Diagnostic, Severity, SourceLocation};

/// Syntax errors raised during parsing.
#[derive(Debug, Clone)]
pub enum SyntaxError {
    /// Found a token that does not match the parser's expectation.
    UnexpectedToken {
        location: SourceLocation,
        expected: Vec<TokenKind>, // range of possible expected tokens
        found: TokenKind,
    },

    /// Reached EOF while still expecting more input.
    UnexpectedEof {
        location: SourceLocation,
        expected: String,
    },

    /// A required token was omitted.
    MissingToken {
        location: SourceLocation,
        expected: TokenKind,
    },

    /// A higher-level construct is malformed.
    InvalidConstruct {
        location: SourceLocation,
        construct: String,
        reason: String,
    },

    /// A delimiter such as `)` or `}` was never closed.
    UnterminatedDelimiter {
        location: SourceLocation,
        delimiter: String,
    },
}

impl SyntaxError {
    pub fn unexpected_token(
        expected: Vec<TokenKind>,
        found: TokenKind,
        location: SourceLocation,
    ) -> Self {
        Self::UnexpectedToken {
            location,
            expected,
            found,
        }
    }

    pub fn unexpected_eof(expected: impl Into<String>, location: SourceLocation) -> Self {
        Self::UnexpectedEof {
            location,
            expected: expected.into(),
        }
    }

    pub fn missing_token(expected: TokenKind, location: SourceLocation) -> Self {
        Self::MissingToken { location, expected }
    }

    pub fn invalid_construct(
        construct: impl Into<String>,
        reason: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::InvalidConstruct {
            location,
            construct: construct.into(),
            reason: reason.into(),
        }
    }

    pub fn unterminated_delimiter(delimiter: impl Into<String>, location: SourceLocation) -> Self {
        Self::UnterminatedDelimiter {
            location,
            delimiter: delimiter.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            SyntaxError::UnexpectedToken { .. } => "S0001",
            SyntaxError::UnexpectedEof { .. } => "S0002",
            SyntaxError::MissingToken { .. } => "S0003",
            SyntaxError::InvalidConstruct { .. } => "S0004",
            SyntaxError::UnterminatedDelimiter { .. } => "S0005",
        }
    }

    fn detailed_message(&self) -> String {
        match self {
            SyntaxError::UnexpectedToken {
                expected, found, ..
            } => {
                format!("expected `{expected:?}`, found `{found}`")
            }
            SyntaxError::UnexpectedEof { expected, .. } => {
                format!("expected {expected} before reaching end of file")
            }
            SyntaxError::MissingToken { expected, .. } => {
                format!("missing required token `{expected}`")
            }
            SyntaxError::InvalidConstruct {
                construct, reason, ..
            } => {
                format!("invalid `{construct}` construct: {reason}")
            }
            SyntaxError::UnterminatedDelimiter { delimiter, .. } => {
                format!("unterminated delimiter `{delimiter}`")
            }
        }
    }
}

impl Diagnostic for SyntaxError {
    fn message(&self) -> &str {
        match self {
            SyntaxError::UnexpectedToken { .. } => "unexpected token",
            SyntaxError::UnexpectedEof { .. } => "unexpected end of file",
            SyntaxError::MissingToken { .. } => "missing token",
            SyntaxError::InvalidConstruct { .. } => "invalid syntax construct",
            SyntaxError::UnterminatedDelimiter { .. } => "unterminated delimiter",
        }
    }

    fn location(&self) -> SourceLocation {
        match self {
            SyntaxError::UnexpectedToken { location, .. } => location.clone(),
            SyntaxError::UnexpectedEof { location, .. } => location.clone(),
            SyntaxError::MissingToken { location, .. } => location.clone(),
            SyntaxError::InvalidConstruct { location, .. } => location.clone(),
            SyntaxError::UnterminatedDelimiter { location, .. } => location.clone(),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} at {}: {}",
            self.code(),
            self.message(),
            self.location(),
            self.detailed_message()
        )
    }
}

impl std::error::Error for SyntaxError {}
