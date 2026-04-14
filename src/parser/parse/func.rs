use crate::{
    ast::{ArgType, ExprKind, FuncDeclStmt, FuncParam, ReturnStmt, Stmt, StmtKind, Type},
    lexer::TokenKind,
    parser::parse::Parse,
    parser::parser::Parser,
    parser::parser_err::ParseError,
    util::Spanned,
};

impl Parse for FuncParam {
    /// Parses a function parameter, which consists of a name and a type.
    ///
    /// Returns a `FuncParam` if successful, or a `ParseError` if the name is invalid.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        // Parse parameter name (identifier)
        let name = p.cursor.cur_text().to_owned();
        p.cursor.advance();
        p.cursor.expect(TokenKind::Colon)?;

        // Parse type
        let ty = match *p.cursor.cur_tok() {
            TokenKind::String => Type::String,
            TokenKind::Int => Type::Int,
            TokenKind::Float => Type::Float,
            _ => {
                return Err(ParseError::new(
                    "Expected type identifier".to_string(),
                    p.cursor.location(),
                ));
            }
        };
        p.cursor.advance(); // consume the type token

        // Consume trailing comma if present
        if p.cursor.check(TokenKind::Comma) {
            p.cursor.advance();
        }

        // Create a placeholder expression for the parameter name
        let value = Spanned::new(ExprKind::Identifier(name), p.cursor.location());
        Ok(Self { ty, value })
    }

    /// Tries to parse a function parameter, which consists of a type and a value.
    ///
    /// Returns `Some` if a valid parameter is found, `None` otherwise.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if let Some(ty) = p.cursor.peek_tok() {
            if let TokenKind::Identifier = ty {
                return FuncParam::parse(p).ok();
            }
        }
        None
    }
}

impl Parse for ArgType {
    /// Parses an argument type, which consists of a name and a type.
    ///
    /// Returns an `ArgType` if successful, or a `ParseError` if the name is invalid.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        if *p.cursor.cur_tok() == TokenKind::Identifier {
            let name = p.cursor.cur_text().to_string();
            let ty;
            if name.starts_with('"') {
                ty = Type::String;
            } else if let Ok(_) = name.parse::<i32>() {
                ty = Type::Int;
            } else if let Ok(_) = name.parse::<f64>() {
                ty = Type::Float;
            } else {
                ty = Type::Var;
            }
            Ok(Self { name, ty })
        } else {
            Err(ParseError::new(
                "Expected identifier".to_string(),
                p.cursor.location(),
            ))
        }
    }

    /// Tries to parse a function parameter, which consists of a type and a value.
    ///
    /// returns `Some` if a valid parameter is found, `None` otherwise.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if let Some(ty) = p.cursor.peek_tok() {
            if let TokenKind::Identifier = ty {
                return ArgType::parse(p).ok();
            }
        }
        None
    }
}

impl Parse for FuncDeclStmt {
    /// Parses a function declaration statement, which consists of a name, arguments, body, and return type.
    ///
    /// Returns a `FuncDeclStmt` if successful, or a `ParseError` if the name is invalid.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Func)?;
        let name = p.cursor.cur_text().to_owned();
        p.cursor.advance(); // consume the function name
        p.cursor.expect(TokenKind::LeftParen)?;
        let args = match p.parse_until::<FuncParam>(TokenKind::RightParen) {
            Ok(args) => args,
            Err(errors) => {
                return Err(errors.into_iter().next().unwrap());
            }
        };
        p.cursor.expect(TokenKind::LeftBrace)?;
        let body = match p.parse_until::<Stmt>(TokenKind::RightBrace) {
            Ok(body) => body,
            Err(errors) => {
                return Err(errors.into_iter().next().unwrap());
            }
        };
        let return_type = FuncDeclStmt::infer_return_type(body.as_ref());

        Ok(Self {
            name,
            args,
            body,
            return_type,
        })
    }

    /// Tries to parse a function declaration statement, which consists of a name, arguments, body, and return type.
    ///
    /// Returns `Some` if a valid statement is found, `None` otherwise.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.cur_tok() == &TokenKind::Identifier {
            Some(FuncDeclStmt::parse(p).ok()?)
        } else {
            None
        }
    }
}

impl FuncDeclStmt {
    fn infer_return_type(body: &[Stmt]) -> Option<Type> {
        for stmt in body {
            if let StmtKind::Return(return_stmt) = &stmt.node {
                match &return_stmt {
                    ReturnStmt::Expr(expr) => match &expr.node {
                        ExprKind::Int(_) => Some(Type::Int),
                        ExprKind::Float(_) => Some(Type::Float),
                        ExprKind::StringLiteral(_) => Some(Type::String),
                        _ => None,
                    },
                    ReturnStmt::DocElem(_) => Some(Type::DocElem),
                };
            }
        }
        None
    }
}
