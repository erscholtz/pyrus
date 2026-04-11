// ! parser token traversal

use super::parser_err::ParseError;
use crate::lexer::{TokenKind, TokenStream};

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
        &self.tokens.kinds[self.pos]
    }

    pub fn cur_text(&self) -> &str {
        let range = self.tokens.ranges[self.pos].clone();
        &self.tokens.source[range]
    }

    pub fn cur_line(&self) -> usize {
        self.tokens.lines[self.pos]
    }

    pub fn cur_col(&self) -> usize {
        self.tokens.cols[self.pos]
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        let token = self.cur_tok();
        token == &kind
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                format!("Expected {:?}, found {:?}", kind, self.cur_tok()),
                self.tokens.lines[self.pos],
                self.tokens.cols[self.pos],
                self.tokens.file.clone(),
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
}
