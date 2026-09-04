mod parse;

pub use parse::Parse;

use crate::{
    diagnostic::{CompilerDiagnostic, SourceLocation, Span, SyntaxError},
    lexer::Lexer,
    tokens::{Token, TokenKind},
};

/// Parser state shared by the grammar-specific parse modules.
pub struct Parser {
    file: String,
    lexer: Lexer,
    current: Token,
}

impl Parser {
    pub fn new(
        file: String,
        mut lexer: Lexer,
    ) -> Result<Self, CompilerDiagnostic> {
        let current = lexer.pull()?;
        Ok(Self {
            file,
            lexer,
            current,
        })
    }

    pub fn parse<T: Parse>(&mut self) -> Result<T, CompilerDiagnostic> {
        T::parse(self)
    }

    /// Returns the current token kind.
    pub(crate) fn current_kind(&self) -> TokenKind {
        self.current.kind
    }

    /// Returns the text of the current token.
    pub(crate) fn current_text(&self) -> &str {
        self.lexer.text(&self.current).unwrap_or("")
    }

    /// Returns the location of the current token.
    pub(crate) fn location(&self) -> SourceLocation {
        SourceLocation::new(
            self.current.line,
            self.current.col,
            self.file.clone(),
        )
    }
    /// Returns `true` if the current token is of the given kind.
    pub(crate) fn at(&self, kind: TokenKind) -> bool {
        self.current_kind() == kind
    }

    /// Returns `true` if the current token is an identifier with the given text.
    pub(crate) fn at_keyword(&self, keyword: &str) -> bool {
        self.at(TokenKind::Identifier) && self.current_text() == keyword
    }

    /// Moves to the next token and returns it.
    pub(crate) fn next(&mut self) -> Result<Token, CompilerDiagnostic> {
        let next = self.lexer.pull()?;
        Ok(std::mem::replace(&mut self.current, next))
    }

    /// Consumes the current token if it is of the given kind, returning it.
    pub(crate) fn consume(
        &mut self,
        kind: TokenKind,
    ) -> Result<Token, CompilerDiagnostic> {
        if self.at(kind) {
            return self.next();
        }

        Err(SyntaxError::unexpected_token(
            vec![kind],
            self.current_kind(),
            self.location(),
        )
        .into())
    }

    pub(crate) fn expect_lexeme(
        &mut self,
        kind: TokenKind,
    ) -> Result<String, CompilerDiagnostic> {
        if !self.at(kind) {
            return Err(SyntaxError::unexpected_token(
                vec![kind],
                self.current_kind(),
                self.location(),
            )
            .into());
        }
        let text = self.consume_lexeme()?;
        Ok(text)
    }

    /// Consumes the current token if it is of the given kind, returning its
    /// text.
    pub(crate) fn consume_lexeme(
        &mut self,
    ) -> Result<String, CompilerDiagnostic> {
        let text = self.current_text().to_owned();
        self.next()?;
        Ok(text)
    }

    /// Consumes the current token if it is a keyword, returning it.
    pub(crate) fn consume_keyword(
        &mut self,
        keyword: &str,
    ) -> Result<Token, CompilerDiagnostic> {
        if self.at_keyword(keyword) {
            return self.next();
        }

        Err(SyntaxError::invalid_construct(
            keyword,
            format!(
                "expected keyword `{keyword}`, found `{}`",
                self.current_text()
            ),
            self.location(),
        )
        .into())
    }

    /// Skips over any trivia tokens (whitespace, comments) and returns `Ok(())`.
    pub(crate) fn skip_trivia(&mut self) -> Result<(), CompilerDiagnostic> {
        while matches!(
            self.current_kind(),
            TokenKind::Whitespace | TokenKind::Newline | TokenKind::LineComment
        ) {
            self.next()?;
        }
        Ok(())
    }
}
