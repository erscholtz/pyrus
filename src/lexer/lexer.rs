use crate::lexer::tokens::TokenKind;

static KEYWORD_TABLE: phf::Map<&'static str, TokenKind> = phf::phf_map! {
    "template" => TokenKind::Template,
    "document" => TokenKind::Document,
    "style" => TokenKind::Style,
    "func" => TokenKind::Func,
    "element" => TokenKind::Element,
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
};

static SYMBOL_LOOKUP_TABLE: [Option<TokenKind>; 256] = {
    let mut t = [None; 256];

    use TokenKind::*;

    t[b'(' as usize] = Some(LeftParen);
    t[b')' as usize] = Some(RightParen);
    t[b'{' as usize] = Some(LeftBrace);
    t[b'}' as usize] = Some(RightBrace);
    t[b'[' as usize] = Some(LeftBracket);
    t[b']' as usize] = Some(RightBracket);
    t[b',' as usize] = Some(Comma);
    t[b'.' as usize] = Some(Dot);
    t[b';' as usize] = Some(Semicolon);
    t[b':' as usize] = Some(Colon);
    t[b'+' as usize] = Some(Plus);
    t[b'-' as usize] = Some(Minus);
    t[b'*' as usize] = Some(Star);
    t[b'/' as usize] = Some(Slash);
    t[b'%' as usize] = Some(Percent);
    t[b'=' as usize] = Some(Equals);
    t[b'$' as usize] = Some(Dollarsign);
    t[b'#' as usize] = Some(Hash);
    t[b'!' as usize] = Some(Bang);
    t[b'>' as usize] = Some(Greater);
    t[b'<' as usize] = Some(Less);
    t[b'@' as usize] = Some(At);
    t[b'|' as usize] = Some(Pipe);

    t
};

#[derive(Debug)]
pub struct TokenStream {
    pub kinds: Vec<TokenKind>,
    pub ranges: Vec<std::ops::Range<usize>>,
    pub lines: Vec<usize>,
    pub cols: Vec<usize>,
    pub source: String,
    pub errors: Vec<LexError>,
    pub warnings: Vec<LexError>, // TODO
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl LexError {
    pub fn new(message: String, line: usize, col: usize) -> Self {
        Self { message, line, col }
    }
}

impl TokenStream {
    pub fn new(source: String) -> Self {
        Self {
            kinds: Vec::new(),
            ranges: Vec::new(),
            lines: Vec::new(),
            cols: Vec::new(),
            source,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[inline]
    fn push(&mut self, kind: TokenKind, start: usize, end: usize, line: usize, col: usize) {
        self.kinds.push(kind);
        self.ranges.push(start..end);
        self.lines.push(line);
        self.cols.push(col);
    }
}

#[inline]
fn is_ident_start(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_'
}

#[inline]
fn is_ident_continue(c: u8) -> bool {
    is_ident_start(c) || (c >= b'0' && c <= b'9')
}

pub fn lex(source: &str) -> Result<TokenStream, Vec<LexError>> {
    let mut out = TokenStream::new(source.to_string());
    let bytes = source.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    let mut line = 1;
    let mut col = 1;

    while i < len {
        let c = bytes[i];

        // --- Whitespace ---
        if c.is_ascii_whitespace() {
            if c == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            i += 1;
            continue;
        }

        let start = i;

        // --- Identifiers ---
        if is_ident_start(c) {
            i += 1;
            while i < len && is_ident_continue(bytes[i]) {
                i += 1;
            }
            // check for let and const
            let ident_str = &source[start..i];
            let kind = KEYWORD_TABLE
                .get(ident_str)
                .copied()
                .unwrap_or(TokenKind::Identifier);
            out.push(kind, start, i, line, col);

            col += i - start;
            continue;
        }

        // --- Numbers ---
        if c.is_ascii_digit() {
            let mut is_float = false;
            i += 1;
            while i < len && bytes[i].is_ascii_digit() {
                i += 1;
            }

            if i < len && bytes[i] == b'.' {
                is_float = true;
                i += 1;
                while i < len && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }

            let kind = if is_float {
                TokenKind::Float
            } else {
                TokenKind::Int
            };
            out.push(kind, start, i, line, col);
            col += i - start;
            continue;
        }

        // --- String literals ---
        if c == b'"' {
            i += 1; // skip opening quote
            let mut escaped = false;
            let string_start_line = line;
            let string_start_col = col;
            while i < len {
                if escaped {
                    escaped = false;
                } else if bytes[i] == b'"' {
                    break;
                } else if bytes[i] == b'\\' {
                    escaped = true;
                }
                // Track newlines inside strings for error reporting
                if bytes[i] == b'\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                i += 1;
            }
            if i >= len {
                // Unterminated string
                out.errors.push(LexError::new(
                    format!("Unterminated string literal"),
                    string_start_line,
                    string_start_col,
                ));
            } else {
                i += 1; // Skip closing quote
                col += 1;
            }
            out.push(TokenKind::StringLiteral, start, i, line, col);
            continue;
        }

        // --- Comments ---
        if c == b'/' && i + 1 < len {
            if bytes[i + 1] == b'/' {
                i += 2;
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            } else if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    if bytes[i] == b'\n' {
                        line += 1;
                        col = 1;
                    }
                    i += 1;
                }
                i += 2; // skip closing */
                continue;
            }
        }

        // --- Single-character tokens ---
        if let Some(kind) = SYMBOL_LOOKUP_TABLE[c as usize] {
            out.push(kind, i, i + 1, line, col);
            i += 1;
            col += 1;
            continue;
        }

        // --- Unknown character ---
        out.errors.push(LexError::new(
            format!("Unknown character: '{}'", c as char),
            line,
            col,
        ));
    }

    // --- EOF ---
    out.push(TokenKind::Eof, len, len, line, col);

    return Ok(out);
}
