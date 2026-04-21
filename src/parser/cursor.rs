// ! parser token traversal

use crate::{
    diagnostic::{SourceLocation, SyntaxError},
    lexer::{TokenKind, TokenStream, tokens::StringEntry},
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
        if self.trace_enabled {
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

    pub fn cur_tok(&self) -> &TokenKind {
        self.tokens
            .kinds
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.kinds.last().unwrap()) // EOF is always last
    }

    pub fn cur_text(&self) -> &str {
        let range = self
            .tokens
            .ranges
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.ranges.last().unwrap()); // EOF is always last
        &self.tokens.source[range.clone()]
    }

    pub fn cur_range(&self) -> Option<std::ops::Range<usize>> {
        self.tokens.ranges.get(self.pos).cloned()
    }

    pub fn source(&self) -> &str {
        &self.tokens.source
    }

    pub fn cur_line(&self) -> usize {
        self.tokens.lines[self.pos]
    }

    pub fn cur_col(&self) -> usize {
        self.tokens.cols[self.pos]
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(
            self.tokens.lines[self.pos],
            self.tokens.cols[self.pos],
            self.tokens.file.clone(),
        )
    }

    pub fn advance(&mut self) -> &TokenKind {
        self.trace("advance:before");
        if self.pos < self.tokens.kinds.len() {
            self.pos += 1;
        }
        self.trace("advance:after");
        self.cur_tok()
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        let token = self.cur_tok();
        token == &kind
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
        if self.check(kind) {
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
        self.tokens.kinds.get(self.pos + 1)
    }

    pub fn peek_text(&self) -> Option<&str> {
        let range = self.tokens.ranges.get(self.pos + 1)?;
        self.tokens.source.get(range.clone())
    }

    // string_table specific

    pub fn get_string(&self, idx: usize) -> Option<&StringEntry> {
        self.tokens.string_table.get(idx)
    }

    pub fn get_string_entry(&self, idx: u32) -> Option<&StringEntry> {
        self.tokens.string_table.get(idx as usize)
    }
}
