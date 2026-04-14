use crate::lexer::{TokenKind, TokenStream};
use crate::parser::cursor::Cursor;
use crate::parser::parse::Parse;
use crate::parser::parser_err::ParseError;

pub struct Parser {
    pub file: String,
    pub cursor: Cursor,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(toks: TokenStream) -> Self {
        Self {
            file: toks.file.clone(),
            errors: Vec::new(),
            cursor: Cursor::new(toks),
        }
    }
    /// Entry: parse any T that implements Parse
    pub fn parse<T: Parse>(&mut self) -> Result<T, Vec<ParseError>> {
        let result = match T::parse(self) {
            Ok(result) => result,
            Err(err) => {
                self.errors.push(err);
                return Err(std::mem::take(&mut self.errors)); // FIX this is wrong I think
            }
        };

        if self.errors.is_empty() {
            Ok(result)
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Error recovery helper
    pub fn synchronize(&mut self, delimiters: &[TokenKind]) {
        // Skip until we hit a delimiter, add to errors
        while let token = self.cursor.advance() {
            // I hate the loop keyword
            if delimiters.contains(&token) {
                break;
            }
            self.errors.push(ParseError::new(
                format!("Unexpected token {:?}", token),
                self.cursor.location(),
            ));
        }
    }

    pub fn parse_until<T: Parse>(&mut self, end: TokenKind) -> Result<Vec<T>, Vec<ParseError>> {
        let mut result = Vec::new();
        // Note: Caller is responsible for positioning cursor at first token
        while !self.cursor.check(end) && !self.cursor.check(TokenKind::Eof) {
            let parsed = match T::parse(self) {
                Ok(parsed) => parsed,
                Err(err) => {
                    self.errors.push(err);
                    self.cursor.advance(); // Skip the problematic token
                    continue;
                }
            };
            result.push(parsed);
        }
        // Consume the end token if present TODO do we want this? hides exiting block
        if self.cursor.check(end) {
            self.cursor.advance();
        }
        if self.errors.is_empty() {
            Ok(result)
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    pub fn parse_all<T: Parse, F>(&mut self, should_continue: F) -> Result<Vec<T>, Vec<ParseError>>
    where
        F: Fn(&mut Self) -> bool,
    {
        let mut items = Vec::new();
        while should_continue(self) {
            let result = match T::parse(self) {
                Ok(parsed) => parsed,
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize(&[
                        TokenKind::Comma,
                        TokenKind::RightBrace,
                        TokenKind::RightParen,
                    ]);
                    continue;
                }
            };

            items.push(result);
        }

        if self.errors.is_empty() {
            Ok(items)
        } else {
            Err(std::mem::take(&mut self.errors)) // Why?
        }
    }
}
