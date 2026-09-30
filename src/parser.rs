mod cursor;
mod lexer;
mod parse;
pub mod tokens;

pub use parse::Parse;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::Severity;
use crate::diagnostic::SourceLocation;
use crate::parser::lexer::Lexer;
use crate::parser::tokens::Token;
use crate::parser::tokens::TokenKind;

/// Parser state shared by the grammar-specific parse modules.
pub struct Parser {
    file: String,
    lexer: Lexer,
}

impl Parser {
    pub fn new(file: String, src: String) -> Result<Self, Diagnostic> {
        let lexer = Lexer::new(file.clone(), src);
        Ok(Self { file, lexer })
    }

    pub fn parse<T: Parse>(&mut self) -> Result<T, Diagnostic> {
        T::parse(self)
    }

    /// Returns the text of the current token.
    pub(crate) fn current_text(&mut self) -> Result<&str, Diagnostic> {
        let tok = self.lexer.peek()?;
        match self.lexer.text(&tok) {
            Some(text) => Ok(text),
            None => Err(Diagnostic {
                severity: Severity::Fatal,
                message: "invalid token source range".to_string(),
                location: self.location_of(&tok),
                span: None,
                help: None,
            }),
        }
    }

    /// Returns the location of the current token.
    pub(crate) fn location(&mut self) -> Result<SourceLocation, Diagnostic> {
        let tok = self.peek()?;
        Ok(SourceLocation::new(tok.line, tok.col, self.file.clone()))
    }

    fn location_of(&self, tok: &Token) -> SourceLocation {
        SourceLocation::new(tok.line, tok.col, self.file.clone())
    }

    /// Returns `true` if the current token is of the given kind.
    pub(crate) fn at(&mut self, kind: TokenKind) -> Result<bool, Diagnostic> {
        Ok(self.peek()?.kind == kind)
    }

    /// Returns `true` if the current token is an identifier with the given text.
    pub(crate) fn at_keyword(
        &mut self,
        keyword: &str,
    ) -> Result<bool, Diagnostic> {
        Ok(self.at(TokenKind::Identifier)? && self.current_text()? == keyword)
    }

    /// Returns the kind of the current token without consuming it.
    pub(crate) fn peek(&mut self) -> Result<Token, Diagnostic> {
        self.lexer.peek()
    }

    /// Returns the kind of the next token without consuming it.
    pub(crate) fn peek_next(&mut self) -> Result<Token, Diagnostic> {
        self.lexer.peek_next()
    }

    /// wrapper over `Lexer::peek_next_significant`, peeks ahead to next non
    /// whitespace token
    pub(crate) fn peek_next_significant(
        &mut self,
    ) -> Result<Token, Diagnostic> {
        self.lexer.peek_next_significant()
    }

    /// Moves to the next token and returns it.
    pub(crate) fn next(&mut self) -> Result<Token, Diagnostic> {
        self.lexer.pull()
    }

    /// Consumes the current token if it is of the given kind, returning it.
    pub(crate) fn consume(
        &mut self,
        kind: TokenKind,
    ) -> Result<Token, Diagnostic> {
        if !self.at(kind)? {
            return Err(Diagnostic {
                severity: Severity::Error,
                message: format!(
                    "unexpected token {}, expetected {}",
                    self.peek()?.kind,
                    kind
                ),
                location: self.location()?,
                span: None,
                help: None,
            });
        }
        self.next()
    }

    pub(crate) fn expect_lexeme(
        &mut self,
        kind: TokenKind,
    ) -> Result<String, Diagnostic> {
        if !self.at(kind)? {
            return Err(Diagnostic {
                severity: Severity::Error,
                message: format!(
                    "unexpected token {}, expected {}",
                    self.peek()?.kind,
                    kind
                ),
                location: self.location()?,
                span: None,
                help: None,
            });
        }
        let text = self.consume_lexeme()?;
        Ok(text)
    }

    /// Consumes the current token if it is of the given kind, returning its
    /// text.
    pub(crate) fn consume_lexeme(&mut self) -> Result<String, Diagnostic> {
        let text = self.current_text()?.to_owned();
        self.next()?;
        Ok(text)
    }

    /// Consumes the current token if it is a keyword, returning it.
    pub(crate) fn consume_keyword(
        &mut self,
        keyword: &str,
    ) -> Result<Token, Diagnostic> {
        if self.at_keyword(keyword)? {
            return self.next();
        }

        Err(Diagnostic {
            severity: Severity::Error,
            message: format!(
                "unexpected token {}, needed keyword {}",
                self.peek()?.kind,
                keyword
            ),
            location: self.location()?,
            span: None,
            help: None,
        })
    }

    /// Skips over any trivia tokens (whitespace, comments) and returns `Ok(())`.
    pub(crate) fn skip_trivia(&mut self) -> Result<(), Diagnostic> {
        while matches!(
            self.peek()?.kind,
            TokenKind::Whitespace | TokenKind::Newline | TokenKind::LineComment
        ) {
            self.next()?;
        }
        Ok(())
    }

    /// Skips over any inline trivia tokens (whitespace, comments) and returns
    /// `Ok(())`.
    pub(crate) fn skip_inline_trivia(&mut self) -> Result<(), Diagnostic> {
        while matches!(
            self.peek()?.kind,
            TokenKind::Whitespace | TokenKind::LineComment
        ) {
            self.next()?;
        }
        Ok(())
    }
}
