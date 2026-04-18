use crate::{
    ast::{Ast, DocElem, DocumentBlock, Stmt, StyleBlock, StyleRule, TemplateBlock},
    lexer::TokenKind,
    parser::{parse::Parse, parser::Parser, parser_err::ParseError},
};

impl Parse for Ast {
    /// Parse an AST.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        let file = p.file.clone();
        let mut template = None;
        let mut document = None;
        let mut style = None;

        // Keep parsing blocks until we hit EOF
        while !p.cursor.check(TokenKind::Eof) {
            match p.cursor.cur_tok() {
                TokenKind::Template => {
                    if template.is_some() {
                        return Err(ParseError::new(
                            "duplicate template block".to_string(),
                            p.cursor.location(),
                        ));
                    }
                    template = Some(TemplateBlock::parse(p)?);
                }
                TokenKind::Document => {
                    if document.is_some() {
                        return Err(ParseError::new(
                            "duplicate document block".to_string(),
                            p.cursor.location(),
                        ));
                    }
                    document = Some(DocumentBlock::parse(p)?);
                }
                TokenKind::Style => {
                    if style.is_some() {
                        return Err(ParseError::new(
                            "duplicate style block".to_string(),
                            p.cursor.location(),
                        ));
                    }
                    style = Some(StyleBlock::parse(p)?);
                }
                _ => {
                    return Err(ParseError::new(
                        format!(
                            "unexpected token {:?}, expected template, document, or style",
                            p.cursor.cur_tok(),
                        ),
                        p.cursor.location(),
                    ));
                }
            }
        }

        Ok(Ast {
            file,
            template,
            document,
            style,
        })
    }

    /// Try to parse an AST.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for TemplateBlock {
    /// Parse a template block.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Template)?;

        p.cursor.expect(TokenKind::LeftBrace)?;
        let statements = match p.parse_until::<Stmt>(TokenKind::RightBrace) {
            Ok(statements) => statements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        Ok(TemplateBlock { statements })
    }

    /// Try to parse a template block.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for DocumentBlock {
    /// Parse a document block.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Document)?;

        p.cursor.expect(TokenKind::LeftBrace)?;
        let elements = match p.parse_until::<DocElem>(TokenKind::RightBrace) {
            Ok(elements) => elements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        Ok(DocumentBlock { elements })
    }

    /// Try to parse a document block.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

// TODO
impl Parse for StyleBlock {
    /// Parse a style block.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Style)?;

        p.cursor.expect(TokenKind::LeftBrace)?;
        let statements = match p.parse_until::<StyleRule>(TokenKind::RightBrace) {
            Ok(statements) => statements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        Ok(StyleBlock { statements })
    }

    /// Try to parse a style block.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}
