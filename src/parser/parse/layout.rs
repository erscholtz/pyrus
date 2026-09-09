use crate::{
    ast::{Ident, LayoutAlignment, LayoutDecl, LayoutProperty, LayoutRow},
    diagnostic::{CompilerDiagnostic, SyntaxError},
    parser::{Parse, Parser},
    tokens::TokenKind,
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
            if parser.at(TokenKind::Greater)? || parser.at(TokenKind::Less)? {
                let row = LayoutRow::parse(parser)?;
                rows.push(row);
            } else if parser.at(TokenKind::Identifier)? {
                if parser.peek_next_significant()?.kind == TokenKind::Colon {
                    let prop = LayoutProperty::parse(parser)?;
                    props.push(prop);
                } else {
                    let row = LayoutRow::parse(parser)?;
                    rows.push(row);
                }
            } else {
                return Err(SyntaxError::unexpected_token(
                    vec![
                        TokenKind::Identifier,
                        TokenKind::Greater,
                        TokenKind::Less,
                    ],
                    parser.peek()?.kind,
                    parser.location()?,
                )
                .into());
            }
            parser.skip_inline_trivia()?;
            if !parser.at(TokenKind::Newline)?
                && !parser.at(TokenKind::RightBrace)?
            {
                return Err(SyntaxError::unexpected_token(
                    vec![TokenKind::Newline, TokenKind::RightBrace],
                    parser.peek()?.kind,
                    parser.location()?,
                )
                .into());
            }
            parser.skip_trivia()?;
        }

        Ok((rows, props))
    }
}

// TODO: Decide whether rows may contain more than two fields before extending
// the AST. A repeated `alignment field` grammar could support forms such as
// `< first | <> second | > third` without adding special cases here.
impl Parse for LayoutRow {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let (alignment, field) = LayoutRow::parse_aligned_field(parser)?;
        parser.skip_inline_trivia()?;

        if parser.at(TokenKind::Pipe)? {
            parser.consume(TokenKind::Pipe)?;
            parser.skip_inline_trivia()?;
            let (r_alignment, r_field) =
                LayoutRow::parse_aligned_field(parser)?;
            return Ok(LayoutRow::Split {
                left: field,
                right: r_field,
                left_alignment: alignment,
                right_alignment: r_alignment,
            });
        }

        Ok(LayoutRow::Single { field, alignment })
    }
}

impl LayoutRow {
    fn parse_aligned_field(
        parser: &mut Parser,
    ) -> Result<(LayoutAlignment, Ident), CompilerDiagnostic> {
        let alignment = match (parser.peek()?.kind, parser.peek_next()?.kind) {
            (TokenKind::Less, TokenKind::Greater) => {
                parser.consume(TokenKind::Less)?;
                parser.consume(TokenKind::Greater)?;
                LayoutAlignment::Centre
            }
            (TokenKind::Less, _) => {
                parser.consume(TokenKind::Less)?;
                LayoutAlignment::Left
            }
            (TokenKind::Greater, _) => {
                parser.consume(TokenKind::Greater)?;
                LayoutAlignment::Right
            }
            (TokenKind::Identifier, _) => LayoutAlignment::Left,
            _ => {
                return Err(SyntaxError::unexpected_token(
                    vec![
                        TokenKind::Less,
                        TokenKind::Greater,
                        TokenKind::Identifier,
                    ],
                    parser.peek()?.kind,
                    parser.location()?,
                )
                .into());
            }
        };

        parser.skip_inline_trivia()?;
        let field = Ident::parse(parser)?;

        Ok((alignment, field))
    }
}

impl Parse for LayoutProperty {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let field = Ident::parse(parser)?;
        parser.skip_inline_trivia()?;
        parser.consume(TokenKind::Colon)?;
        parser.skip_inline_trivia()?;
        // NOTE see if this is ident or should be something else
        let value = Ident::parse(parser)?;

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
        let prop = layout.props[0].clone();
        assert_eq!(prop.field.text, "item");
        assert_eq!(prop.value.text, "xl");
    }

    #[test]
    fn parses_single_layout_alignment() {
        let layout = parse_layout("layout card { <> text \n > date }")
            .expect("single layout alignment should parse");

        assert_eq!(layout.element.text, "card");
        assert_eq!(layout.rows.len(), 2);
        assert!(layout.props.is_empty());

        let row1 = match layout.rows[0].clone() {
            LayoutRow::Single { field, alignment } => (field, alignment),
            _ => panic!("expected single layout row"),
        };
        assert_eq!(row1.0.text, "text");
        assert_eq!(row1.1, LayoutAlignment::Centre);
        let row2 = match layout.rows[1].clone() {
            LayoutRow::Single { field, alignment } => (field, alignment),
            _ => panic!("expected single layout row"),
        };
        assert_eq!(row2.0.text, "date");
        assert_eq!(row2.1, LayoutAlignment::Right);
    }

    #[test]
    fn parses_split_layout_row() {
        let layout = parse_layout("layout card { \n < text | > date \n }")
            .expect("split layout row should parse");

        assert_eq!(layout.element.text, "card");
        assert_eq!(layout.rows.len(), 1);
        assert!(layout.props.is_empty());

        let row = match layout.rows[0].clone() {
            LayoutRow::Split {
                left,
                right,
                left_alignment,
                right_alignment,
            } => (left, right, left_alignment, right_alignment),
            _ => panic!("expected split layout row"),
        };
        assert_eq!(row.0.text, "text");
        assert_eq!(row.1.text, "date");
        assert_eq!(row.2, LayoutAlignment::Left);
        assert_eq!(row.3, LayoutAlignment::Right);
    }

    #[test]
    fn parses_default_alignment() {
        let layout = parse_layout("layout card { text }")
            .expect("default alignment should parse");

        assert_eq!(layout.element.text, "card");
        assert_eq!(layout.rows.len(), 1);
        assert!(layout.props.is_empty());

        let row = match layout.rows[0].clone() {
            LayoutRow::Single { field, alignment } => (field, alignment),
            _ => panic!("expected single layout row"),
        };
        assert_eq!(row.0.text, "text");
        assert_eq!(row.1, LayoutAlignment::Left);
    }

    #[test]
    fn parses_inline_comments() {
        assert!(parse_layout("layout cart { > first //comment \n }").is_ok());
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
    fn rejects_bad_layout_alignment() {
        assert!(parse_layout("layout card { >< text }").is_err());
    }

    #[test]
    fn rejects_layout_without_newline() {
        assert!(parse_layout("layout card { first second }").is_err());
    }

    #[test]
    fn rejects_repeated_greater_less_symbols() {
        assert!(parse_layout("layout card { << first }").is_err());
        assert!(parse_layout("layout card { >> second }").is_err());
        assert!(parse_layout("layout card { <<>> third }").is_err());
    }
}
