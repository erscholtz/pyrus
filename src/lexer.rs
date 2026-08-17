mod lexer_cursor;
pub mod tokens;

use tokens::{StringEntry, TokenKind};
pub use tokens::{Token, TokenStream};

use crate::{
    diagnostic::{CompilerDiagnostic, SourceLocation, SyntaxError},
    lexer::lexer_cursor::{Cursor, Mark},
};

static KEYWORD_TABLE: phf::Map<&'static str, TokenKind> = phf::phf_map! {
    "template" => TokenKind::Template,
    "document" => TokenKind::Document,
    "style" => TokenKind::Style,
    "func" => TokenKind::Func,
    "children" => TokenKind::Children,
    "let" => TokenKind::Let,
    "const" => TokenKind::Const,
    "if" => TokenKind::If,
    "else" => TokenKind::Else,
    "for" => TokenKind::For,
    "while" => TokenKind::While,
    "return" => TokenKind::Return,
    "text" => TokenKind::Text,
    "image" => TokenKind::Image,
    "list" => TokenKind::List,
    "section" => TokenKind::Section,
    "table" => TokenKind::Table,
    "link" => TokenKind::Link,
    "separator" => TokenKind::Separator,
    "String" => TokenKind::String,
    "Int" => TokenKind::Int,
    "Float" => TokenKind::Float,
};

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
    t[b'-' as usize] = Some(TokenKind::Minus);
    t[b'*' as usize] = Some(TokenKind::Star);
    t[b'/' as usize] = Some(TokenKind::Slash);
    t[b'%' as usize] = Some(TokenKind::Percent);
    t[b'=' as usize] = Some(TokenKind::Assign);
    t[b'$' as usize] = Some(TokenKind::Dollarsign);
    t[b'#' as usize] = Some(TokenKind::Hash);
    t[b'!' as usize] = Some(TokenKind::Bang);
    t[b'>' as usize] = Some(TokenKind::Greater);
    t[b'<' as usize] = Some(TokenKind::Less);
    t[b'@' as usize] = Some(TokenKind::At);
    t[b'|' as usize] = Some(TokenKind::Pipe);

    t
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LexerMode {
    Normal,
    ElementInvokation, // @
    AwaitTextBodyOrAttrs,
    TextAttributes(usize),
    TextBody,
}

/// Lexer stuct for generating tokens
///
/// contains a cursor holding the current position in the file and a mode stack
/// as a state machine of the different types of tokens that need to be
/// generated
pub struct Lexer {
    cursor: Cursor,                 // info on where in source we are
    mode: Vec<LexerMode>,           // current mode of the lexer
    identifier_table: Vec<String>,  // variable names
    string_table: Vec<StringEntry>, // strings deduplication table
    queued_tokens: Vec<Token>,      // any trailing tokens like ], }, "
    pub debug: bool,
}

impl Lexer {
    pub fn new(file: String, src: String) -> Self {
        let lexer = Lexer {
            cursor: Cursor::new(file, src),
            mode: Vec::new(),
            identifier_table: Vec::new(),
            string_table: Vec::new(),
            queued_tokens: Vec::new(),
            debug: false,
        };
        lexer
    }

    pub fn enable_debug(&mut self) {
        self.debug = true;
    }

    /// eager lexing implementation
    pub fn lex_all(&mut self) -> Result<TokenStream, Vec<CompilerDiagnostic>> {
        let mut errors = Vec::new();
        let mut output = TokenStream::new(self.cursor.file.clone());
        output.source = self.cursor.src.clone();

        // right before lexing starts define normal state
        self.mode.push(LexerMode::Normal);
        loop {
            let token = match self.pull() {
                Ok(token) => token,
                Err(err) => {
                    errors.push(err);

                    if self.cursor.peek().is_none() {
                        break;
                    }
                    continue;
                }
            };

            let done = token.kind == TokenKind::Eof;
            output.tokens.push(token);

            // each file will end with a end of file token which is our sign to
            // exit
            if done {
                break;
            }
        }

        // sanity check that lexer mode is normal after lexing
        let last = self.mode.pop();
        debug_assert_eq!(
            last,
            Some(LexerMode::Normal),
            "lexer mode should be normal after lexing"
        );

        if !errors.is_empty() {
            return Err(errors);
        }
        output.identifier_table = std::mem::take(&mut self.identifier_table);
        output.string_table = std::mem::take(&mut self.string_table);
        if self.debug {
            eprintln!("{}", output.debug_tokens());
        }
        Ok(output)
    }

    /// state machine based pull request for the cursor to make the next token
    fn pull(&mut self) -> Result<Token, CompilerDiagnostic> {
        if let Some(token) = self.queued_tokens.pop() {
            return Ok(token);
        }

        let token = match self.current_mode() {
            LexerMode::Normal => {
                self.skip_trivia()?;
                self.lex_normal_token()
            }
            LexerMode::ElementInvokation => self.lex_element_invokation_token(),
            LexerMode::AwaitTextBodyOrAttrs | LexerMode::TextAttributes(_) => {
                self.skip_trivia()?;
                self.lex_text_transition_token()
            }
            LexerMode::TextBody => self.lex_text_body_token(),
        }?;

        Ok(token)
    }

    // NOTE: should this splitting be handled by the mode? that makes so much
    // sense with a mode stack
    fn lex_normal_token(&mut self) -> Result<Token, CompilerDiagnostic> {
        let start = self.cursor.mark();
        let Some(byte) = self.cursor.peek() else {
            return Ok(self.token(TokenKind::Eof, start));
        };

        if byte.is_ascii_alphabetic() || byte == b'_' {
            return self.lex_identifier_or_keyword(start);
        }

        if byte.is_ascii_digit() {
            return self.lex_number(start);
        }

        if byte == b'"' {
            return self.lex_quoted_string(start);
        }

        if let Some(kind) = self.two_char_operator_kind(byte) {
            self.cursor.advance()?;
            self.cursor.advance()?;
            return Ok(self.token(kind, start));
        }

        if let Some(kind) = SYMBOL_LOOKUP_TABLE[byte as usize] {
            self.cursor.advance()?;
            if kind == TokenKind::At {
                self.mode.push(LexerMode::ElementInvokation);
            }
            return Ok(self.token(kind, start));
        }

        self.cursor.advance()?;
        Err(SyntaxError::invalid_construct(
            "character",
            format!("unknown character: '{}'", byte as char),
            SourceLocation::new(
                start.line,
                start.col,
                self.cursor.file.clone(),
            ),
        )
        .into())
    }

    fn lex_element_invokation_token(
        &mut self,
    ) -> Result<Token, CompilerDiagnostic> {
        debug_assert_eq!(self.current_mode(), LexerMode::ElementInvokation);
        let start = self.cursor.mark();

        if self
            .cursor
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        {
            let token = self.lex_identifier_or_keyword(start)?;
            self.mode.pop();
            if token.kind == TokenKind::Text {
                self.mode.push(LexerMode::AwaitTextBodyOrAttrs);
            }
            return Ok(token);
        }

        self.mode.pop();
        Err(SyntaxError::invalid_construct(
            "element invocation",
            "expected an element name immediately after `@`",
            SourceLocation::new(
                start.line,
                start.col,
                self.cursor.file.clone(),
            ),
        )
        .into())
    }

    fn lex_text_transition_token(
        &mut self,
    ) -> Result<Token, CompilerDiagnostic> {
        let mode = self.current_mode();
        let start = self.cursor.mark();

        // TODO: this should only be called in AwaitTextBodyOrAttrs mode, should
        // assert!(mode == LexerMode::AwaitTextBodyOrAttrs);
        // but we don't have access to the mode here, so we can't assert,

        if mode == LexerMode::AwaitTextBodyOrAttrs
            && self.cursor.peek() == Some(b'[')
        {
            self.cursor.advance()?;
            self.mode.pop(); // remove await
            self.mode.push(LexerMode::TextBody);
            return Ok(self.token(TokenKind::LeftBracket, start));
        }

        let token = self.lex_normal_token()?;
        match (mode, token.kind) {
            (LexerMode::AwaitTextBodyOrAttrs, TokenKind::LeftParen) => {
                self.mode.push(LexerMode::TextAttributes(1));
            }
            (LexerMode::TextAttributes(depth), TokenKind::LeftParen) => {
                self.mode.push(LexerMode::TextAttributes(depth + 1));
            }
            (LexerMode::TextAttributes(depth), TokenKind::RightParen)
                if depth > 1 =>
            {
                self.mode.pop();
            }
            (LexerMode::TextAttributes(1), TokenKind::RightParen) => {
                self.mode.pop();
            }
            (_, TokenKind::Eof) => {
                self.reset_mode();
                return Err(SyntaxError::unexpected_eof(
                    "a text body",
                    SourceLocation::new(
                        start.line,
                        start.col,
                        self.cursor.file.clone(),
                    ),
                )
                .into());
            }
            (_, _) => {} // TODO
        };

        Ok(token)
    }

    fn lex_text_body_token(&mut self) -> Result<Token, CompilerDiagnostic> {
        let start = self.cursor.mark();
        let mut interpolation_depth = 0usize;

        while let Some(byte) = self.cursor.peek() {
            if byte == b'$' && self.cursor.peek_next() == Some(b'{') {
                interpolation_depth += 1;
                self.cursor.advance()?;
                self.cursor.advance()?;
                continue;
            }

            if byte == b'}' && interpolation_depth > 0 {
                interpolation_depth -= 1;
                self.cursor.advance()?;
                continue;
            }

            if byte == b']' && interpolation_depth == 0 {
                let content = self.cursor.src[start.offset..self.cursor.offset]
                    .to_string();
                let string_idx = self.push_string(content);
                let body_token =
                    self.token(TokenKind::StringLiteral(string_idx), start);

                let close_start = self.cursor.mark();
                self.cursor.advance()?;
                self.mode.pop();
                self.queued_tokens
                    .push(self.token(TokenKind::RightBracket, close_start));

                return Ok(body_token);
            }

            self.cursor.advance()?;
        }

        self.reset_mode();
        Err(SyntaxError::unterminated_delimiter(
            "]",
            SourceLocation::new(
                start.line,
                start.col,
                self.cursor.file.clone(),
            ),
        )
        .into())
    }

    fn lex_identifier_or_keyword(
        &mut self,
        start: Mark,
    ) -> Result<Token, CompilerDiagnostic> {
        self.cursor.advance()?;
        while self.cursor.peek().is_some_and(|byte| {
            byte.is_ascii_alphabetic() || byte == b'_' || byte.is_ascii_digit()
        }) {
            self.cursor.advance()?;
        }

        let text = &self.cursor.src[start.offset..self.cursor.offset];
        let kind = match KEYWORD_TABLE.get(text).copied() {
            Some(kind) => kind,
            None => {
                TokenKind::Identifier(self.push_identifier(text.to_string()))
            }
        };

        Ok(self.token(kind, start))
    }

    fn lex_number(&mut self, start: Mark) -> Result<Token, CompilerDiagnostic> {
        self.cursor.advance()?;
        while self.cursor.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.cursor.advance()?;
        }

        let mut kind = TokenKind::Int;
        if self.cursor.peek() == Some(b'.')
            && self
                .cursor
                .peek_next()
                .is_some_and(|byte| byte.is_ascii_digit())
        {
            kind = TokenKind::Float;
            self.cursor.advance()?;
            while self.cursor.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.cursor.advance()?;
            }
        }

        Ok(self.token(kind, start))
    }

    fn lex_quoted_string(
        &mut self,
        start: Mark,
    ) -> Result<Token, CompilerDiagnostic> {
        self.cursor.advance()?;
        let content_start = self.cursor.offset;
        let mut escaped = false;

        while let Some(byte) = self.cursor.peek() {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                let content = self.cursor.src
                    [content_start..self.cursor.offset]
                    .to_string();
                self.cursor.advance()?;
                let string_idx = self.push_string(content);
                return Ok(
                    self.token(TokenKind::StringLiteral(string_idx), start)
                );
            }

            self.cursor.advance()?;
        }

        Err(SyntaxError::unterminated_delimiter(
            "\"",
            SourceLocation::new(
                start.line,
                start.col,
                self.cursor.file.clone(),
            ),
        )
        .into())
    }

    fn skip_trivia(&mut self) -> Result<(), CompilerDiagnostic> {
        loop {
            match (self.cursor.peek(), self.cursor.peek_next()) {
                (Some(byte), _) if byte.is_ascii_whitespace() => {
                    self.cursor.advance()?
                }
                (Some(b'/'), Some(b'/')) => {
                    self.cursor.advance()?;
                    self.cursor.advance()?;
                    while self.cursor.peek().is_some_and(|byte| byte != b'\n') {
                        self.cursor.advance()?;
                    }
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn two_char_operator_kind(&self, byte: u8) -> Option<TokenKind> {
        match (byte, self.cursor.peek_next()) {
            (b'=', Some(b'=')) => Some(TokenKind::Equals),
            (b'!', Some(b'=')) => Some(TokenKind::NotEquals),
            (b'>', Some(b'=')) => Some(TokenKind::GreaterEquals),
            (b'<', Some(b'=')) => Some(TokenKind::LessEquals),
            _ => None,
        }
    }

    fn current_mode(&self) -> LexerMode {
        *self.mode.last().expect("lexer mode must not be empty")
    }

    fn reset_mode(&mut self) {
        self.mode.truncate(1);
        debug_assert_eq!(self.current_mode(), LexerMode::Normal);
    }

    // TODO can be squashed
    fn push_identifier(&mut self, text: String) -> usize {
        let idx = self.identifier_table.len();
        self.identifier_table.push(text);
        idx
    }

    // TODO can be squashed, also could know that the if interp exists based on
    // the string lexing function with a simple if check
    fn push_string(&mut self, content: String) -> usize {
        let idx = self.string_table.len();
        self.string_table.push(StringEntry {
            has_interpolation: Self::has_unescaped_interpolation(&content),
            content,
        });
        idx
    }

    fn token(&self, kind: TokenKind, start: Mark) -> Token {
        Token {
            kind,
            range: start.offset..self.cursor.offset,
            line: start.line,
            col: start.col,
        }
    }

    fn has_unescaped_interpolation(content: &str) -> bool {
        let bytes = content.as_bytes();
        let mut i = 0;

        while i + 1 < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == b'$' && bytes[i + 1] == b'{' {
                return true;
            }
            i += 1;
        }
        false
    }
}

pub fn lex_all(
    source: &str,
    file: &str,
) -> Result<TokenStream, Vec<CompilerDiagnostic>> {
    Lexer::new(file.to_string(), source.to_string()).lex_all()
}
