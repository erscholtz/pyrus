use crate::{
    ast::{
        ChildrenStmt, ConstAssignStmt, DefaultSetStmt, DocElem, Expr, FuncDeclStmt, ReturnStmt,
        Stmt, StmtKind, VarAssignStmt,
    },
    diagnostic::SyntaxError,
    lexer::TokenKind,
    parser::{Parser, parse::Parse},
    util::Spanned,
};

impl Parse for Stmt {
    /// Parse a statement.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        Ok(Spanned::new(StmtKind::parse(p)?, p.cursor.location()))
    }
}

impl Parse for StmtKind {
    /// Parse a statement kind.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        match p.cursor.cur_tok() {
            TokenKind::Identifier => DefaultSetStmt::parse(p).map(|s| s.into()),
            TokenKind::Const => ConstAssignStmt::parse(p).map(|s| s.into()),
            TokenKind::Let => VarAssignStmt::parse(p).map(|s| s.into()),
            TokenKind::Return => ReturnStmt::parse(p).map(|s| s.into()),
            TokenKind::Func => FuncDeclStmt::parse(p).map(|s| s.into()),
            TokenKind::Children => ChildrenStmt::parse(p).map(|s| s.into()),
            // TODO: these ones below
            // TokenKind::If => IfStmt::parse(p).map(|s| s.into()),
            // TokenKind::While => WhileStmt::parse(p).map(|s| s.into()),
            // TokenKind::For => ForStmt::parse(p).map(|s| s.into()),
            _ => Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![
                    TokenKind::Identifier,
                    TokenKind::Const,
                    TokenKind::Let,
                    TokenKind::Return,
                    TokenKind::Func,
                    TokenKind::Children,
                ],
                found: p.cursor.cur_tok().clone(),
            }),
        }
    }
}

impl Parse for DefaultSetStmt {
    /// Parse a default set statement.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        let varname = p.cursor.cur_text().to_owned();
        p.cursor.advance();
        p.cursor.expect(TokenKind::Equals)?;
        let value = Expr::parse(p)?;
        Ok(DefaultSetStmt {
            key: varname.to_string(),
            value,
        })
    }
}

impl Parse for ConstAssignStmt {
    /// Parse a constant assignment statement.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Const)?;
        let varname = p.cursor.cur_text().to_owned();
        p.cursor.advance();
        p.cursor.expect(TokenKind::Equals)?;
        let value = Expr::parse(p)?;
        Ok(ConstAssignStmt {
            name: varname.to_string(),
            value,
        })
    }
}

impl Parse for VarAssignStmt {
    /// Parse a variable assignment statement.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Let)?;
        let varname = p.cursor.cur_text().to_owned();
        p.cursor.advance();
        p.cursor.expect(TokenKind::Equals)?;
        let value = Expr::parse(p)?;
        Ok(VarAssignStmt {
            name: varname.to_string(),
            value,
        })
    }
}

impl Parse for ReturnStmt {
    /// Parse a return statement.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Return)?;
        if p.cursor.cur_tok() == &TokenKind::At {
            let doc_elem = DocElem::parse(p)?;
            return Ok(ReturnStmt::DocElem(doc_elem));
        }
        let expr = Expr::parse(p)?;
        Ok(ReturnStmt::Expr(expr))
    }
}

impl Parse for ChildrenStmt {
    /// Parse a children statement.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Children)?;
        Ok(ChildrenStmt { children: true })
    }
}
