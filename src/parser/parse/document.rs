use crate::ast::DocumentConfig;
use crate::ast::DocumentEntry;
use crate::ast::Ident;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::Severity;
use crate::parser::Parse;
use crate::parser::Parser;
use crate::parser::tokens::TokenKind;
use crate::util::Spanned;

impl Parse for DocumentConfig {
    fn parse(parser: &mut Parser) -> Result<Self, Diagnostic> {
        parser.consume_keyword("document")?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::LeftBrace)?;
        parser.skip_trivia()?;
        let mut entries = Vec::new();
        while !parser.at(TokenKind::RightBrace)? && !parser.at(TokenKind::Eof)? {
            entries.push(DocumentEntry::parse(parser)?);
            parser.skip_trivia()?;
        }
        parser.consume(TokenKind::RightBrace)?;

        Ok(Self { entries })
    }
}

impl Parse for DocumentEntry {
    fn parse(parser: &mut Parser) -> Result<Self, Diagnostic> {
        let name = Ident::parse(parser)?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::Colon)?;
        parser.skip_inline_trivia()?;
        let location = parser.location()?;
        // values run to the end of the line so metadata like
        // title: Erik Scholtz Resume or link: https://x.com stays intact
        let mut node = String::new();
        while !matches!(
            parser.peek()?.kind,
            TokenKind::Newline | TokenKind::RightBrace | TokenKind::LineComment | TokenKind::Eof
        ) {
            node.push_str(&parser.consume_lexeme()?);
        }
        let node = node.trim_end().to_string();
        if node.is_empty() {
            return Err(Diagnostic {
                severity: Severity::Error,
                message: format!("missing value for document entry `{}`", name.text),
                location,
                span: None,
                help: None,
            });
        }
        let value = Spanned::new(node, location);

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

    fn parse_document(source: &str) -> Result<DocumentConfig, Diagnostic> {
        let file = "document-test.pyr".to_string();
        let mut parser = Parser::new(file, source.to_string())?;
        DocumentConfig::parse(&mut parser)
    }

    #[test]
    fn parses_compact_entries_and_trailing_newline() {
        let document =
            parse_document("document {type:A4\nmargin:4\n}").expect("document config should parse");

        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].name.text, "type");
        assert_eq!(document.entries[0].value.node, "A4");
        assert_eq!(document.entries[1].name.text, "margin");
        assert_eq!(document.entries[1].value.node, "4");
    }

    #[test]
    fn parses_comments_between_entries() {
        let document = parse_document("document { type: A4 // page size\n margin: 4 }")
            .expect("comments should be trivia");

        assert_eq!(document.entries.len(), 2);
    }

    #[test]
    fn parses_multi_word_and_url_values() {
        let document = parse_document(
            "document {\n title: Erik Scholtz Resume // pdf title\n link: https://github.com/erik\n}",
        )
        .expect("metadata entries should parse");

        assert_eq!(document.entries[0].value.node, "Erik Scholtz Resume");
        assert_eq!(document.entries[1].value.node, "https://github.com/erik");
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
