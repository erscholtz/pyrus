use super::parser_err::ParseError;
use crate::lexer::TokenKind;
use crate::parser::parser::Parser;

impl Parser {
    pub fn current_token_kind(&self) -> TokenKind {
        if self.idx >= self.toks.kinds.len() {
            return TokenKind::Eof;
        }
        self.toks.kinds[self.idx]
    }

    pub fn current_token_line(&self) -> usize {
        if self.idx >= self.toks.lines.len() {
            return self
                .toks
                .lines
                .last()
                .copied()
                .expect("lines vector empty but idx >= len");
        }
        self.toks.lines[self.idx]
    }

    pub fn current_token_col(&self) -> usize {
        if self.idx >= self.toks.cols.len() {
            return self
                .toks
                .cols
                .last()
                .copied()
                .expect("cols vector empty but idx >= len");
        }
        self.toks.cols[self.idx]
    }

    pub fn current_text(&self) -> String {
        if self.idx >= self.toks.ranges.len() {
            return String::new();
        }
        let range = &self.toks.ranges[self.idx];
        self.toks.source[range.start..range.end].to_string()
    }

    pub fn advance(&mut self) -> TokenKind {
        if self.idx < self.toks.kinds.len() {
            self.idx += 1;
        }
        // Skip whitespace tokens after advancing
        while self.idx < self.toks.kinds.len() && self.toks.kinds[self.idx] == TokenKind::Whitespace
        {
            self.idx += 1;
        }
        self.toks.kinds[self.idx - 1]
    }

    pub fn expect(&mut self, kind: TokenKind) -> TokenKind {
        // Skip whitespace tokens before checking
        while self.current_token_kind() == TokenKind::Whitespace {
            self.advance();
        }
        if self.current_token_kind() == kind {
            return self.advance().clone();
        }
        self.errors.push(ParseError::new(
            format!(
                "Parse error: expected {:?} but found {:?} at {}:{}",
                kind,
                self.current_token_kind(),
                self.current_token_line(),
                self.current_token_col()
            ),
            self.current_token_line(),
            self.current_token_col(),
        ));
        self.advance()
    }

    pub fn match_kind(&mut self, kind: TokenKind) -> bool {
        // Skip whitespace tokens before checking
        while self.current_token_kind() == TokenKind::Whitespace {
            self.advance();
        }
        if self.current_token_kind() == kind {
            self.advance();
            return true;
        }
        false
    }

    pub fn peek(&self) -> Option<TokenKind> {
        if self.idx + 1 < self.toks.kinds.len() {
            Some(self.toks.kinds[self.idx + 1])
        } else {
            None
        }
    }

    /// If a block `{ ... }` follows, skip it including nested braces.
    pub fn skip_optional_block(&mut self) {
        // skip optional whitespace-free tokens; if next is LeftBrace, skip until matching RightBrace
        if self.idx < self.toks.kinds.len() && self.current_token_kind() == TokenKind::LeftBrace {
            let mut depth: i32 = 0;
            while self.idx < self.toks.kinds.len() {
                match self.current_token_kind() {
                    TokenKind::LeftBrace => {
                        depth += 1;
                    }
                    TokenKind::RightBrace => {
                        depth -= 1;
                        if depth <= 0 {
                            self.advance();
                            break;
                        }
                    }
                    TokenKind::Eof => break,
                    _ => {}
                }
                self.advance();
            }
        }
    }

    pub fn synchronize(&mut self, sync_tokens: &[TokenKind]) {
        while !sync_tokens.contains(&self.current_token_kind()) {
            self.advance();
        }
    }
}
