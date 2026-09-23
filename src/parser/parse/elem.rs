use crate::ast::Content;
use crate::ast::ElemDecl;
use crate::ast::ElemInvoke;
use crate::ast::FieldValue;
use crate::ast::Ident;
use crate::ast::InlineText;
use crate::diagnostic::CompilerDiagnostic;
use crate::parser::tokens::TokenKind;
use crate::parser::Parse;
use crate::parser::Parser;

impl Parse for ElemDecl {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        parser.consume_keyword("elem")?;
        parser.skip_trivia()?;
        let name = Ident::parse(parser)?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::LeftBrace)?;
        parser.skip_trivia()?;
        let fields = ElemDecl::comsume_fields(parser)?;
        let mut content = false;
        if parser.at_keyword("content")? {
            parser.consume_keyword("content")?;
            parser.skip_trivia()?;
            content = true;
        }
        if parser.at(TokenKind::Comma)? {
            parser.consume(TokenKind::Comma)?;
            parser.skip_trivia()?;
        }
        parser.consume(TokenKind::RightBrace)?;

        Ok(Self {
            name,
            fields,
            content,
        })
    }
}

impl ElemDecl {
    fn comsume_fields(
        parser: &mut Parser,
    ) -> Result<Vec<Ident>, CompilerDiagnostic> {
        let mut fields = Vec::new();
        while !parser.at_keyword("content")?
            && !parser.at(TokenKind::RightBrace)?
            && !parser.at(TokenKind::Eof)?
        {
            fields.push(Ident::parse(parser)?);
            parser.skip_trivia()?;
            if parser.at(TokenKind::Comma)? {
                parser.consume(TokenKind::Comma)?;
                parser.skip_trivia()?;
            } else {
                break;
            }
        }
        Ok(fields)
    }
}

impl Parse for ElemInvoke {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        parser.consume(TokenKind::At)?;
        let name = Ident::parse(parser)?;
        parser.skip_trivia()?;
        parser.consume(TokenKind::LeftBrace)?;
        parser.skip_trivia()?;
        let fields = ElemInvoke::comsume_field_values(parser)?;
        parser.skip_trivia()?;
        let content_val = Content::parse(parser)?;
        let mut content = None;
        if !content_val.blocks.is_empty() {
            content = Some(content_val);
        }
        parser.consume(TokenKind::RightBrace)?;

        Ok(Self {
            name,
            fields,
            content,
        })
    }
}

impl ElemInvoke {
    fn comsume_field_values(
        parser: &mut Parser,
    ) -> Result<Vec<FieldValue>, CompilerDiagnostic> {
        let mut field_values = Vec::new();
        while !parser.at(TokenKind::RightBrace)?
            && !parser.at(TokenKind::Eof)?
        {
            let field = Ident::parse(parser)?;
            parser.skip_inline_trivia()?;
            parser.consume(TokenKind::Colon)?;
            parser.skip_inline_trivia()?;
            let value = InlineText::parse(parser)?;
            field_values.push(FieldValue { name: field, value });
            parser.skip_inline_trivia()?;
            if parser.at(TokenKind::Newline)? {
                if parser.peek_next_significant()?.kind == TokenKind::Newline {
                    break;
                }
                parser.consume(TokenKind::Newline)?;
                parser.skip_trivia()?;
            } else {
                break;
            }
        }
        Ok(field_values)
    }
}

//////////////////////////////
///                        ///
/// ELEMENT PARSING TESTS  ///
///                        ///
//////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ContentBlock;

    fn parser(source: &str) -> Result<Parser, CompilerDiagnostic> {
        let file = "element-test.pyr".to_string();
        Parser::new(file, source.to_string())
    }

    fn parse_decl(source: &str) -> Result<ElemDecl, CompilerDiagnostic> {
        ElemDecl::parse(&mut parser(source)?)
    }

    fn parse_invoke(source: &str) -> Result<ElemInvoke, CompilerDiagnostic> {
        ElemInvoke::parse(&mut parser(source)?)
    }

    #[test]
    fn parses_declaration_fields_with_flexible_trivia() {
        let declaration = parse_decl("elem card { title,\n body }")
            .expect("element declaration should parse");

        assert_eq!(declaration.name.text, "card");
        assert_eq!(
            declaration
                .fields
                .iter()
                .map(|field| field.text.as_str())
                .collect::<Vec<_>>(),
            vec!["title", "body"]
        );
        assert!(!declaration.content);
    }

    #[test]
    fn parses_declaration_with_content_marker() {
        let declaration = parse_decl("elem card { title, content }")
            .expect("content marker should parse");

        assert_eq!(declaration.fields.len(), 1);
        assert!(declaration.content);
    }

    #[test]
    fn rejects_missing_comma_between_fields() {
        assert!(parse_decl("elem card { title body }").is_err());
    }

    #[test]
    fn rejects_duplicate_commas_between_fields() {
        assert!(parse_decl("elem card { title,, body }").is_err());
    }

    #[test]
    fn rejects_non_identifier_declaration_name() {
        assert!(parse_decl("elem 123 {}").is_err());
    }

    #[test]
    fn rejects_unclosed_declaration() {
        assert!(parse_decl("elem card {").is_err());
    }

    #[test]
    fn parses_empty_invocation() {
        let invocation =
            parse_invoke("@card {}").expect("empty invocation should parse");

        assert_eq!(invocation.name.text, "card");
        assert!(invocation.fields.is_empty());
        assert!(invocation.content.is_none());
    }

    #[test]
    fn parses_same_line_closing_brace() {
        let invocation = parse_invoke("@card { name: hello }")
            .expect("same line closing brace should parse");

        assert_eq!(invocation.name.text, "card");
        assert_eq!(invocation.fields.len(), 1);
        assert!(invocation.content.is_none());
    }

    #[test]
    fn parses_full_entry() {
        let entry = parse_invoke( "@entry { \n title: Software Engineer \n company: ACME \n \n Built a compiler in Rust. \n }")
            .expect("full entry should parse");

        assert_eq!(entry.name.text, "entry");
        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields[0].name.text, "title");
        assert_eq!(
            entry.fields[0].value.parts,
            vec![crate::ast::Inline::Text("Software Engineer ".to_string())]
        );
        assert_eq!(entry.fields[1].name.text, "company");
        assert_eq!(
            entry.fields[1].value.parts,
            vec![crate::ast::Inline::Text("ACME ".to_string())]
        );
        let content = entry.content.expect("content should be present");
        assert_eq!(
            content.blocks,
            vec![ContentBlock::Paragraph(InlineText {
                parts: vec![crate::ast::Inline::Text(
                    "Built a compiler in Rust. ".to_string()
                )],
            })]
        );
    }

    #[test]
    fn rejects_invocation_field_without_colon() {
        assert!(parse_invoke("@card { title }").is_err());
    }

    #[test]
    fn rejects_non_identifier_invocation_name() {
        assert!(parse_invoke("@123 {}").is_err());
    }

    #[test]
    fn rejects_unclosed_invocation() {
        assert!(parse_invoke("@card {").is_err());
    }
}
