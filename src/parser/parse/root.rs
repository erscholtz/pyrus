use crate::{
    ast::{Ast, DocumentConfig, ElemDecl, ElemInvoke, Ident, Item, LayoutDecl},
    diagnostic::{CompilerDiagnostic, Span, SyntaxError},
    parser::{Parse, Parser},
    tokens::TokenKind,
    util::Spanned,
};

impl Parse for Ast {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let mut items = Vec::new();
        parser.skip_trivia()?;

        while !parser.at(TokenKind::Eof) {
            let location = parser.location();
            let item = if parser.at_keyword("document") {
                Item::Document(DocumentConfig::parse(parser)?)
            } else if parser.at_keyword("elem") {
                Item::ElemDecl(ElemDecl::parse(parser)?)
            } else if parser.at_keyword("layout") {
                Item::LayoutDecl(LayoutDecl::parse(parser)?)
            } else if parser.at(TokenKind::At) {
                Item::ElemInvoke(ElemInvoke::parse(parser)?)
            } else {
                return Err(SyntaxError::invalid_construct(
                    "top-level item",
                    format!("unexpected token `{}`", parser.current_text()),
                    parser.location(),
                )
                .into());
            };

            items.push(Spanned::new(item, location));
            parser.skip_trivia()?;
        }

        Ok(Ast {
            file: parser.location().file,
            items,
        })
    }
}

impl Parse for Ident {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let text = parser.consume_lexeme()?;
        let span = Span::new(
            parser.current.range.start,
            parser.current.range.end,
            parser.file.clone(),
        );
        Ok(Ident { text, span })
    }
}

///////////////////////////
///                     ///
/// ROOT PARSING TESTS  ///
///                     ///
///////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_ast(source: &str) -> Result<Ast, CompilerDiagnostic> {
        let file = "root-test.pyr".to_string();
        let lexer = Lexer::new(file.clone(), source.to_string());
        let mut parser = Parser::new(file, lexer)?;
        Ast::parse(&mut parser)
    }

    #[test]
    fn parses_empty_and_trivia_only_sources() {
        assert!(
            parse_ast("")
                .expect("empty source should parse")
                .items
                .is_empty()
        );
        assert!(
            parse_ast(" \n // only trivia\n")
                .expect("trivia-only source should parse")
                .items
                .is_empty()
        );
    }

    #[test]
    fn parses_adjacent_top_level_items_without_whitespace() {
        let ast = parse_ast("document{}elem marker{}layout marker{}@marker{}")
            .expect("adjacent items should parse");

        assert_eq!(ast.items.len(), 4);
    }

    #[test]
    fn rejects_unknown_top_level_construct() {
        assert!(parse_ast("unknown {}").is_err());
    }

    #[test]
    fn treats_keywords_as_case_sensitive() {
        assert!(parse_ast("Document {}").is_err());
    }

    #[test]
    fn does_not_match_keyword_prefixes() {
        assert!(parse_ast("documentary {}").is_err());
    }

    #[test]
    fn rejects_stray_top_level_delimiters() {
        for source in ["}", "{", "@"] {
            assert!(parse_ast(source).is_err(), "source should fail: {source}");
        }
    }

    #[test]
    fn identifier_span_covers_the_consumed_identifier() {
        let ast = parse_ast("elem marker {}").expect("element should parse");
        let Item::ElemDecl(element) = &ast.items[0].node else {
            panic!("expected element declaration");
        };

        assert_eq!(element.name.text, "marker");
        assert_eq!(element.name.span.start, 5);
        assert_eq!(element.name.span.end, 11);
    }
}
