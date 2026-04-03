use crate::ast::{
    Ast, DocumentBlock, Expression, ExpressionKind, InterpPart, StyleBlock, TemplateBlock,
};
use crate::error::SourceLocation;
use crate::lexer::{TokenKind, TokenStream};
use crate::parser::parser_err::ParseError;
use crate::util::Spanned;

pub fn parse(tokens: TokenStream) -> Result<Ast, Vec<ParseError>> {
    let p = Parser::new(tokens);
    p.parse()
}

pub struct Parser {
    pub file: String,
    pub toks: TokenStream,
    pub idx: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    fn new(toks: TokenStream) -> Self {
        Self {
            file: toks.file.clone(),
            toks,
            idx: 0,
            errors: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<Ast, Vec<ParseError>> {
        // high level pass

        let mut template = None;
        let mut document = None;
        let mut style = None;

        while self.idx < self.toks.kinds.len() {
            // Skip whitespace at top level
            while self.current_token_kind() == TokenKind::Whitespace {
                self.advance();
            }

            if self.idx >= self.toks.kinds.len() {
                break;
            }

            match self.current_token_kind() {
                TokenKind::Template => {
                    self.expect(TokenKind::Template);
                    self.expect(TokenKind::LeftBrace);
                    let template_block = self.parse_template_block();
                    template = Some(TemplateBlock {
                        statements: template_block,
                    });
                    self.expect(TokenKind::RightBrace);
                }
                TokenKind::Document => {
                    self.expect(TokenKind::Document);
                    self.expect(TokenKind::LeftBrace);
                    let document_block = self.parse_document_block();
                    document = Some(DocumentBlock {
                        elements: document_block,
                    });
                    self.expect(TokenKind::RightBrace);
                }
                TokenKind::Style => {
                    self.expect(TokenKind::Style);
                    self.expect(TokenKind::LeftBrace);
                    let style_block = self.parse_style_block();
                    style = Some(StyleBlock {
                        statements: style_block,
                    });
                    self.expect(TokenKind::RightBrace);
                }
                TokenKind::Eof => break,
                _ => {
                    self.errors.push(ParseError::new(
                        format!(
                            "Parse error: unexpected token at top level (can only be Template, Document, Style at top level). Found: {:?} at {}:{}",
                            self.current_token_kind(),
                            self.current_token_line(),
                            self.current_token_col()
                        ),
                        self.current_token_line() as usize,
                        self.current_token_col() as usize,
                        self.file.clone(),
                    ));
                    self.advance();
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(Ast {
            file: self.file.clone(),
            template,
            document,
            style,
        })
    }

    pub fn parse_expression(&mut self) -> Expression {
        let start_line = self.current_token_line();
        let start_col = self.current_token_col();

        let kind = match self.current_token_kind() {
            TokenKind::Minus => {
                self.advance();
                let right = self.parse_expression();
                ExpressionKind::Unary {
                    operator: crate::ast::UnaryOp::Negate,
                    expression: Box::new(right),
                }
            }
            TokenKind::Bang => {
                self.advance();
                let right = self.parse_expression();
                ExpressionKind::Unary {
                    operator: crate::ast::UnaryOp::Not,
                    expression: Box::new(right),
                }
            }
            TokenKind::StringLiteral(idx) => {
                let idx_usize = idx as usize;
                let entry = self.toks.string_table[idx_usize].clone();
                self.advance();
                // Check if the string contains interpolation patterns
                if entry.has_interpolation {
                    self.parse_string_with_interpolation(&entry.content)
                } else {
                    // Process escape sequences even for non-interpolated strings
                    let processed = self.process_escape_sequences(&entry.content);
                    ExpressionKind::StringLiteral(processed)
                }
            }
            TokenKind::Float => {
                let value = self.current_text();
                self.advance();
                ExpressionKind::Float(value.parse().unwrap())
            }
            TokenKind::Int => {
                let value = self.current_text();
                self.advance();
                ExpressionKind::Int(value.parse().unwrap())
            }
            TokenKind::Dollarsign => {
                self.advance(); // first $
                let expression = self.parse_expression();
                self.advance(); // other $
                return expression; // Return directly to preserve location from inner expression
            }
            TokenKind::Identifier => self.parse_binary_expr(),
            _ => {
                self.errors.push(ParseError::new(
                    format!(
                        "Unexpected token '{}' at {}:{}",
                        self.current_text(),
                        self.current_token_line(),
                        self.current_token_col()
                    ),
                    self.current_token_line(),
                    self.current_token_col(),
                    self.file.clone(),
                ));
                self.advance();
                ExpressionKind::Int(0)
            }
        };

        let location = SourceLocation::new(start_line, start_col, self.file.clone());
        Spanned::new(kind, location)
    }

    pub fn parse_string_with_interpolation(&mut self, s: &str) -> ExpressionKind {
        // Strip surrounding quotes if present
        let content = if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
            &s[1..s.len() - 1]
        } else {
            s
        };

        let mut parts = Vec::new();
        let mut chars = content.chars().peekable();
        let mut current_text = String::new();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    current_text.push('{');
                    continue;
                }

                if !current_text.is_empty() {
                    parts.push(InterpPart::Text(current_text.clone()));
                    current_text.clear();
                }

                let mut expr_str = String::new();
                let mut brace_depth = 1;

                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        brace_depth += 1;
                        expr_str.push(ch);
                    } else if ch == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            break;
                        } else {
                            expr_str.push(ch);
                        }
                    } else {
                        expr_str.push(ch);
                    }
                }

                let expr = self.parse_expression_from_str(&expr_str.trim());
                parts.push(InterpPart::Expression(expr));
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next(); // consume second }
                    current_text.push('}');
                } else {
                    // Single } is just added to text (or could be an error)
                    current_text.push(ch);
                }
            } else if ch == '\\' {
                // Handle escape sequences
                if let Some(next_ch) = chars.next() {
                    match next_ch {
                        'n' => current_text.push('\n'),
                        't' => current_text.push('\t'),
                        'r' => current_text.push('\r'),
                        '\\' => current_text.push('\\'),
                        '"' => current_text.push('"'),
                        '{' => current_text.push('{'),
                        '}' => current_text.push('}'),
                        _ => {
                            // Unknown escape - keep both characters
                            current_text.push('\\');
                            current_text.push(next_ch);
                        }
                    }
                } else {
                    current_text.push('\\');
                }
            } else {
                current_text.push(ch);
            }
        }

        // Add remaining text
        if !current_text.is_empty() {
            parts.push(InterpPart::Text(current_text));
        }

        if parts.is_empty() {
            ExpressionKind::StringLiteral(String::new())
        } else if parts.len() == 1 {
            match &parts[0] {
                InterpPart::Text(text) => ExpressionKind::StringLiteral(text.clone()),
                InterpPart::Expression(_) => ExpressionKind::InterpolatedString(parts),
            }
        } else {
            ExpressionKind::InterpolatedString(parts)
        }
    }

    fn process_escape_sequences(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next_ch) = chars.next() {
                    match next_ch {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '{' => result.push('{'),
                        '}' => result.push('}'),
                        _ => {
                            // Unknown escape - keep both characters
                            result.push('\\');
                            result.push(next_ch);
                        }
                    }
                } else {
                    result.push('\\');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn parse_expression_from_str(&mut self, expr_str: &str) -> ExpressionKind {
        let trimmed = expr_str.trim();
        if trimmed.is_empty() {
            return ExpressionKind::StringLiteral(String::new());
        }
        if let Ok(n) = trimmed.parse::<i64>() {
            return ExpressionKind::Int(n);
        }
        if let Ok(f) = trimmed.parse::<f64>() {
            return ExpressionKind::Float(f);
        }

        for (op_pos, op_char) in trimmed.chars().enumerate() {
            match op_char {
                '+' | '-' | '*' | '/' | '=' => {
                    let left = &trimmed[..op_pos];
                    let right = &trimmed[op_pos + 1..];
                    let operator = match op_char {
                        '+' => crate::ast::BinaryOp::Add,
                        '-' => crate::ast::BinaryOp::Subtract,
                        '*' => crate::ast::BinaryOp::Multiply,
                        '/' => crate::ast::BinaryOp::Divide,
                        '=' => crate::ast::BinaryOp::Equals,
                        _ => unreachable!(),
                    };
                    // For string-interpolated expressions, we create simple ExpressionKind
                    // without location info since we don't have token positions
                    let left_expr = self.parse_expression_from_str(left.trim());
                    let right_expr = self.parse_expression_from_str(right.trim());
                    return ExpressionKind::Binary {
                        left: Box::new(Spanned::new(
                            left_expr,
                            SourceLocation::new(0, 0, self.file.clone()),
                        )),
                        operator,
                        right: Box::new(Spanned::new(
                            right_expr,
                            SourceLocation::new(0, 0, self.file.clone()),
                        )),
                    };
                }
                _ => {
                    // NOTE: Continue to next character - this is not an error,
                    // we're just looking for operators. Non-operator characters
                    // are part of the identifier.
                }
            }
        }

        ExpressionKind::Identifier(trimmed.to_string())
    }

    fn parse_binary_expr(&mut self) -> ExpressionKind {
        let left = match self.current_token_kind() {
            TokenKind::Identifier => {
                let name = self.current_text();
                self.advance();
                ExpressionKind::Identifier(name)
            }
            _ => {
                self.errors.push(ParseError::new(
                    format!(
                        "Unexpected token '{}' at {}:{}",
                        self.current_text(),
                        self.current_token_line(),
                        self.current_token_col()
                    ),
                    self.current_token_line(),
                    self.current_token_col(),
                    self.file.clone(),
                ));
                self.advance();
                ExpressionKind::Int(0)
            }
        };

        while let TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Equals = self.current_token_kind()
        {
            let operator = match self.current_token_kind() {
                TokenKind::Plus => crate::ast::BinaryOp::Add,
                TokenKind::Minus => crate::ast::BinaryOp::Subtract,
                TokenKind::Star => crate::ast::BinaryOp::Multiply,
                TokenKind::Slash => crate::ast::BinaryOp::Divide,
                TokenKind::Equals => crate::ast::BinaryOp::Equals,
                _ => unreachable!(),
            };
            self.advance(); // consume operator
            let right = self.parse_expression();
            let left_span = SourceLocation::new(
                self.current_token_line(),
                self.current_token_col(),
                self.file.clone(),
            );
            return ExpressionKind::Binary {
                left: Box::new(Spanned::new(left, left_span)),
                operator,
                right: Box::new(right),
            };
        }
        left
    }
}
