mod cursor;
mod lexer;
mod parse;
pub mod tokens;

pub use parse::Parse;

use crate::diagnostic::CompilerDiagnostic;
use crate::diagnostic::FatalError;
use crate::diagnostic::SourceLocation;
use crate::diagnostic::SyntaxError;
use crate::parser::lexer::Lexer;
use crate::parser::tokens::Token;
use crate::parser::tokens::TokenKind;

/// Parser state shared by the grammar-specific parse modules.
pub struct Parser {
    file: String,
    lexer: Lexer,
}

impl Parser {
    pub fn new(file: String, src: String) -> Result<Self, CompilerDiagnostic> {
        let lexer = Lexer::new(file.clone(), src);
        Ok(Self { file, lexer })
    }

    pub fn parse<T: Parse>(&mut self) -> Result<T, CompilerDiagnostic> {
        T::parse(self)
    }

    /// Returns the text of the current token.
    pub(crate) fn current_text(&mut self) -> Result<&str, CompilerDiagnostic> {
        let tok = self.lexer.peek()?;
        match self.lexer.text(&tok) {
            Some(text) => Ok(text),
            None => Err(CompilerDiagnostic::Fatal(
                FatalError::InternalCompilerError {
                    location: self.location_of(&tok),
                    phase: "parsing".to_string(),
                    details: format!(
                        "invalid token source range {:?} [parser::current_text()]",
                        tok.range
                    ),
                },
            )),
        }
    }

    /// Returns the location of the current token.
    pub(crate) fn location(
        &mut self,
    ) -> Result<SourceLocation, CompilerDiagnostic> {
        let tok = self.peek()?;
        Ok(SourceLocation::new(tok.line, tok.col, self.file.clone()))
    }

    fn location_of(&self, tok: &Token) -> SourceLocation {
        SourceLocation::new(tok.line, tok.col, self.file.clone())
    }

    /// Returns `true` if the current token is of the given kind.
    pub(crate) fn at(
        &mut self,
        kind: TokenKind,
    ) -> Result<bool, CompilerDiagnostic> {
        Ok(self.peek()?.kind == kind)
    }

    /// Returns `true` if the current token is an identifier with the given text.
    pub(crate) fn at_keyword(
        &mut self,
        keyword: &str,
    ) -> Result<bool, CompilerDiagnostic> {
        Ok(self.at(TokenKind::Identifier)? && self.current_text()? == keyword)
    }

    /// Returns the kind of the current token without consuming it.
    pub(crate) fn peek(&mut self) -> Result<Token, CompilerDiagnostic> {
        self.lexer.peek()
    }

    /// Returns the kind of the next token without consuming it.
    pub(crate) fn peek_next(&mut self) -> Result<Token, CompilerDiagnostic> {
        self.lexer.peek_next()
    }

    /// wrapper over `Lexer::peek_next_significant`, peeks ahead to next non
    /// whitespace token
    pub(crate) fn peek_next_significant(
        &mut self,
    ) -> Result<Token, CompilerDiagnostic> {
        self.lexer.peek_next_significant()
    }

    /// Moves to the next token and returns it.
    pub(crate) fn next(&mut self) -> Result<Token, CompilerDiagnostic> {
        self.lexer.pull()
    }

    /// Consumes the current token if it is of the given kind, returning it.
    pub(crate) fn consume(
        &mut self,
        kind: TokenKind,
    ) -> Result<Token, CompilerDiagnostic> {
        if !self.at(kind)? {
            return Err(SyntaxError::unexpected_token(
                vec![kind],
                self.peek()?.kind,
                self.location()?,
            )
            .into());
        }
        self.next()
    }

    pub(crate) fn expect_lexeme(
        &mut self,
        kind: TokenKind,
    ) -> Result<String, CompilerDiagnostic> {
        if !self.at(kind)? {
            return Err(SyntaxError::unexpected_token(
                vec![kind],
                self.peek()?.kind,
                self.location()?,
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
        let text = self.current_text()?.to_owned();
        self.next()?;
        Ok(text)
    }

    /// Consumes the current token if it is a keyword, returning it.
    pub(crate) fn consume_keyword(
        &mut self,
        keyword: &str,
    ) -> Result<Token, CompilerDiagnostic> {
        if self.at_keyword(keyword)? {
            return self.next();
        }

        Err(SyntaxError::invalid_construct(
            keyword,
            format!(
                "expected keyword `{keyword}`, found `{}`",
                self.current_text()?
            ),
            self.location()?,
        )
        .into())
    }

    /// Skips over any trivia tokens (whitespace, comments) and returns `Ok(())`.
    pub(crate) fn skip_trivia(&mut self) -> Result<(), CompilerDiagnostic> {
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
    pub(crate) fn skip_inline_trivia(
        &mut self,
    ) -> Result<(), CompilerDiagnostic> {
        while matches!(
            self.peek()?.kind,
            TokenKind::Whitespace | TokenKind::LineComment
        ) {
            self.next()?;
        }
        Ok(())
    }
}
