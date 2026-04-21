use crate::{
    ast::{ArgType, ExprKind, FuncDeclStmt, FuncParam, ReturnStmt, Stmt, StmtKind, Type},
    diagnostic::SyntaxError,
    lexer::TokenKind,
    parser::{Parser, parse::Parse},
    util::Spanned,
};

impl Parse for FuncParam {
    /// Parses a function parameter, which consists of a name and a type.
    ///
    /// Returns a `FuncParam` if successful, or a `ParseError` if the name is invalid.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
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
                return Err(SyntaxError::InvalidConstruct {
                    location: p.cursor.location(),
                    construct: "func param".to_string(),
                    reason: "Expected type identifier".to_string(),
                });
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
}

impl Parse for ArgType {
    /// Parses an argument type, which consists of a name and a type.
    ///
    /// Returns an `ArgType` if successful, or a `ParseError` if the name is invalid.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        match *p.cursor.cur_tok() {
            TokenKind::Identifier => {
                let name = p.cursor.cur_text().to_string();
                let ty = Type::Var;
                p.cursor.advance();
                Ok(Self { name, ty })
            }
            TokenKind::Int => {
                let name = p.cursor.cur_text().to_string();
                p.cursor.advance();
                Ok(Self {
                    name,
                    ty: Type::Int,
                })
            }
            TokenKind::Float => {
                let name = p.cursor.cur_text().to_string();
                p.cursor.advance();
                Ok(Self {
                    name,
                    ty: Type::Float,
                })
            }
            TokenKind::StringLiteral(idx) => {
                let name = {
                    let entry =
                        p.cursor
                            .get_string(idx)
                            .ok_or_else(|| SyntaxError::InvalidConstruct {
                                location: p.cursor.location(),
                                construct: "func param".to_string(),
                                reason: "Expected string literal".to_string(),
                            })?;
                    entry.content.clone()
                };
                p.cursor.advance();

                let ty = Type::String;
                Ok(Self { name, ty })
            }
            _ => Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::Identifier],
                found: p.cursor.cur_tok().clone(),
            }),
        }
    }
}

impl Parse for FuncDeclStmt {
    /// Parses a function declaration statement, which consists of a name, arguments, body, and return type.
    ///
    /// Returns a `FuncDeclStmt` if successful, or a `ParseError` if the name is invalid.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
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
        p.cursor.expect(TokenKind::RightParen)?;

        p.cursor.expect(TokenKind::LeftBrace)?;
        let body = match p.parse_until::<Stmt>(TokenKind::RightBrace) {
            Ok(body) => body,
            Err(errors) => {
                return Err(errors.into_iter().next().unwrap());
            }
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        let return_type = FuncDeclStmt::infer_return_type(body.as_ref());

        Ok(Self {
            name,
            args,
            body,
            return_type,
        })
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
