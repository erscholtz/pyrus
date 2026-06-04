// ! parser token traversal

use crate::{
    diagnostic::{SourceLocation, SyntaxError},
    lexer::{
        tokens::{StringEntry, TokenKind},
        Token, TokenStream,
    },
};

pub struct Cursor {
    tokens: TokenStream,
    pos: usize,
    trace_enabled: bool,
    trace_context: String,
}

impl Cursor {
    pub fn set_trace_context<S: Into<String>>(&mut self, context: S) {
        self.trace_context = context.into();
    }

    pub fn trace_context(&self) -> &str {
        &self.trace_context
    }

    pub fn enable_tracing(&mut self) -> bool {
        self.trace_enabled = true;
        self.trace_enabled
    }

    fn trace(&self, event: &str) {
        if !self.trace_enabled {
            return;
        }

        eprintln!(
            "[parse:{}:{event}] pos={} tok={:?} text={:?} line={} col={}",
            self.trace_context,
            self.pos,
            self.cur_tok(),
            self.cur_text(),
            self.cur_line(),
            self.cur_col(),
        );
    }

    pub fn new(toks: TokenStream) -> Self {
        Self {
            tokens: toks,
            pos: 0,
            trace_enabled: false,
            trace_context: "main".to_string(),
        }
    }

    fn cur_token(&self) -> &Token {
        self.tokens
            .tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.tokens.last().unwrap()) // EOF is always last
    }

    pub fn cur_tok(&self) -> &TokenKind {
        &self.cur_token().kind
    }

    pub fn cur_text(&self) -> &str {
        match self.cur_tok() {
            TokenKind::Identifier(idx) => self.get_identifier(*idx).unwrap_or(""),
            TokenKind::Int | TokenKind::Float => self
                .cur_range()
                .and_then(|range| self.tokens.source.get(range))
                .unwrap_or(""),
            _ => "",
        }
    }

    pub fn cur_range(&self) -> Option<std::ops::Range<usize>> {
        self.tokens
            .tokens
            .get(self.pos)
            .map(|token| token.range.clone())
    }

    pub fn source(&self) -> &str {
        &self.tokens.source
    }

    pub fn cur_line(&self) -> usize {
        self.cur_token().line
    }

    pub fn cur_col(&self) -> usize {
        self.cur_token().col
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(self.cur_line(), self.cur_col(), self.tokens.file.clone())
    }

    pub fn advance(&mut self) -> &TokenKind {
        self.trace("advance:before");
        if self.pos < self.tokens.tokens.len() {
            self.pos += 1;
        }
        self.trace("advance:after");
        self.cur_tok()
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        let token = self.cur_tok();
        token == &kind
    }

    pub fn check_identifier(&self) -> bool {
        matches!(self.cur_tok(), TokenKind::Identifier(_))
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<TokenKind, SyntaxError> {
        if self.trace_enabled {
            eprintln!(
                "[parse:{}:expect] want={:?} have={:?} text={:?} line={} col={}",
                self.trace_context,
                kind,
                self.cur_tok(),
                self.cur_text(),
                self.cur_line(),
                self.cur_col(),
            );
        }
        if self.check(kind.clone()) {
            self.advance();
            Ok(kind)
        } else {
            Err(SyntaxError::UnexpectedToken {
                location: self.location(),
                expected: vec![kind],
                found: self.cur_tok().clone(),
            })
        }
    }

    pub fn expect_identifier(&mut self) -> Result<String, SyntaxError> {
        let TokenKind::Identifier(name) = self.cur_tok() else {
            return Err(SyntaxError::UnexpectedToken {
                location: self.location(),
                expected: vec![TokenKind::Identifier(0)],
                found: self.cur_tok().clone(),
            });
        };

        let name = self.get_identifier(*name).unwrap_or("").to_string();
        self.advance();
        Ok(name)
    }

    // backtracking

    pub fn checkpoint(&self) -> usize {
        self.trace("checkpoint");
        self.pos
    }

    pub fn restore(&mut self, checkpoint: usize) {
        if self.trace_enabled {
            eprintln!(
                "[parse:{}:restore] from={} to={} current_tok={:?}",
                self.trace_context,
                self.pos,
                checkpoint,
                self.cur_tok(),
            );
        }
        self.pos = checkpoint;
        self.trace("restore:after");
    }

    // lookahead

    pub fn peek_tok(&self) -> Option<&TokenKind> {
        self.tokens
            .tokens
            .get(self.pos + 1)
            .map(|token| &token.kind)
    }

    pub fn peek_text(&self) -> Option<&str> {
        match self.peek_tok()? {
            TokenKind::Identifier(idx) => self.get_identifier(*idx),
            _ => Some(""),
        }
    }

    // string_table specific

    pub fn get_identifier(&self, idx: usize) -> Option<&str> {
        self.tokens.identifier_table.get(idx).map(String::as_str)
    }

    pub fn get_string(&self, idx: usize) -> Option<&StringEntry> {
        self.tokens.string_table.get(idx)
    }

    pub fn get_string_entry(&self, idx: u32) -> Option<&StringEntry> {
        self.tokens.string_table.get(idx as usize)
    }
}

