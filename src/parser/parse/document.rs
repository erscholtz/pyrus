use crate::{
    ast::{DocumentConfig, DocumentEntry, Ident},
    diagnostic::CompilerDiagnostic,
    parser::{Parse, Parser},
    tokens::TokenKind,
    util::Spanned,
};

impl Parse for DocumentConfig {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        parser.consume_keyword("document")?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::LeftBrace)?;
        parser.skip_trivia()?;
        let mut entries = Vec::new();
        while !parser.at(TokenKind::RightBrace) {
            entries.push(DocumentEntry::parse(parser)?);
            parser.skip_trivia()?;
        }
        parser.consume(TokenKind::RightBrace)?;

        Ok(Self { entries })
    }
}

impl Parse for DocumentEntry {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let name = Ident::parse(parser)?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::Colon)?;
        parser.skip_trivia()?;
        let node = parser.consume_lexeme()?;
        let value = Spanned::new(node.to_owned(), parser.location());

        Ok(Self { name, value })
    }
}

///////////////////////////////
///                         ///
/// DOCUMENT PARSING TESTS  ///
///                         ///
///////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_document(
        source: &str,
    ) -> Result<DocumentConfig, CompilerDiagnostic> {
        let file = "document-test.pyr".to_string();
        let lexer = Lexer::new(file.clone(), source.to_string());
        let mut parser = Parser::new(file, lexer)?;
        DocumentConfig::parse(&mut parser)
    }

    #[test]
    fn parses_compact_entries_and_trailing_newline() {
        let document = parse_document("document {type:A4\nmargin:4\n}")
            .expect("document config should parse");

        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].name.text, "type");
        assert_eq!(document.entries[0].value.node, "A4");
        assert_eq!(document.entries[1].name.text, "margin");
        assert_eq!(document.entries[1].value.node, "4");
    }

    #[test]
    fn parses_comments_between_entries() {
        let document =
            parse_document("document { type: A4 // page size\n margin: 4 }")
                .expect("comments should be trivia");

        assert_eq!(document.entries.len(), 2);
    }

    // NOTE are these parser's responsibility?
    #[test]
    fn rejects_entry_without_colon() {
        assert!(parse_document("document { type A4 }").is_err());
    }

    #[test]
    fn rejects_entry_without_value() {
        assert!(parse_document("document { type: }").is_err());
    }

    #[test]
    fn rejects_non_identifier_entry_name() {
        assert!(parse_document("document { 123: A4 }").is_err());
    }

    #[test]
    fn rejects_unclosed_document() {
        assert!(parse_document("document { type: A4").is_err());
    }
}
