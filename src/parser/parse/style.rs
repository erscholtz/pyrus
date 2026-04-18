/// ! style parsing submodule of the parser
use crate::{
    ast::{Expr, KeyValue, Selector, StyleRule, StyleValue},
    lexer::TokenKind,
    parser::{Parser, parse::Parse, parser_err::ParseError},
};

impl Parse for StyleRule {
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
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

        Ok(Self {
            selector_list,
            declaration_block: declarations,
            specificity: 0,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for Selector {
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
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
            Err(ParseError::new(
                format!("Unexpected token: {:?}", p.cursor.cur_tok()),
                p.cursor.location(),
            ))
        }
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Selector::parse(p).ok()
    }
}

impl Parse for KeyValue {
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        if p.cursor.cur_tok() != &TokenKind::Identifier {
            return Err(ParseError::new(
                format!("Expected identifier, found {:?}", p.cursor.cur_tok()),
                p.cursor.location(),
            ));
        }

        let key = KeyValue::parse_key(p)?;

        match p.cursor.cur_tok() {
            TokenKind::Colon | TokenKind::Equals => {
                p.cursor.advance();
            }
            _ => {
                return Err(ParseError::new(
                    format!("Expected ':' or '=', found {:?}", p.cursor.cur_tok()),
                    p.cursor.location(),
                ));
            }
        }

        let value = KeyValue::parse_value(p)?;
        p.cursor.expect(TokenKind::Semicolon)?;

        Ok(KeyValue { key, value })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        KeyValue::parse(p).ok()
    }
}

impl KeyValue {
    fn parse_key(p: &mut Parser) -> Result<String, ParseError> {
        let mut key = String::new();
        while p.cursor.cur_tok() != &TokenKind::Colon && p.cursor.cur_tok() != &TokenKind::Equals {
            key.push_str(p.cursor.cur_text());
            p.cursor.advance();
        }

        Ok(key)
    }

    fn parse_value(p: &mut Parser) -> Result<StyleValue, ParseError> {
        let expr = Expr::parse(p)?;
        let unit = match p.cursor.cur_tok() {
            TokenKind::Identifier => Some(p.cursor.cur_text().to_string()),
            _ => None,
        };
        if let Some(unit) = unit.clone() {
            p.cursor.advance();
        }

        Ok(StyleValue { expr, unit })
    }
}
