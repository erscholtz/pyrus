use core::panic;

use crate::ast::{Ast, DocumentBlock, Expression, InterpPart, StyleBlock, TemplateBlock};
use crate::lexer::{TokenKind, TokenStream};
use crate::parser::parser_err::ParseError;

pub fn parse(tokens: TokenStream) -> Result<Ast, Vec<ParseError>> {
    let p = Parser::new(tokens);
    p.parse()
}

pub struct Parser {
    pub toks: TokenStream,
    pub idx: usize,
}

impl Parser {
    fn new(toks: TokenStream) -> Self {
        Self { toks, idx: 0 }
    }

    fn parse(mut self) -> Result<Ast, Vec<ParseError>> {
        // high level pass

        let mut template = None;
        let mut document = None;
        let mut style = None;
        let mut errors: Vec<ParseError> = Vec::new();

        while self.idx < self.toks.kinds.len() {
            match self.current_token_kind() {
                TokenKind::Template => {
                    self.expect(TokenKind::Template);
                    self.expect(TokenKind::LeftBrace);
                    let template_block = self.parse_template_block();
                    template = Some(TemplateBlock {
                        statements: template_block,
                    });
                }
                TokenKind::Document => {
                    self.expect(TokenKind::Document);
                    self.expect(TokenKind::LeftBrace);
                    let document_block = self.parse_document_block();
                    document = Some(DocumentBlock {
                        elements: document_block,
                    });
                }
                TokenKind::Style => {
                    self.expect(TokenKind::Style);
                    self.expect(TokenKind::LeftBrace);
                    let style_block = self.parse_style_block();
                    style = Some(StyleBlock {
                        statements: style_block,
                    });
                }
                TokenKind::Eof => break,
                _ => errors.push(ParseError::new(
                    format!(
                        "Parse error: unexpected token at top level (can only be Template, Document, Style at top level). Found: {:?} at {}:{}",
                        self.current_token_kind(),
                        self.current_token_line(),
                        self.current_token_col()
                    ),
                    self.current_token_line() as usize,
                    self.current_token_col() as usize,
                )),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Ast {
            template,
            document,
            style,
        })
    }

    pub fn parse_expression(&mut self) -> Expression {
        match self.current_token_kind() {
            // TODO handle binary operators (eventually)
            TokenKind::Minus => {
                self.advance();
                let right = self.parse_expression();
                Expression::Unary {
                    operator: crate::ast::UnaryOp::Negate,
                    expression: Box::new(right),
                }
            }
            TokenKind::Bang => {
                self.advance();
                let right = self.parse_expression();
                Expression::Unary {
                    operator: crate::ast::UnaryOp::Not,
                    expression: Box::new(right),
                }
            }
            TokenKind::StringLiteral => {
                let value = self.current_text();
                self.advance();
                // Check if the string contains interpolation patterns
                self.parse_string_with_interpolation(&value)
            }
            TokenKind::Float => {
                let value = self.current_text();
                self.advance();
                Expression::Float(value.parse().unwrap())
            }
            TokenKind::Int => {
                let value = self.current_text();
                self.advance();
                Expression::Int(value.parse().unwrap())
            }
            TokenKind::Dollarsign => {
                self.advance(); // first $
                let expression = self.parse_expression();
                self.advance(); // other $
                expression
            }
            TokenKind::Identifier => self.parse_binary_expr(),
            _ => panic!(
                "Parse error: unexpected token parsing expression. Found: {:?} at {}:{}",
                self.current_token_kind(),
                self.current_token_line(),
                self.current_token_col()
            ),
        }
    }

    fn parse_string_with_interpolation(&self, s: &str) -> Expression {
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
            Expression::StringLiteral(String::new())
        } else if parts.len() == 1 {
            match &parts[0] {
                InterpPart::Text(text) => Expression::StringLiteral(text.clone()),
                InterpPart::Expression(_) => Expression::InterpolatedString(parts),
            }
        } else {
            Expression::InterpolatedString(parts)
        }
    }

    fn parse_expression_from_str(&self, expr_str: &str) -> Expression {
        let trimmed = expr_str.trim();
        if trimmed.is_empty() {
            return Expression::StringLiteral(String::new());
        }
        if let Ok(n) = trimmed.parse::<i64>() {
            return Expression::Int(n);
        }
        if let Ok(f) = trimmed.parse::<f64>() {
            return Expression::Float(f);
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
                    return Expression::Binary {
                        left: Box::new(self.parse_expression_from_str(left.trim())),
                        operator,
                        right: Box::new(self.parse_expression_from_str(right.trim())),
                    };
                }
                _ => {}
            }
        }

        Expression::Identifier(trimmed.to_string())
    }

    fn parse_binary_expr(&mut self) -> Expression {
        let left = match self.current_token_kind() {
            TokenKind::Identifier => {
                let name = self.current_text();
                self.advance();
                Expression::Identifier(name)
            }
            _ => panic!(
                "Parse error: unexpected token in binary expression {:?} at {}:{}",
                self.current_token_kind(),
                self.current_token_line(),
                self.current_token_col()
            ),
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
            return Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        left
    }
}
