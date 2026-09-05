use std::{convert::identity, fmt::Alignment::Left};

use crate::{
    ast::{Ident, LayoutAlignment, LayoutDecl, LayoutProperty, LayoutRow},
    diagnostic::{CompilerDiagnostic, SyntaxError},
    parser::{Parse, Parser},
    tokens::TokenKind::{self, Identifier},
};

impl Parse for LayoutDecl {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        parser.consume_keyword("layout")?;
        parser.skip_trivia()?;
        let element = Ident::parse(parser)?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::LeftBrace)?;
        parser.skip_trivia()?;
        let (rows, props) = LayoutDecl::parse_contents(parser)?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::RightBrace)?;

        Ok(Self {
            element,
            rows,
            props,
        })
    }
}

// TODO return to this at some point this is another 100 lines horror
impl LayoutDecl {
    fn parse_contents(
        parser: &mut Parser,
    ) -> Result<(Vec<LayoutRow>, Vec<LayoutProperty>), CompilerDiagnostic> {
        let mut rows = Vec::new();
        let mut props = Vec::new();

        while !parser.at(TokenKind::RightBrace) {
            if parser.at(TokenKind::Greater) || parser.at(TokenKind::Less) {
                let alignment = if parser.at(TokenKind::Greater) {
                    Some(LayoutAlignment::Right)
                } else {
                    Some(LayoutAlignment::Left)
                };
                parser.next()?; // NOTE find a nicer way to consume this token here
                parser.skip_inline_trivia()?;
                let ident = Ident::parse(parser)?;
                let row = LayoutRow::parse_after(
                    ident.to_owned(),
                    alignment,
                    parser,
                )?;
                rows.push(row);
            }
            let ident = Ident::parse(parser)?;
            parser.skip_inline_trivia()?;
            if parser.at(TokenKind::Colon) {
                let prop =
                    LayoutProperty::parse_after(ident.to_owned(), parser)?;
                props.push(prop);
            } else {
                let row =
                    LayoutRow::parse_after(ident.to_owned(), None, parser)?;
                rows.push(row);
            }
        }

        Ok((rows, props))
    }
}

// TODO rewrite this for the case that alignment is given before the ident, writen correctly above but not here yet
impl LayoutRow {
    fn parse_after(
        ident: Ident,
        alignment: Option<LayoutAlignment>,
        parser: &mut Parser,
    ) -> Result<Self, CompilerDiagnostic> {
        parser.skip_inline_trivia()?;
        // NOTE alignment check, there
        // TODO there is a lot of cool ideas here:
        // <> | <>      centeres both elements with split in the middle
        // < | < | <    3 column layout
        // <>           centre a single item
        //
        // <<           absulte push to the edge formatting, where the regular
        //              < might try and align with the top element??
        //
        // etc etc etc,
        let mut just_left = true; // always aligned left to start
        let mut just_right = false;
        let mut split = false;
        //
        while !parser.at(TokenKind::RightBracket)
            && !parser.at(TokenKind::Eof)
            && !parser.at(TokenKind::Newline)
            && !parser.at(Identifier)
        {
            match parser.current.kind {
                TokenKind::Less => {
                    parser.consume(TokenKind::Less)?;
                    just_left = true;
                }
                TokenKind::Greater => {
                    parser.consume(TokenKind::Greater)?;
                    just_right = true;
                }
                TokenKind::Pipe => {
                    parser.consume(TokenKind::Pipe)?;
                    split = true;
                }
                _ => {
                    // NOTE this could also be wrong for example
                    // item1
                    // item2
                    //
                    // item1: sm
                    // item2: sm
                    //
                    // this causes an error for item1,
                    // FIX this is a jank fix
                    parser.skip_inline_trivia()?;
                    if parser.current.kind == TokenKind::Newline {
                        continue;
                    }
                    return Err(SyntaxError::unexpected_token(
                        vec![TokenKind::Pipe],
                        parser.current_kind(),
                        parser.location(),
                    )
                    .into());
                }
            }
            parser.skip_inline_trivia()?;
        }
        if split {
            let right = Ident::parse(parser)?;
            return Ok(LayoutRow::Split {
                left: ident,
                right,
                left_alignment: LayoutAlignment::Left,
                right_alignment: LayoutAlignment::Right,
            });
        }

        Ok(LayoutRow::Single {
            field: ident,
            alignment: LayoutAlignment::Left,
        })
    }
}

impl LayoutProperty {
    fn parse_after(
        field: Ident,
        parser: &mut Parser,
    ) -> Result<Self, CompilerDiagnostic> {
        parser.skip_inline_trivia()?;
        parser.consume(TokenKind::Colon)?;
        parser.skip_inline_trivia()?;
        let value = Ident::parse(parser)?;
        parser.skip_trivia()?;

        Ok(Self { field, value })
    }
}

/////////////////////////////
///                       ///
/// LAYOUT PARSING TESTS  ///
///                       ///
/////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_layout(source: &str) -> Result<LayoutDecl, CompilerDiagnostic> {
        let file = "layout-test.pyr".to_string();
        let lexer = Lexer::new(file.clone(), source.to_string());
        let mut parser = Parser::new(file, lexer)?;
        LayoutDecl::parse(&mut parser)
    }

    #[test]
    fn parses_empty_layout_with_internal_trivia() {
        let layout = parse_layout("layout card {\n // empty for now\n}")
            .expect("empty layout should parse");

        assert_eq!(layout.element.text, "card");
        assert!(layout.rows.is_empty());
        assert!(layout.props.is_empty());
    }

    #[test]
    fn rejects_missing_layout_name() {
        assert!(parse_layout("layout {}").is_err());
    }

    #[test]
    fn rejects_non_identifier_layout_name() {
        assert!(parse_layout("layout 123 {}").is_err());
    }

    #[test]
    fn rejects_missing_left_brace() {
        assert!(parse_layout("layout card }").is_err());
    }

    #[test]
    fn rejects_unclosed_layout() {
        assert!(parse_layout("layout card {").is_err());
    }

    #[test]
    fn rejects_unsupported_layout_body_without_ignoring_it() {
        assert!(parse_layout("layout card { title: value }").is_err());
    }
}
