mod lexer_cursor;
pub mod tokens;

use std::usize;

use tokens::{StringEntry, TokenKind};

use crate::{
    diagnostic::{CompilerDiagnostic, SourceLocation, SyntaxError},
    lexer::lexer_cursor::Cursor,
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
    let mut t = [None; 256];

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

/// token representing a specific or group of keywords. holds the kind of token
/// it is and position data.
///
/// NOTE: this is currently in AOS format instead of previous SOA format due to
/// wanting a pull configuration for the lexer simplifying logic
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub range: std::ops::Range<usize>,
    pub line: usize,
    pub col: usize,
}

/// token stream of the current source file.
#[derive(Debug)]
pub struct TokenStream {
    pub file: String,
    pub tokens: Vec<Token>,
    pub string_table: Vec<StringEntry>,
}

impl TokenStream {
    pub fn new(file: String) -> Self {
        Self {
            file,
            tokens: Vec::new(),
            string_table: Vec::new(),
        }
    }
}

/// Lexer stuct for generating tokens
///
/// contains a cursor holding the current position in the file and a mode stack
/// as a state machine of the different types of tokens that need to be 
/// generated
pub struct Lexer {
    cursor: Cursor,
    mode: Vec<LexerModes>,
}

impl Lexer {
    pub fn new(file: String, src: String) -> Self {
        Lexer {
            cursor: Cursor::new(file, &src),
            mode: Vec::new(),
        }
    }
    
    /// eager lexing implementation
    pub fn lex_all(&mut self) -> Result<TokenStream, Vec<CompilerDiagnostic>>{
        let errors = Vec::new();
        let output = TokenStream::new(self.file);
        loop {
            let token = match self.pull() {
                Ok(token) => token,
                Err(err) => errors.push(err),
           };
            
            // each file will end with a end of file token which is our sign to
            // exit
            if token.kind == TokenKind::Eof {
                break;
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(output)
    }

    /// state machine based pull request for the cursor to make the next token
    ///
    fn pull(&mut self) -> Result<Token, CompilerDiagnostic> {
        match self.mode {
            
        }       
    }
}
