use crate::{
    ast::{BinOp, BinaryExpr, Expr, ExprKind, InterpolatedStringExpr, UnaryExpr, UnaryOp},
    diagnostic::SourceLocation,
    lexer::TokenKind,
    lexer::lex,
    lexer::tokens::StringEntry,
    parser::parse::Parse,
    parser::parser::Parser,
    parser::parser_util::parser_err::ParseError,
    util::Spanned,
};

impl Parse for Expr {
    /// Parse an expression.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        Ok(Spanned::new(ExprKind::parse(p)?, p.cursor.location()))
    }

    /// Try to parse an expression.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for ExprKind {
    /// Parse an expression.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        ExprKind::parse_expr(p, 0)
    }

    /// Try to parse an expression.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        ExprKind::parse_expr(p, 0).ok()
    }
}

impl ExprKind {
    /// Pratt parser for expressions.
    pub fn parse_expr(p: &mut Parser, min_bp: u8) -> Result<Self, ParseError> {
        // Prefix
        let mut lhs = match p.cursor.cur_tok() {
            TokenKind::Int => ExprKind::parse_int(p)?,
            TokenKind::Float => ExprKind::parse_float(p)?,
            TokenKind::StringLiteral(idx) => ExprKind::parse_string(p, *idx)?,
            TokenKind::Identifier => ExprKind::parse_identifier(p)?,
            TokenKind::Minus | TokenKind::Bang => ExprKind::parse_prefix(p)?,
            TokenKind::LeftParen => ExprKind::parse_grouped(p)?,
            _ => {
                return Err(ParseError::new(
                    format!("expected expression, found {:?}", p.cursor.cur_tok()),
                    p.cursor.location(),
                ));
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
                ParseError::new(
                    format!("expected binary operator, found {:?}", p.cursor.cur_tok()),
                    p.cursor.location(),
                )
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
    fn parse_int(p: &mut Parser) -> Result<ExprKind, ParseError> {
        if let TokenKind::Int = p.cursor.cur_tok() {
            let value = p.cursor.cur_text();
            let int = value.parse::<i64>().unwrap();
            p.cursor.advance();
            Ok(ExprKind::Int(int))
        } else {
            Err(ParseError::new(
                format!("expected integer, found {:?}", p.cursor.cur_tok()),
                p.cursor.location(),
            ))
        }
    }

    /// Parse a float literal.
    fn parse_float(p: &mut Parser) -> Result<ExprKind, ParseError> {
        if let TokenKind::Float = p.cursor.cur_tok() {
            let value = p.cursor.cur_text();
            let float = value.parse::<f64>().unwrap();
            p.cursor.advance();
            Ok(ExprKind::Float(float))
        } else {
            Err(ParseError::new(
                format!("expected float, found {:?}", p.cursor.cur_tok()),
                p.cursor.location(),
            ))
        }
    }

    /// Parse a string literal.
    fn parse_string(p: &mut Parser, idx: usize) -> Result<ExprKind, ParseError> {
        let value = p.cursor.get_string(idx).cloned().ok_or_else(|| {
            ParseError::new(
                format!("expected string, found {:?}", p.cursor.cur_tok()),
                p.cursor.location(),
            )
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
    fn parse_identifier(p: &mut Parser) -> Result<ExprKind, ParseError> {
        let name = p.cursor.cur_text().to_owned();
        p.cursor.advance();
        Ok(ExprKind::Identifier(name))
    }

    /// Parse a prefix expression.
    fn parse_prefix(p: &mut Parser) -> Result<ExprKind, ParseError> {
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
            _ => Err(ParseError::new(
                format!("expected prefix operator, found {:?}", token),
                p.cursor.location(),
            )),
        }
    }

    /// Parse a grouped expression.
    fn parse_grouped(p: &mut Parser) -> Result<ExprKind, ParseError> {
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
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        let TokenKind::StringLiteral(idx) = p.cursor.cur_tok() else {
            return Err(ParseError::new(
                format!("expected string literal, found: {:?}", p.cursor.cur_tok()),
                p.cursor.location(),
            ));
        };
        let Some(raw_string) = p.cursor.get_string(*idx).cloned() else {
            return Err(ParseError::new(
                format!("expected string literal, found: {:?}", p.cursor.cur_tok()),
                p.cursor.location(),
            ));
        };

        p.cursor.advance();
        InterpolatedStringExpr::from_string_entry(&raw_string, p.cursor.location())
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl InterpolatedStringExpr {
    fn from_string_entry(
        raw_string: &StringEntry,
        location: SourceLocation,
    ) -> Result<Self, ParseError> {
        let parts = InterpolatedStringExpr::interpolate(raw_string, &location)?;
        Ok(InterpolatedStringExpr { parts })
    }

    fn interpolate(
        raw_string: &StringEntry,
        location: &SourceLocation,
    ) -> Result<Vec<ExprKind>, ParseError> {
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
                    return Err(ParseError::new(
                        "unclosed interpolation".to_string(),
                        location.clone(),
                    ));
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

    fn parse_embedded_expr(expr: &str, location: &SourceLocation) -> Result<ExprKind, ParseError> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(ParseError::new(
                "empty interpolation".to_string(),
                location.clone(),
            ));
        }

        let tokens = lex(expr, &location.file).map_err(|errors| {
            let first = errors.into_iter().next().unwrap();
            ParseError::new(first.message, location.clone())
        })?;

        let mut parser = Parser::new(tokens);
        ExprKind::parse(&mut parser)
    }
}
