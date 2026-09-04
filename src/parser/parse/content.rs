use crate::ast::Content;
use crate::ast::ContentBlock;
use crate::ast::Inline;
use crate::ast::InlineText;
use crate::diagnostic::CompilerDiagnostic;
use crate::parser::Parse;
use crate::parser::Parser;
use crate::tokens::TokenKind;

impl Parse for Content {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let mut blocks = Vec::new();
        while !parser.at(TokenKind::RightBrace) {
            blocks.push(ContentBlock::parse(parser)?);
        }
        Ok(Content { blocks })
    }
}

impl Parse for ContentBlock {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        match parser.current.kind {
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
    // TODO split on newline + dash sequentially
    fn consume_list(
        parser: &mut Parser,
    ) -> Result<Vec<InlineText>, CompilerDiagnostic> {
        let mut list = Vec::new();
        while parser.at(TokenKind::Newline) {
            list.push(InlineText::parse(parser)?);
        }
        Ok(list)
    }
}

impl Parse for InlineText {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let mut parts = Vec::new();
        while !matches!(
            parser.current_kind(),
            TokenKind::Newline | TokenKind::RightBrace | TokenKind::Eof
        ) {
            parts.push(Inline::parse(parser)?);
        }
        return Ok(InlineText { parts });
    }
}

impl Parse for Inline {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic> {
        let mut opener = String::new();
        match parser.current_kind() {
            TokenKind::Backtick => {
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
            TokenKind::Star => {
                opener.push_str(&parser.expect_lexeme(TokenKind::Star)?);
                if parser.at(TokenKind::Star) {
                    opener.push_str(&parser.expect_lexeme(TokenKind::Star)?);
                    let text = Inline::concat_bold_text(parser)?;
                    match text {
                        InlineParseResult::Closed(text) => {
                            Ok(Inline::Bold(text))
                        }
                        InlineParseResult::Open(text) => {
                            opener.push_str(&text);
                            Ok(Inline::Text(opener))
                        }
                    }
                } else {
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
            }
            TokenKind::LeftBracket => {
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
                if !parser.at(TokenKind::LeftParen) {
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
                    parser.current_kind(),
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

        while !parser.at(TokenKind::Eof) && !parser.at(TokenKind::Newline) {
            if parser.at(TokenKind::Star) {
                let star = parser.expect_lexeme(TokenKind::Star)?;
                if parser.at(TokenKind::Star) {
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
        while !parser.at(delimiter)
            && !parser.at(TokenKind::Eof)
            && !parser.at(TokenKind::Newline)
        {
            text.push_str(&parser.consume_lexeme()?);
        }
        if parser.at(delimiter) {
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
    use crate::lexer::Lexer;

    fn parse_inline(source: &str) -> Result<InlineText, CompilerDiagnostic> {
        let file = "inline-test.pyr".to_string();
        let lexer = Lexer::new(file.clone(), source.to_string());
        let mut parser = Parser::new(file, lexer)?;
        InlineText::parse(&mut parser)
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
