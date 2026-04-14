// ! parser token traversal

use super::parser_err::ParseError;
use crate::{
    diagnostic::SourceLocation,
    lexer::tokens::StringEntry,
    lexer::{TokenKind, TokenStream},
};

pub struct Cursor {
    tokens: TokenStream,
    pos: usize,
}

impl Cursor {
    pub fn new(toks: TokenStream) -> Self {
        Self {
            tokens: toks,
            pos: 0,
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
        if self.pos < self.tokens.kinds.len() {
            self.pos += 1;
        }
        self.cur_tok()
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        let token = self.cur_tok();
        token == &kind
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<TokenKind, ParseError> {
        if self.check(kind) {
            self.advance();
            Ok(kind)
        } else {
            Err(ParseError::new(
                format!("Expected {:?}, found {:?}", kind, self.cur_tok()),
                self.location(),
            ))
        }
    }

    // backtracking

    pub fn checkpoint(&self) -> usize {
        self.pos
    }

    pub fn restore(&mut self, checkpoint: usize) {
        self.pos = checkpoint;
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
