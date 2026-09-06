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

impl LayoutDecl {
    fn parse_contents(
        parser: &mut Parser,
    ) -> Result<(Vec<LayoutRow>, Vec<LayoutProperty>), CompilerDiagnostic> {
        let mut rows = Vec::new();
        let mut props = Vec::new();

        while !parser.at(TokenKind::RightBrace)? {
            parser.skip_trivia()?;

            // TODO: Make property detection trivia-aware so `field : value`
            // dispatches to `LayoutProperty` just like `field: value`.
            if parser.at(TokenKind::Greater)? || parser.at(TokenKind::Less)? {
                let row = LayoutRow::parse(parser)?;
                rows.push(row);
            } else if parser.at(TokenKind::Identifier)? {
                if parser.peek_next_significant()?.kind == TokenKind::Colon {
                    let prop = LayoutProperty::parse(parser)?;
                    props.push(prop);
                } else {
                    // NOTE this case is the case of applying the default layout
                    let row = LayoutRow::parse(parser)?;
                    rows.push(row);
                }
            } else {
                return Err(SyntaxError::unexpected_token(
                    vec![TokenKind::Identifier],
                    parser.peek()?.kind,
                    parser.location()?,
                )
                .into());
            }
        }

        Ok((rows, props))
    }
}

// TODO: Decide whether rows may contain more than two fields before extending
// the AST. A repeated `alignment field` grammar could support forms such as
// `< first | <> second | > third` without adding special cases here.
impl Parse for LayoutRow {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let alignment = LayoutRow::parse_alignment(parser)?;
        parser.skip_inline_trivia()?;
        let field = Ident::parse(parser)?;
        parser.skip_inline_trivia()?;

        // TODO: Require a newline or closing brace after the row. Without an
        // explicit terminator, `first second` is accepted as two separate rows.
        if parser.at(TokenKind::Pipe)? {
            parser.consume(TokenKind::Pipe)?;
            parser.skip_inline_trivia()?;
            let right_alignment = LayoutRow::parse_alignment(parser)?;
            let right = Ident::parse(parser)?;
            return Ok(LayoutRow::Split {
                left: field,
                right,
                left_alignment: alignment,
                right_alignment,
            });
        }

        Ok(LayoutRow::Single { field, alignment })
    }
}

impl LayoutRow {
    fn parse_alignment(
        parser: &mut Parser,
    ) -> Result<LayoutAlignment, CompilerDiagnostic> {
        // TODO: Parse only the alignment forms supported by the grammar (`<`,
        // `>`, and `<>`). The flag-based approach also accepts malformed forms
        // such as `><` and repeated markers like `<<>>` as centred.
        let mut just_left = false;
        let mut just_right = false;

        while !parser.at(TokenKind::RightBracket)?
            && !parser.at(TokenKind::Eof)?
            && !parser.at(TokenKind::Newline)?
            && !parser.at(Identifier)?
        {
            match parser.peek()?.kind {
                TokenKind::Less => {
                    parser.consume(TokenKind::Less)?;
                    just_left = true;
                }
                TokenKind::Greater => {
                    parser.consume(TokenKind::Greater)?;
                    just_right = true;
                }
                _ => {
                    parser.skip_inline_trivia()?;
                    if parser.peek()?.kind == TokenKind::Identifier {
                        break;
                    }
                    // TODO: Report the alignment/identifier tokens that are
                    // valid here instead of always claiming a pipe was expected.
                    return Err(SyntaxError::unexpected_token(
                        vec![TokenKind::Pipe],
                        parser.peek()?.kind,
                        parser.location()?,
                    )
                    .into());
                }
            }
            parser.skip_inline_trivia()?;
        }
        if just_left && just_right {
            return Ok(LayoutAlignment::Centre);
        } else if just_left {
            return Ok(LayoutAlignment::Left);
        } else if just_right {
            return Ok(LayoutAlignment::Right);
        } else {
            return Ok(LayoutAlignment::Left);
        }
    }
}

impl Parse for LayoutProperty {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        parser.skip_inline_trivia()?;
        let field = Ident::parse(parser)?;
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
    fn parses_odd_prop_formatting() {
        let layout = parse_layout("layout card {\n item : xl\n}")
            .expect("empty layout should parse");

        assert_eq!(layout.element.text, "card");
        assert!(layout.rows.is_empty());
        assert_eq!(layout.props.len(), 1);
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
}
