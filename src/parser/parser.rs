use crate::{
    lexer::{TokenKind, TokenStream},
    parser::{cursor::Cursor, parse::Parse, parser_err::ParseError},
};

pub struct Parser {
    pub file: String,
    pub cursor: Cursor,
    pub errors: Vec<ParseError>,
    pub trace_enabled: bool,
    pub stop_on_error: bool,
}

impl Parser {
    // flags

    pub fn enable_tracing(mut self) -> Self {
        self.trace_enabled = true;
        self.cursor.enable_tracing();
        self
    }

    pub fn continue_on_error(mut self) -> Self {
        self.stop_on_error = false;
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
            stop_on_error: true,
        }
    }
    /// Entry: parse any T that implements Parse
    pub fn parse<T: Parse>(&mut self) -> Result<T, Vec<ParseError>> {
        self.trace("parse:start");
        let result = match T::parse(self) {
            Ok(result) => result,
            Err(err) => {
                if self.stop_on_error {
                    // FIX make this a bit better and return the first error the parser faces
                    return Err(vec![err]);
                }
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
        while let token = self.cursor.advance() {
            // I hate the loop keyword
            if delimiters.contains(&token) {
                self.trace("sync:hit-delimiter");
                break;
            }
            self.errors.push(ParseError::new(
                format!("Unexpected token {:?}", token),
                self.cursor.location(),
            ));
        }
    }

    pub fn parse_until<T: Parse>(&mut self, end: TokenKind) -> Result<Vec<T>, Vec<ParseError>> {
        if self.trace_enabled {
            eprintln!(
                "[parser:{}:parse_until:start] end={end:?}",
                self.cursor.trace_context()
            );
        }
        let mut result = Vec::new();
        // Note: Caller is responsible for positioning cursor at first token
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
        // Consume the end token if present TODO do we want this? hides exiting block
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
            self.cursor.advance();
        }
        if self.errors.is_empty() {
            self.trace("parse_until:ok");
            Ok(result)
        } else {
            self.trace("parse_until:errors");
            Err(std::mem::take(&mut self.errors))
        }
    }

    pub fn parse_all<T: Parse, F>(&mut self, should_continue: F) -> Result<Vec<T>, Vec<ParseError>>
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
}
