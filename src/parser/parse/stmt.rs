use crate::{
    ast::{
        ChildrenStmt, ConstAssignStmt, DefaultSetStmt, DocElem, DocElemEmitStmt, Expr,
        FuncDeclStmt, ReturnStmt, Stmt, StmtKind, VarAssignStmt,
    },
    lexer::TokenKind,
    parser::parse::Parse,
    parser::parser::Parser,
    parser::parser_err::ParseError,
    util::Spanned,
};

impl Parse for Stmt {
    /// Parse a statement.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        Ok(Spanned::new(StmtKind::parse(p)?, p.cursor.location()))
    }

    /// Try to parse a statement.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for StmtKind {
    /// Parse a statement kind.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
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
            _ => Err(ParseError::new(
                format!("Unexpected Token: {}", p.cursor.cur_tok()),
                p.cursor.location(),
            )),
        }
    }

    /// Try to parse a statement kind.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for DefaultSetStmt {
    /// Parse a default set statement.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        let varname = p.cursor.cur_text().to_owned();
        p.cursor.advance();
        p.cursor.expect(TokenKind::Equals)?;
        let value = Expr::parse(p)?;
        Ok(DefaultSetStmt {
            key: varname.to_string(),
            value,
        })
    }

    /// Try to parse a default set statement.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.cur_tok() == &TokenKind::Identifier {
            Some(DefaultSetStmt::parse(p).ok()?)
        } else {
            None
        }
    }
}

impl Parse for ConstAssignStmt {
    /// Parse a constant assignment statement.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
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

    /// Try to parse a constant assignment statement.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.cur_tok() == &TokenKind::Identifier {
            Some(ConstAssignStmt::parse(p).ok()?)
        } else {
            None
        }
    }
}

impl Parse for VarAssignStmt {
    /// Parse a variable assignment statement.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
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

    /// Try to parse a variable assignment statement.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.cur_tok() == &TokenKind::Identifier {
            Some(VarAssignStmt::parse(p).ok()?)
        } else {
            None
        }
    }
}

impl Parse for ReturnStmt {
    /// Parse a return statement.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Return)?;
        if p.cursor.cur_tok() == &TokenKind::At {
            let doc_elem = DocElem::parse(p)?;
            return Ok(ReturnStmt::DocElem(doc_elem));
        }
        let expr = Expr::parse(p)?;
        Ok(ReturnStmt::Expr(expr))
    }

    /// Try to parse a return statement.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.cur_tok() == &TokenKind::Return {
            Some(ReturnStmt::parse(p).ok()?)
        } else {
            None
        }
    }
}

impl Parse for DocElemEmitStmt {
    /// Parse a document element emit statement.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        Err(ParseError::new(
            "not implemented!".to_string(),
            p.cursor.location(),
        ))
    }

    /// Try to parse a document element emit statement.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.cur_tok() == &TokenKind::Identifier {
            Some(DocElemEmitStmt::parse(p).ok()?)
        } else {
            None
        }
    }
}

impl Parse for ChildrenStmt {
    /// Parse a children statement.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Children)?;
        Ok(ChildrenStmt { children: true })
    }

    /// Try to parse a children statement.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.cur_tok() == &TokenKind::Children {
            Some(ChildrenStmt::parse(p).ok()?)
        } else {
            None
        }
    }
}
