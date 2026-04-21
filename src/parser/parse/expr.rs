use crate::{
    ast::{BinOp, BinaryExpr, Expr, ExprKind, InterpolatedStringExpr, UnaryExpr, UnaryOp},
    diagnostic::{SourceLocation, SyntaxError},
    lexer::{TokenKind, lex, tokens::StringEntry},
    parser::{Parser, parse::Parse},
    util::Spanned,
};

impl Parse for Expr {
    /// Parse an expression.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        Ok(Spanned::new(ExprKind::parse(p)?, p.cursor.location()))
    }
}

impl Parse for ExprKind {
    /// Parse an expression.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        ExprKind::parse_expr(p, 0)
    }
}

impl ExprKind {
    /// Pratt parser for expressions.
    pub fn parse_expr(p: &mut Parser, min_bp: u8) -> Result<Self, SyntaxError> {
        // Prefix
        let mut lhs = match p.cursor.cur_tok() {
            TokenKind::Int => ExprKind::parse_int(p)?,
            TokenKind::Float => ExprKind::parse_float(p)?,
            TokenKind::StringLiteral(idx) => ExprKind::parse_string(p, *idx)?,
            TokenKind::Identifier => ExprKind::parse_identifier(p)?,
            TokenKind::Minus | TokenKind::Bang => ExprKind::parse_prefix(p)?,
            TokenKind::LeftParen => ExprKind::parse_grouped(p)?,
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    location: p.cursor.location(),
                    expected: vec![
                        TokenKind::Int,
                        TokenKind::Float,
                        TokenKind::StringLiteral(0),
                        TokenKind::Identifier,
                        TokenKind::Minus,
                        TokenKind::Bang,
                        TokenKind::LeftParen,
                    ],
                    found: p.cursor.cur_tok().clone(),
                });
            }
        };

        // Infix
        loop {
            let (l_bp, r_bp) = match ExprKind::infix_binding_power(p.cursor.cur_tok()) {
                Some(bp) => bp,
                None => break,
            };

            if l_bp < min_bp {
                break;
            }

            let op = ExprKind::binary_op(p.cursor.cur_tok()).ok_or_else(|| {
                SyntaxError::UnexpectedToken {
                    location: p.cursor.location(),
                    expected: vec![
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Star,
                        TokenKind::Slash,
                        TokenKind::Equals,
                    ],
                    found: p.cursor.cur_tok().clone(),
                }
            })?;

            p.cursor.advance(); // consume operator
            let rhs = ExprKind::parse_expr(p, r_bp)?;

            lhs = ExprKind::Binary(BinaryExpr {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });
        }

        Ok(lhs)
    }

    /// Get the binding power of an infix operator.
    fn infix_binding_power(tok: &TokenKind) -> Option<(u8, u8)> {
        match tok {
            TokenKind::Plus | TokenKind::Minus => Some((1, 2)),
            TokenKind::Star | TokenKind::Slash => Some((3, 4)),
            TokenKind::Equals => Some((5, 6)),
            _ => None,
        }
    }

    /// Parse an integer literal.
    fn parse_int(p: &mut Parser) -> Result<ExprKind, SyntaxError> {
        if let TokenKind::Int = p.cursor.cur_tok() {
            let value = p.cursor.cur_text();
            let int = value.parse::<i64>().unwrap();
            p.cursor.advance();
            Ok(ExprKind::Int(int))
        } else {
            Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::Int],
                found: p.cursor.cur_tok().clone(),
            })
        }
    }

    /// Parse a float literal.
    fn parse_float(p: &mut Parser) -> Result<ExprKind, SyntaxError> {
        if let TokenKind::Float = p.cursor.cur_tok() {
            let value = p.cursor.cur_text();
            let float = value.parse::<f64>().unwrap();
            p.cursor.advance();
            Ok(ExprKind::Float(float))
        } else {
            Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::Float],
                found: p.cursor.cur_tok().clone(),
            })
        }
    }

    /// Parse a string literal.
    fn parse_string(p: &mut Parser, idx: usize) -> Result<ExprKind, SyntaxError> {
        let value =
            p.cursor
                .get_string(idx)
                .cloned()
                .ok_or_else(|| SyntaxError::UnexpectedToken {
                    location: p.cursor.location(),
                    expected: vec![TokenKind::String],
                    found: p.cursor.cur_tok().clone(),
                })?;
        p.cursor.advance();
        if value.has_interpolation {
            Ok(ExprKind::InterpolatedString(
                InterpolatedStringExpr::from_string_entry(&value, p.cursor.location())?,
            ))
        } else {
            Ok(ExprKind::StringLiteral(value.content))
        }
    }

    /// Parse an identifier.
    fn parse_identifier(p: &mut Parser) -> Result<ExprKind, SyntaxError> {
        let name = p.cursor.cur_text().to_owned();
        p.cursor.advance();
        Ok(ExprKind::Identifier(name))
    }

    /// Parse a prefix expression.
    fn parse_prefix(p: &mut Parser) -> Result<ExprKind, SyntaxError> {
        let token = p.cursor.cur_tok();
        match token {
            TokenKind::Minus => {
                let bp = 5;
                p.cursor.advance();
                let expr = ExprKind::parse_expr(p, bp)?;
                Ok(ExprKind::Unary(UnaryExpr {
                    op: UnaryOp::Negate,
                    expr: Box::new(expr),
                }))
            }
            TokenKind::Bang => {
                let bp = 5;
                p.cursor.advance();
                let expr = ExprKind::parse_expr(p, bp)?;
                Ok(ExprKind::Unary(UnaryExpr {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                }))
            }
            _ => Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::Minus, TokenKind::Bang],
                found: token.clone(),
            }),
        }
    }

    /// Parse a grouped expression.
    fn parse_grouped(p: &mut Parser) -> Result<ExprKind, SyntaxError> {
        p.cursor.advance();
        let expr = ExprKind::parse_expr(p, 0)?;
        p.cursor.expect(TokenKind::RightParen)?;
        Ok(expr)
    }

    /// Parse a binary operator.
    fn binary_op(op: &TokenKind) -> Option<BinOp> {
        match op {
            TokenKind::Plus => Some(BinOp::Add),
            TokenKind::Minus => Some(BinOp::Subtract),
            TokenKind::Star => Some(BinOp::Multiply),
            TokenKind::Slash => Some(BinOp::Divide),
            TokenKind::Equals => Some(BinOp::Equals),
            TokenKind::Percent => Some(BinOp::Mod),
            _ => None,
        }
    }
}

impl Parse for InterpolatedStringExpr {
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        let TokenKind::StringLiteral(idx) = p.cursor.cur_tok() else {
            return Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::StringLiteral(0)],
                found: p.cursor.cur_tok().clone(),
            });
        };
        let Some(raw_string) = p.cursor.get_string(*idx).cloned() else {
            return Err(SyntaxError::UnexpectedEof {
                location: p.cursor.location(),
                expected: "string literal".to_string(),
            });
        };

        p.cursor.advance();
        InterpolatedStringExpr::from_string_entry(&raw_string, p.cursor.location())
    }
}

impl InterpolatedStringExpr {
    fn from_string_entry(
        raw_string: &StringEntry,
        location: SourceLocation,
    ) -> Result<Self, SyntaxError> {
        let parts = InterpolatedStringExpr::interpolate(raw_string, &location)?;
        Ok(InterpolatedStringExpr { parts })
    }

    fn interpolate(
        raw_string: &StringEntry,
        location: &SourceLocation,
    ) -> Result<Vec<ExprKind>, SyntaxError> {
        let mut parts = Vec::new();
        let bytes = raw_string.content.as_bytes();
        let mut i = 0;
        let mut literal_start = 0;

        while i < bytes.len() {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                if literal_start < i {
                    parts.push(ExprKind::StringLiteral(
                        raw_string.content[literal_start..i].to_string(),
                    ));
                }

                let expr_start = i + 2;
                let mut brace_depth = 1;
                i += 2;

                while i < bytes.len() && brace_depth > 0 {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }

                    if bytes[i] == b'{' {
                        brace_depth += 1;
                    } else if bytes[i] == b'}' {
                        brace_depth -= 1;
                    }

                    i += 1;
                }

                if brace_depth != 0 {
                    return Err(SyntaxError::UnterminatedDelimiter {
                        location: location.clone(),
                        delimiter: "}".to_string(),
                    });
                }

                let expr_end = i - 1;
                let expr = InterpolatedStringExpr::parse_embedded_expr(
                    &raw_string.content[expr_start..expr_end],
                    location,
                )?;
                parts.push(expr);
                literal_start = i;
                continue;
            }

            i += 1;
        }

        if literal_start < raw_string.content.len() {
            parts.push(ExprKind::StringLiteral(
                raw_string.content[literal_start..].to_string(),
            ));
        }

        Ok(parts)
    }

    fn parse_embedded_expr(expr: &str, location: &SourceLocation) -> Result<ExprKind, SyntaxError> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(SyntaxError::UnexpectedEof {
                location: location.clone(),
                expected: "expression".to_string(),
            });
        }

        let tokens = match lex(expr, &location.file) {
            Ok(tokens) => tokens,
            Err(errors) => {
                return Err(errors.into_iter().next().unwrap());
            }
        };

        let mut parser = Parser::new(tokens);
        parser.cursor.set_trace_context("embedded-expr");
        ExprKind::parse(&mut parser)
    }
}
