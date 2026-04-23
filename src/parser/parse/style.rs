/// ! style parsing submodule of the parser
use crate::{
    ast::{Expr, KeyValue, Selector, StyleRule, StyleValue},
    diagnostic::SyntaxError,
    lexer::TokenKind,
    parser::{Parser, parse::Parse},
};

impl Parse for StyleRule {
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        // selector list
        let selector_list = match p.parse_split_on(
            |p| p.cursor.cur_tok() == &TokenKind::LeftBrace,
            |p| p.cursor.cur_tok() == &TokenKind::Comma,
            None,
        ) {
            Ok(list) => list,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };

        p.cursor.expect(TokenKind::LeftBrace)?;
        // declaration list
        let declarations = match p.parse_until(TokenKind::RightBrace) {
            Ok(list) => list,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBrace)?;

        Ok(Self::new(selector_list, declarations))
    }
}

impl Parse for Selector {
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        if p.cursor.cur_tok() == &TokenKind::Identifier {
            let name = p.cursor.cur_text().to_owned();
            p.cursor.advance();
            Ok(Selector::Type(name))
        } else if p.cursor.cur_tok() == &TokenKind::Dot {
            p.cursor.advance();
            let name = p.cursor.cur_text().to_owned();
            p.cursor.advance();
            Ok(Selector::Class(name))
        } else if p.cursor.cur_tok() == &TokenKind::Hash {
            p.cursor.advance();
            let name = p.cursor.cur_text().to_owned();
            p.cursor.advance();
            Ok(Selector::Id(name))
        } else {
            Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::Identifier, TokenKind::Dot, TokenKind::Hash],
                found: p.cursor.cur_tok().clone(),
            })
        }
    }
}

impl Parse for KeyValue {
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        if p.cursor.cur_tok() != &TokenKind::Identifier {
            return Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::Identifier],
                found: p.cursor.cur_tok().clone(),
            });
        }

        let key = KeyValue::parse_key(p)?;

        match p.cursor.cur_tok() {
            TokenKind::Colon | TokenKind::Equals => {
                p.cursor.advance();
            }
            _ => {
                return Err(SyntaxError::UnexpectedToken {
                    location: p.cursor.location(),
                    expected: vec![TokenKind::Equals, TokenKind::Colon],
                    found: p.cursor.cur_tok().clone(),
                });
            }
        }

        let value = KeyValue::parse_value(p)?;
        p.cursor.expect(TokenKind::Semicolon)?;

        Ok(KeyValue { key, value })
    }
}

impl KeyValue {
    fn parse_key(p: &mut Parser) -> Result<String, SyntaxError> {
        let mut key = String::new();
        while p.cursor.cur_tok() != &TokenKind::Colon && p.cursor.cur_tok() != &TokenKind::Equals {
            key.push_str(p.cursor.cur_text());
            p.cursor.advance();
        }

        Ok(key)
    }

    fn parse_value(p: &mut Parser) -> Result<StyleValue, SyntaxError> {
        let expr = Expr::parse(p)?;
        let unit = match p.cursor.cur_tok() {
            TokenKind::Identifier => Some(p.cursor.cur_text().to_string()),
            _ => None,
        };
        if let Some(_) = unit.clone() {
            p.cursor.advance();
        }

        Ok(StyleValue { expr, unit })
    }
}
