use crate::{
    ast::{Ident, LayoutDecl},
    diagnostic::CompilerDiagnostic,
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
        parser.consume(TokenKind::RightBrace)?;

        Ok(Self {
            element,
            items: Vec::new(),
        })
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
        assert!(layout.items.is_empty());
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
