mod cursor;

use crate::{
    diagnostic::CompilerDiagnostic,
    tokens::{Token, TokenKind},
};
use cursor::{Cursor, Mark};

static SYMBOL_LOOKUP_TABLE: [Option<TokenKind>; 256] = {
    let mut t = [const { None }; 256];

    t[b'(' as usize] = Some(TokenKind::LeftParen);
    t[b')' as usize] = Some(TokenKind::RightParen);
    t[b'{' as usize] = Some(TokenKind::LeftBrace);
    t[b'}' as usize] = Some(TokenKind::RightBrace);
    t[b'[' as usize] = Some(TokenKind::LeftBracket);
    t[b']' as usize] = Some(TokenKind::RightBracket);
    t[b',' as usize] = Some(TokenKind::Comma);
    t[b'.' as usize] = Some(TokenKind::Dot);
    t[b';' as usize] = Some(TokenKind::Semicolon);
    t[b':' as usize] = Some(TokenKind::Colon);
    t[b'+' as usize] = Some(TokenKind::Plus);
    t[b'-' as usize] = Some(TokenKind::Dash);
    t[b'*' as usize] = Some(TokenKind::Star);
    t[b'/' as usize] = Some(TokenKind::Slash);
    t[b'%' as usize] = Some(TokenKind::Percent);
    t[b'=' as usize] = Some(TokenKind::Assign);
    t[b'$' as usize] = Some(TokenKind::DollarSign);
    t[b'#' as usize] = Some(TokenKind::Hash);
    t[b'!' as usize] = Some(TokenKind::Bang);
    t[b'?' as usize] = Some(TokenKind::Question);
    t[b'@' as usize] = Some(TokenKind::At);
    t[b'`' as usize] = Some(TokenKind::Backtick);
    t[b'\\' as usize] = Some(TokenKind::Backslash);
    t[b'>' as usize] = Some(TokenKind::Greater);
    t[b'<' as usize] = Some(TokenKind::Less);
    t[b'|' as usize] = Some(TokenKind::Pipe);
    t[b'~' as usize] = Some(TokenKind::Tilde);

    t
};

/// Lexer stuct for generating tokens
///
/// contains a cursor holding the current position in the
pub struct Lexer {
    cursor: Cursor,
    lookahead: Vec<Token>, // NOTE currently this is only used for
                           // peek_next_significant() but think it could have
                           // other uses
}

impl Lexer {
    /// Creates a new lexer with the given file and source
    pub fn new(file: String, src: String) -> Self {
        Self {
            cursor: Cursor::new(file, src),
            lookahead: Vec::new(),
        }
    }

    /// Peeks at current token (token pull() would return)
    pub fn peek(&mut self) -> Result<Token, CompilerDiagnostic> {
        if self.lookahead.is_empty() {
            let tok = self.lex_token()?;
            self.lookahead.push(tok);
        }

        Ok(self.lookahead.first().unwrap().to_owned())
    }

    /// Peeks at the next token without advancing the cursor
    pub fn peek_next(&mut self) -> Result<Token, CompilerDiagnostic> {
        if self.lookahead.is_empty() {
            let first = self.lex_token()?;
            self.lookahead.push(first);
            let second = self.lex_token()?;
            self.lookahead.push(second);
        } else if self.lookahead.len() < 2 {
            let tok = self.lex_token()?;
            self.lookahead.push(tok);
        }

        Ok(self.lookahead.get(1).unwrap().to_owned())
    }

    /// Peeks at next non whitespace token
    pub fn peek_next_significant(
        &mut self,
    ) -> Result<Token, CompilerDiagnostic> {
        if self.lookahead.len() < 2 {
            self.peek_next()?; // buffering side-effect
        }
        for i in 1..self.lookahead.len() {
            let tok = &self.lookahead[i];
            if tok.kind != TokenKind::Whitespace {
                return Ok(tok.to_owned());
            }
        }
        loop {
            let tok = self.lex_token()?;
            self.lookahead.push(tok.clone());
            if tok.kind == TokenKind::Eof {
                return Ok(tok);
            }
            if tok.kind != TokenKind::Whitespace {
                return Ok(tok);
            }
        }
    }

    /// Pulls the next token from the lexer, if there is something in the
    /// lookahead buffer it pulls from there first
    pub fn pull(&mut self) -> Result<Token, CompilerDiagnostic> {
        if self.lookahead.is_empty() {
            self.lex_token()
        } else {
            Ok(self.lookahead.remove(0))
        }
    }

    /// Returns the exact source covered by a token.
    pub fn text(&self, token: &Token) -> Option<&str> {
        self.cursor.src.get(token.range.clone())
    }

    fn lex_token(&mut self) -> Result<Token, CompilerDiagnostic> {
        let start = self.cursor.mark();
        let Some(byte) = self.cursor.peek() else {
            return Ok(self.token(TokenKind::Eof, start));
        };

        match byte {
            byte if byte.is_ascii_alphabetic() || byte == b'_' => {
                self.lex_identifier(start)
            }
            byte if byte.is_ascii_digit() => self.lex_number(start),
            b' ' | b'\t' => self.lex_whitespace(start),
            b'\n' | b'\r' => {
                self.cursor.advance()?;
                Ok(self.token(TokenKind::Newline, start))
            }
            b'/' if self.cursor.peek_next() == Some(b'/')
                && self
                    .cursor
                    .peek_previous()
                    .is_none_or(|byte| byte.is_ascii_whitespace()) =>
            {
                self.lex_comment(start)
            }
            _ => {
                if let Some(kind) = SYMBOL_LOOKUP_TABLE[byte as usize] {
                    self.cursor.advance()?;
                    Ok(self.token(kind, start))
                } else {
                    self.lex_text_fragment(start)
                }
            }
        }
    }

    fn lex_number(&mut self, start: Mark) -> Result<Token, CompilerDiagnostic> {
        while self.cursor.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.cursor.advance()?;
        }
        Ok(self.token(TokenKind::Number, start))
    }

    fn lex_identifier(
        &mut self,
        start: Mark,
    ) -> Result<Token, CompilerDiagnostic> {
        self.cursor.advance()?;
        while self.cursor.peek().is_some_and(|byte| {
            byte.is_ascii_alphabetic() || byte == b'_' || byte.is_ascii_digit()
        }) {
            self.cursor.advance()?;
        }

        Ok(self.token(TokenKind::Identifier, start))
    }

    fn lex_whitespace(
        &mut self,
        start: Mark,
    ) -> Result<Token, CompilerDiagnostic> {
        while self
            .cursor
            .peek()
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            self.cursor.advance()?;
        }
        Ok(self.token(TokenKind::Whitespace, start))
    }

    fn lex_comment(
        &mut self,
        start: Mark,
    ) -> Result<Token, CompilerDiagnostic> {
        while self
            .cursor
            .peek()
            .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
        {
            self.cursor.advance()?;
        }
        Ok(self.token(TokenKind::LineComment, start))
    }

    fn lex_text_fragment(
        &mut self,
        start: Mark,
    ) -> Result<Token, CompilerDiagnostic> {
        while self.cursor.peek_char().is_some_and(|ch| {
            if !ch.is_ascii() {
                return true;
            }

            let byte = ch as u8;
            !byte.is_ascii_alphanumeric()
                && byte != b'_'
                && !matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
                && SYMBOL_LOOKUP_TABLE[byte as usize].is_none()
        }) {
            self.cursor.advance()?;
        }

        Ok(self.token(TokenKind::TextFragment, start))
    }

    fn token(&self, kind: TokenKind, start: Mark) -> Token {
        Token {
            kind,
            range: start.offset..self.cursor.offset,
            line: start.line,
            col: start.col,
        }
    }
}
