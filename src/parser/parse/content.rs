use crate::ast::Content;
use crate::ast::ContentBlock;
use crate::ast::Inline;
use crate::ast::InlineText;
use crate::diagnostic::CompilerDiagnostic;
use crate::parser::Parse;
use crate::parser::Parser;
use crate::parser::tokens::TokenKind;

impl Parse for Content {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let mut blocks = Vec::new();
        while !parser.at(TokenKind::RightBrace)?
            && !parser.at(TokenKind::Eof)?
        {
            parser.skip_trivia()?;
            blocks.push(ContentBlock::parse(parser)?);
            parser.skip_trivia()?;
        }
        Ok(Content { blocks })
    }
}

impl Parse for ContentBlock {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        match parser.peek()?.kind {
            TokenKind::Dash => {
                let list = ContentBlock::consume_list(parser)?;
                Ok(ContentBlock::BulletList(list))
            }
            _ => {
                let paragraph = InlineText::parse(parser)?;
                Ok(ContentBlock::Paragraph(paragraph))
            }
        }
    }
}

impl ContentBlock {
    fn consume_list(
        parser: &mut Parser,
    ) -> Result<Vec<InlineText>, CompilerDiagnostic> {
        let mut list = Vec::new();
        while parser.at(TokenKind::Dash)? {
            parser.consume(TokenKind::Dash)?;
            parser.skip_inline_trivia()?;
            list.push(InlineText::parse(parser)?);
            if parser.at(TokenKind::Eof)? {
                break;
            }
            parser.consume(TokenKind::Newline)?;
        }
        Ok(list)
    }
}

impl Parse for InlineText {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let mut parts = Vec::new();
        while !matches!(
            parser.peek()?.kind,
            TokenKind::Newline | TokenKind::RightBrace | TokenKind::Eof
        ) {
            parts.push(Inline::parse(parser)?);
        }
        return Ok(InlineText { parts });
    }
}

// TODO want to fix this at some point, 150 lines of straight hatred below
impl Parse for Inline {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let mut opener = String::new();
        match (parser.peek()?.kind, parser.peek_next()?.kind) {
            (TokenKind::Backtick, _) => {
                opener.push_str(&parser.expect_lexeme(TokenKind::Backtick)?);
                let text = Inline::concat_text(parser, TokenKind::Backtick)?;
                match text {
                    InlineParseResult::Closed(text) => {
                        parser.consume(TokenKind::Backtick)?;
                        Ok(Inline::NerdFont(text))
                    }
                    InlineParseResult::Open(text) => {
                        opener.push_str(&text);
                        Ok(Inline::Text(opener))
                    }
                }
            }
            (TokenKind::Star, TokenKind::Star) => {
                opener.push_str(&parser.expect_lexeme(TokenKind::Star)?);
                opener.push_str(&parser.expect_lexeme(TokenKind::Star)?);
                let text = Inline::concat_bold_text(parser)?;
                match text {
                    InlineParseResult::Closed(text) => Ok(Inline::Bold(text)),
                    InlineParseResult::Open(text) => {
                        opener.push_str(&text);
                        Ok(Inline::Text(opener))
                    }
                }
            }
            (TokenKind::Star, _) => {
                opener.push_str(&parser.expect_lexeme(TokenKind::Star)?);
                let text = Inline::concat_text(parser, TokenKind::Star)?;
                match text {
                    InlineParseResult::Closed(text) => {
                        parser.consume(TokenKind::Star)?;
                        Ok(Inline::Italic(text))
                    }
                    InlineParseResult::Open(text) => {
                        // FIX there is still work to be done here for a single star
                        opener.push_str(&text);
                        Ok(Inline::Text(opener))
                    }
                }
            }
            (TokenKind::LeftBracket, _) => {
                opener.push_str(&parser.expect_lexeme(TokenKind::LeftBracket)?);
                let label_layer =
                    Inline::concat_text(parser, TokenKind::RightBracket)?;
                let label_val = match label_layer {
                    InlineParseResult::Closed(text) => {
                        opener.push_str(&text);
                        text
                    }
                    InlineParseResult::Open(text) => {
                        opener.push_str(&text);
                        return Ok(Inline::Text(opener));
                    }
                };
                opener
                    .push_str(&parser.expect_lexeme(TokenKind::RightBracket)?);
                if !parser.at(TokenKind::LeftParen)? {
                    return Ok(Inline::Text(opener));
                }
                opener.push_str(&parser.expect_lexeme(TokenKind::LeftParen)?);
                let href = Inline::concat_text(parser, TokenKind::RightParen)?;
                match href {
                    InlineParseResult::Closed(text) => {
                        parser.consume(TokenKind::RightParen)?;
                        Ok(Inline::Link {
                            label: label_val,
                            href: text,
                        })
                    }
                    InlineParseResult::Open(text) => {
                        opener.push_str(&text);
                        Ok(Inline::Text(opener))
                    }
                }
            }
            _ => {
                let mut text = String::new();
                while !matches!(
                    parser.peek()?.kind,
                    TokenKind::Star
                        | TokenKind::Backtick
                        | TokenKind::LeftBracket
                        | TokenKind::Newline
                        | TokenKind::Eof
                ) {
                    text.push_str(&parser.consume_lexeme()?);
                }

                Ok(Inline::Text(text))
            }
        }
    }
}

enum InlineParseResult {
    Closed(String),
    Open(String),
}

impl Inline {
    fn concat_bold_text(
        parser: &mut Parser,
    ) -> Result<InlineParseResult, CompilerDiagnostic> {
        let mut text = String::new();

        while !parser.at(TokenKind::Eof)? && !parser.at(TokenKind::Newline)? {
            if parser.at(TokenKind::Star)? {
                let star = parser.expect_lexeme(TokenKind::Star)?;
                if parser.at(TokenKind::Star)? {
                    parser.consume(TokenKind::Star)?;
                    return Ok(InlineParseResult::Closed(text));
                }
                text.push_str(&star);
            } else {
                text.push_str(&parser.consume_lexeme()?);
            }
        }

        Ok(InlineParseResult::Open(text))
    }

    fn concat_text(
        parser: &mut Parser,
        delimiter: TokenKind,
    ) -> Result<InlineParseResult, CompilerDiagnostic> {
        let mut text = String::new();
        while !parser.at(delimiter)?
            && !parser.at(TokenKind::Eof)?
            && !parser.at(TokenKind::Newline)?
            && !parser.at(TokenKind::RightBrace)?
        {
            if parser.at(TokenKind::Backslash)? {
                parser.consume(TokenKind::Backslash)?;
                text.push_str(&parser.consume_lexeme()?);
            } else {
                text.push_str(&parser.consume_lexeme()?);
            }
        }
        if parser.at(delimiter)? {
            Ok(InlineParseResult::Closed(text))
        } else {
            Ok(InlineParseResult::Open(text))
        }
    }
}

/////////////////////////////
///                       ///
/// CONTENT PARSING TESTS ///
///                       ///
/////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_inline(source: &str) -> Result<InlineText, CompilerDiagnostic> {
        let file = "inline-test.pyr".to_string();
        let mut parser = Parser::new(file, source.to_string())?;
        InlineText::parse(&mut parser)
    }

    fn parse_content(source: &str) -> Result<Content, CompilerDiagnostic> {
        let file = "content-block-test.pyr".to_string();
        let mut parser = Parser::new(file, source.to_string())?;
        Content::parse(&mut parser)
    }

    #[test]
    fn parses_adjacent_italic_and_bold_text() {
        let parsed =
            parse_inline("*one***two**").expect("inline text should parse");

        assert_eq!(
            parsed.parts,
            vec![
                Inline::Italic("one".to_string()),
                Inline::Bold("two".to_string()),
            ]
        );
    }

    #[test]
    fn parses_formatting_without_surrounding_whitespace() {
        let parsed = parse_inline("**bold**and*italic*")
            .expect("inline text should parse");

        assert_eq!(
            parsed.parts,
            vec![
                Inline::Bold("bold".to_string()),
                Inline::Text("and".to_string()),
                Inline::Italic("italic".to_string()),
            ]
        );
    }

    #[test]
    fn preserves_spaces_and_punctuation_inside_formatting() {
        let parsed = parse_inline("*multiple words, with spaces!*")
            .expect("inline text should parse");

        assert_eq!(
            parsed.parts,
            vec![Inline::Italic("multiple words, with spaces!".to_string())]
        );
    }

    #[test]
    fn parses_link_between_plain_text() {
        let parsed = parse_inline(
            "see [the documentation](https://example.com/docs) now",
        )
        .expect("inline text should parse");

        assert_eq!(
            parsed.parts,
            vec![
                Inline::Text("see ".to_string()),
                Inline::Link {
                    label: "the documentation".to_string(),
                    href: "https://example.com/docs".to_string(),
                },
                Inline::Text(" now".to_string()),
            ]
        );
    }

    #[test]
    fn parses_single_bulleted_list_item() {
        let parsed = parse_content("- one").expect("list should parse");
        assert_eq!(parsed.blocks.len(), 1);
        assert!(matches!(parsed.blocks[0], ContentBlock::BulletList(_)));
        let parts = match parsed.blocks[0].clone() {
            ContentBlock::BulletList(parts) => parts,
            _ => unreachable!(),
        };

        let inline1 = parts[0].parts.to_owned();
        assert_eq!(inline1, vec![Inline::Text("one".to_string())]);
    }

    #[test]
    fn parses_bulleted_list() {
        let parsed =
            parse_content("- item 1\n- item 2").expect("list should parse");
        assert_eq!(parsed.blocks.len(), 1);
        assert!(matches!(parsed.blocks[0], ContentBlock::BulletList(_)));
        let parts = match parsed.blocks[0].clone() {
            ContentBlock::BulletList(parts) => parts,
            _ => unreachable!(),
        };

        let inline1 = parts[0].parts.to_owned();
        let inline2 = parts[1].parts.to_owned();
        assert_eq!(inline1, vec![Inline::Text("item 1".to_string())]);
        assert_eq!(inline2, vec![Inline::Text("item 2".to_string())]);
    }

    #[test]
    fn parses_bulleted_list_and_text() {
        let parsed = parse_content("- one\n- two\n\ntext after")
            .expect("list and text should parse");
        assert!(matches!(parsed.blocks[0], ContentBlock::BulletList(_)));
        let parts = match parsed.blocks[0].clone() {
            ContentBlock::BulletList(parts) => parts,
            _ => unreachable!(),
        };

        let inline1 = parts[0].parts.to_owned();
        let inline2 = parts[1].parts.to_owned();
        assert_eq!(inline1, vec![Inline::Text("one".to_string())]);
        assert_eq!(inline2, vec![Inline::Text("two".to_string())]);
        assert_eq!(parsed.blocks.len(), 2);
        assert!(matches!(parsed.blocks[1], ContentBlock::Paragraph(_)));
    }

    #[test]
    fn parses_bulleted_list_and_more_text() {
        let parsed = parse_content("- one\n- two\ntext after")
            .expect("list and text should parse");
        assert!(matches!(parsed.blocks[0], ContentBlock::BulletList(_)));
        let parts = match parsed.blocks[0].clone() {
            ContentBlock::BulletList(parts) => parts,
            _ => unreachable!(),
        };

        let inline1 = parts[0].parts.to_owned();
        let inline2 = parts[1].parts.to_owned();
        assert_eq!(inline1, vec![Inline::Text("one".to_string())]);
        assert_eq!(inline2, vec![Inline::Text("two".to_string())]);
        assert_eq!(parsed.blocks.len(), 2);
        assert!(matches!(parsed.blocks[1], ContentBlock::Paragraph(_)));
    }

    #[test]
    fn parses_special_rightbrace() {
        let parsed = parse_content("some text\\}")
            .expect("special right brace should be parsed as text");
        assert_eq!(parsed.blocks.len(), 1);
        let text = match parsed.blocks[0].clone() {
            ContentBlock::Paragraph(text) => text,
            _ => unreachable!(),
        };
        assert_eq!(text.parts, vec![Inline::Text("some text\\}".to_string())]);
    }

    #[test]
    fn treats_unclosed_italic_opener_as_text_at_eof() {
        let parsed = parse_inline("*some text")
            .expect("unclosed formatting should fall back to text");

        assert_eq!(parsed.parts, vec![Inline::Text("*some text".to_string())]);
    }

    #[test]
    fn treats_unclosed_italic_opener_as_text_at_newline() {
        let parsed = parse_inline("*some text\nnext line")
            .expect("unclosed formatting should fall back to text");

        assert_eq!(parsed.parts, vec![Inline::Text("*some text".to_string())]);
    }

    #[test]
    fn treats_unclosed_bold_opener_as_text() {
        let parsed = parse_inline("**unclosed bold")
            .expect("unclosed formatting should fall back to text");

        assert_eq!(
            parsed.parts,
            vec![Inline::Text("**unclosed bold".to_string())]
        );
    }

    #[test]
    fn treats_mismatched_bold_delimiter_as_text() {
        let parsed = parse_inline("**mismatched bold*")
            .expect("mismatched formatting should fall back to text");

        assert_eq!(
            parsed.parts,
            vec![Inline::Text("**mismatched bold*".to_string())]
        );
    }

    #[test]
    fn treats_link_without_closing_parenthesis_as_text() {
        let parsed = parse_inline("[label](https://example.com")
            .expect("unclosed link should fall back to text");

        assert_eq!(
            parsed.parts,
            vec![Inline::Text("[label](https://example.com".to_string())]
        );
    }
}
