use crate::{
    ast::{Ast, DocElem, DocumentBlock, Stmt, StyleBlock, StyleRule, TemplateBlock},
    diagnostic::SyntaxError,
    lexer::TokenKind,
    parser::{Parser, parse::Parse},
};

impl Parse for Ast {
    /// Parse an AST.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        let file = p.file.clone();
        let mut template = None;
        let mut document = None;
        let mut style = None;

        // Keep parsing blocks until we hit EOF
        while !p.cursor.check(TokenKind::Eof) {
            match p.cursor.cur_tok() {
                TokenKind::Template => {
                    if template.is_some() {
                        return Err(SyntaxError::InvalidConstruct {
                            location: p.cursor.location(),
                            construct: "template".to_string(),
                            reason: "duplicate template block".to_string(),
                        });
                    }
                    template = Some(TemplateBlock::parse(p)?);
                }
                TokenKind::Document => {
                    if document.is_some() {
                        return Err(SyntaxError::InvalidConstruct {
                            location: p.cursor.location(),
                            construct: "document".to_string(),
                            reason: "duplicate document block".to_string(),
                        });
                    }
                    document = Some(DocumentBlock::parse(p)?);
                }
                TokenKind::Style => {
                    if style.is_some() {
                        return Err(SyntaxError::InvalidConstruct {
                            location: p.cursor.location(),
                            construct: "style".to_string(),
                            reason: "duplicate style block".to_string(),
                        });
                    }
                    style = Some(StyleBlock::parse(p)?);
                }
                _ => {
                    return Err(SyntaxError::InvalidConstruct {
                        location: p.cursor.location(),
                        construct: "root".to_string(),
                        reason: format!(
                            "unexpected token {:?}, expected template, document, or style",
                            p.cursor.cur_tok(),
                        ),
                    });
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
}

impl Parse for TemplateBlock {
    /// Parse a template block.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Template)?;

        p.cursor.expect(TokenKind::LeftBrace)?;
        let statements = match p.parse_until::<Stmt>(TokenKind::RightBrace) {
            Ok(statements) => statements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        Ok(TemplateBlock { statements })
    }
}

impl Parse for DocumentBlock {
    /// Parse a document block.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Document)?;

        p.cursor.expect(TokenKind::LeftBrace)?;
        let elements = match p.parse_until::<DocElem>(TokenKind::RightBrace) {
            Ok(elements) => elements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        Ok(DocumentBlock { elements })
    }
}

// TODO
impl Parse for StyleBlock {
    /// Parse a style block.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Style)?;

        p.cursor.expect(TokenKind::LeftBrace)?;
        let statements = match p.parse_until::<StyleRule>(TokenKind::RightBrace) {
            Ok(statements) => statements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        Ok(StyleBlock { statements })
    }
}
