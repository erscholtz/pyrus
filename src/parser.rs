mod cursor;
mod parse;
mod parser;

use crate::{
    diagnostic::{DiagnosticManager, SyntaxError},
    lexer::{TokenKind, TokenStream},
    parser::{cursor::Cursor, parse::Parse},
};

pub struct Parser {
    pub file: String,
    pub cursor: Cursor,
    pub errors: Vec<SyntaxError>,
    pub trace_enabled: bool,
}

impl Parser {
    // flags

    pub fn enable_tracing(mut self) -> Self {
        self.trace_enabled = true;
        self.cursor.enable_tracing();
        self
    }

    // work

    fn trace(&self, event: &str) {
        if self.trace_enabled {
            return;
        }

        eprintln!(
            "[parser:{}:{event}] tok={:?} text={:?} line={} col={}",
            self.cursor.trace_context(),
            self.cursor.cur_tok(),
            self.cursor.cur_text(),
            self.cursor.cur_line(),
            self.cursor.cur_col(),
        );
    }

    pub fn new(toks: TokenStream) -> Self {
        Self {
            file: toks.file.clone(),
            errors: Vec::new(),
            cursor: Cursor::new(toks),
            trace_enabled: false,
        }
    }

    pub fn gather_errors(mut self, dm: &mut DiagnosticManager) -> Self {
        for error in self.errors.drain(..) {
            dm.push(error);
        }
        self
    }

    /// Entry: parse any T that implements Parse
    pub fn parse<T: Parse>(&mut self) -> Result<T, Vec<SyntaxError>> {
        self.trace("parse:start");
        let result = match T::parse(self) {
            Ok(result) => result,
            Err(err) => {
                self.errors.push(err);
                self.trace("parse:error");
                return Err(std::mem::take(&mut self.errors)); // FIX this is wrong I think
            }
        };

        if self.errors.is_empty() {
            self.trace("parse:ok");
            Ok(result)
        } else {
            self.trace("parse:errors");
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Error recovery helper
    pub fn synchronize(&mut self, delimiters: &[TokenKind]) {
        if self.trace_enabled {
            eprintln!(
                "[parser:{}:sync:start] delimiters={delimiters:?}",
                self.cursor.trace_context()
            );
        }
        // Skip until we hit a delimiter, add to errors
        while !self.cursor.check(TokenKind::Eof) {
            let token = self.cursor.advance();
            if delimiters.contains(&token) {
                self.trace("sync:hit-delimiter");
                break;
            }
        }
    }

    pub fn parse_until<T: Parse>(&mut self, end: TokenKind) -> Result<Vec<T>, Vec<SyntaxError>> {
        if self.trace_enabled {
            eprintln!(
                "[parser:{}:parse_until:start] end={end:?}",
                self.cursor.trace_context()
            );
        }
        let mut result = Vec::new();
        // NOTE: Caller is responsible for positioning cursor at first token
        while !self.cursor.check(end) && !self.cursor.check(TokenKind::Eof) {
            self.trace("parse_until:item");
            let parsed = match T::parse(self) {
                Ok(parsed) => parsed,
                Err(err) => {
                    self.errors.push(err);
                    self.trace("parse_until:error-skip");
                    self.cursor.advance(); // Skip the problematic token
                    continue;
                }
            };
            result.push(parsed);
        }
        if self.cursor.check(end) {
            if self.trace_enabled {
                eprintln!(
                    "[parser:{}:parse_until:consume_end] end={end:?} tok={:?} text={:?} at line={} col={}",
                    self.cursor.trace_context(),
                    self.cursor.cur_tok(),
                    self.cursor.cur_text(),
                    self.cursor.cur_line(),
                    self.cursor.cur_col(),
                );
            }
        }
        if self.errors.is_empty() {
            self.trace("parse_until:ok");
            Ok(result)
        } else {
            self.trace("parse_until:errors");
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Parses all items until the given condition is no longer true.
    pub fn parse_all<T: Parse, F>(&mut self, should_continue: F) -> Result<Vec<T>, Vec<SyntaxError>>
    where
        F: Fn(&mut Self) -> bool,
    {
        self.trace("parse_all:start");
        let mut items = Vec::new();
        while should_continue(self) {
            self.trace("parse_all:item");
            let result = match T::parse(self) {
                Ok(parsed) => parsed,
                Err(err) => {
                    self.errors.push(err);
                    self.trace("parse_all:error-sync");
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
            self.trace("parse_all:ok");
            Ok(items)
        } else {
            self.trace("parse_all:errors");
            Err(std::mem::take(&mut self.errors)) // Why?
        }
    }

    /// Parses all items until the given condition is no longer true, and then splits the result on the given delimiter.
    pub fn parse_split_on<T: Parse, FEnd, FDelim>(
        &mut self,
        end: FEnd,
        deliminer: FDelim,
        starts_with_delimiter: Option<TokenKind>,
    ) -> Result<Vec<T>, Vec<SyntaxError>>
    where
        FEnd: Fn(&mut Self) -> bool,
        FDelim: Fn(&mut Self) -> bool,
    {
        self.trace("parse_split_on:start");
        if let Some(delim) = starts_with_delimiter {
            self.trace("parse_split_on:delimiter");
            match self.cursor.expect(delim) {
                Ok(_) => {}
                Err(err) => {
                    return Err(vec![err]);
                }
            }
        }

        let mut items = Vec::new();
        while !end(self) && !self.cursor.check(TokenKind::Eof) {
            self.trace("parse_split_on:item");
            if deliminer(self) {
                self.trace("parse_split_on:delimiter");
                self.cursor.advance();
                continue;
            }
            let result = match T::parse(self) {
                Ok(parsed) => parsed,
                Err(err) => {
                    self.errors.push(err);
                    self.trace("parse_split_on:error-sync");
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
            self.trace("parse_split_on:ok");
            Ok(items)
        } else {
            self.trace("parse_split_on:errors");
            Err(std::mem::take(&mut self.errors)) // Why?
        }
    }
}
