use std::collections::HashMap;

use crate::{
    ast::{
        ArgType, CallElem, ChildrenElem, DocElem, DocElemKind, Expr, ImageElem, LinkElem, ListElem,
        SectionElem, TableElem, TextElem,
    },
    lexer::TokenKind,
    parser::{parse::Parse, parser::Parser, parser_err::ParseError},
};

impl Parse for DocElem {
    /// Parses a document element, e.g. `@Text("hello")`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        let location = p.cursor.location();
        let node = DocElemKind::parse(p)?;
        Ok(Self { node, location })
    }

    /// Tries to parse a document element.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        if p.cursor.check(TokenKind::At) {
            let location = p.cursor.location();
            let node = DocElemKind::try_parse(p)?;
            Some(Self { node, location })
        } else {
            None
        }
    }
}

impl Parse for DocElemKind {
    /// Parses a document element.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::At)?;
        match p.cursor.cur_tok() {
            TokenKind::Text => TextElem::parse(p).map(|s| s.into()),
            TokenKind::Image => ImageElem::parse(p).map(|s| s.into()),
            TokenKind::Table => TableElem::parse(p).map(|s| s.into()),
            TokenKind::List => ListElem::parse(p).map(|s| s.into()),
            TokenKind::Identifier => CallElem::parse(p).map(|s| s.into()),
            TokenKind::Link => LinkElem::parse(p).map(|s| s.into()),
            TokenKind::Section => SectionElem::parse(p).map(|s| s.into()),
            TokenKind::Children => ChildrenElem::parse(p).map(|s| s.into()),
            _ => {
                return Err(ParseError::new(
                    format!("Expected Text, found {:?}", p.cursor.cur_tok()),
                    p.cursor.location(),
                ));
            }
        }
    }

    /// Tries to parse a document element.
    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl DocElemKind {
    fn parse_style_attributes(p: &mut Parser) -> Option<HashMap<String, Expr>> {
        if !p.cursor.check(TokenKind::LeftParen) {
            return None;
        }

        p.cursor.expect(TokenKind::LeftParen).ok()?;
        let mut attributes = HashMap::new();
        while !p.cursor.check(TokenKind::RightParen) {
            let name = p.cursor.cur_text().to_owned();
            p.cursor.advance(); // consume identifier

            p.cursor.expect(TokenKind::Equals).ok()?;
            let value = Expr::parse(p).ok()?;
            attributes.insert(name, value);

            if p.cursor.check(TokenKind::Comma) {
                p.cursor.advance(); // consume comma
            }
        }
        p.cursor.expect(TokenKind::RightParen).ok()?; // consume right paren
        Some(attributes)
    }
}

impl Parse for TextElem {
    /// Parses a text element,
    ///
    /// `@Text() [ hello ]` -> `TextElem { content: "hello", attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Text)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let content = Expr::parse(p)?;
        p.cursor.expect(TokenKind::RightBracket)?;
        Ok(Self {
            content,
            attributes,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for ImageElem {
    /// Parses an image element,
    ///
    /// `@Image("path/to/image.png")` -> `ImageElem { src: "path/to/image.png", attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Image)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let src = Expr::parse(p)?.to_string();
        p.cursor.expect(TokenKind::RightBracket)?;
        Ok(Self { src, attributes })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for TableElem {
    /// Parses a table element, e.g.
    ///
    /// `@Table() [
    ///     | text | text |
    ///     |------|------|
    ///     | text | text |
    /// ]`
    /// -> `TableElem { table: [[text, text], [text, text]], attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Table)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let table = TableElem::parse_table(p)?;
        p.cursor.expect(TokenKind::RightBracket)?;

        Ok(Self { table, attributes })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl TableElem {
    fn parse_table(p: &mut Parser) -> Result<Vec<Vec<DocElem>>, ParseError> {
        let mut table = Vec::new();
        // header
        let header: Vec<_> = match p.parse_split_on(
            |p: &mut Parser| {
                !(p.cursor.cur_tok() == &TokenKind::Pipe
                    && p.cursor.peek_tok() == Some(&TokenKind::Pipe))
            },
            |p: &mut Parser| p.cursor.cur_tok() == &TokenKind::Pipe,
            Some(TokenKind::Pipe),
        ) {
            Ok(header) => header,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        let colum_count = header.len();
        table.push(header);

        // divider (needed for table creation)
        TableElem::parse_divider_row(p, colum_count)?;

        // rows
        while p.cursor.cur_tok() == &TokenKind::Pipe
            && p.cursor.peek_tok() != Some(&TokenKind::RightBracket)
        {
            let row = match p.parse_split_on(
                |p: &mut Parser| {
                    !((p.cursor.cur_tok() == &TokenKind::Pipe
                        && p.cursor.peek_tok() == Some(&TokenKind::Pipe))
                        || (p.cursor.cur_tok() == &TokenKind::Pipe
                            && p.cursor.peek_tok() == Some(&TokenKind::RightBracket)))
                },
                |p: &mut Parser| p.cursor.cur_tok() == &TokenKind::Pipe,
                Some(TokenKind::Pipe),
            ) {
                Ok(row) => row,
                Err(errors) => return Err(errors.into_iter().next().unwrap()),
            };
            if row.len() == colum_count {
                table.push(row);
            } else {
                return Err(ParseError::new(
                    format!("Expected {} columns, got {}", colum_count, row.len()),
                    p.cursor.location(),
                ));
            }
        }

        if p.cursor.cur_tok() == &TokenKind::Pipe
            && p.cursor.peek_tok() == Some(&TokenKind::RightBracket)
        {
            p.cursor.expect(TokenKind::Pipe)?;
        }

        Ok(table)
    }

    fn parse_divider_row(p: &mut Parser, column_count: usize) -> Result<(), ParseError> {
        p.cursor.expect(TokenKind::Pipe)?;
        p.cursor.expect(TokenKind::Pipe)?;

        for _ in 0..column_count {
            let mut dash_count = 0;
            while p.cursor.cur_tok() == &TokenKind::Minus {
                p.cursor.advance();
                dash_count += 1;
            }

            if dash_count == 0 {
                return Err(ParseError::new(
                    "expected at least one '-' in table divider row".to_string(),
                    p.cursor.location(),
                ));
            }

            p.cursor.expect(TokenKind::Pipe)?;
        }

        Ok(())
    }
}

impl Parse for ListElem {
    /// Parses a list element, e.g.
    ///
    /// `@List[
    ///     - item 1
    ///     - item 2
    /// ]`
    /// -> `ListElem { items: [item1, item2], attributes: HashMap::new(), numbered: false }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::List)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let items = match p.parse_split_on::<DocElem, _, _>(
            |p| p.cursor.cur_tok() != &TokenKind::RightBracket,
            |p| p.cursor.cur_tok() == &TokenKind::Minus,
            Some(TokenKind::Minus),
        ) {
            // FIX this might have to be its own funcion due to specific style of lists
            Ok(items) => items,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBracket)?;

        Ok(Self {
            items,
            attributes,
            numbered: false,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for CallElem {
    /// Parses a call element, e.g.
    ///
    /// `@Call("func", [arg1, arg2])` -> `CallElem { name: "func", args: [arg1, arg2], children: [] }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        let name = p.cursor.expect(TokenKind::Identifier)?.to_string();

        p.cursor.expect(TokenKind::LeftParen)?;
        let args = match p.parse_split_on::<ArgType, _, _>(
            |p| p.cursor.cur_tok() != &TokenKind::RightParen,
            |p| p.cursor.cur_tok() == &TokenKind::Comma,
            None,
        ) {
            Ok(args) => args,
            Err(err) => return Err(err.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightParen)?;

        let children = if p.cursor.check(TokenKind::LeftBracket) {
            p.cursor.expect(TokenKind::LeftBracket)?;
            let children = match p.parse_split_on::<DocElem, _, _>(
                |p| p.cursor.cur_tok() != &TokenKind::RightBracket,
                |p| p.cursor.cur_tok() == &TokenKind::Comma,
                None,
            ) {
                Ok(children) => children,
                Err(err) => return Err(err.into_iter().next().unwrap()),
            };
            p.cursor.expect(TokenKind::RightBracket)?;
            Some(children)
        } else {
            None
        };

        Ok(Self {
            name,
            args,
            children,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for LinkElem {
    /// Parses a link element, e.g.
    ///
    /// `@Link("https://example.com", "Example")` -> `LinkElem { href: "https://example.com", content: "Example", attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Link)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let TokenKind::StringLiteral(idx) = p.cursor.cur_tok().clone() else {
            return Err(ParseError::new(
                "Expected a string literal for the href of a link".to_string(),
                p.cursor.location(),
            ));
        };
        p.cursor.advance();
        let Some(href) = p.cursor.get_string(idx).cloned() else {
            return Err(ParseError::new(
                "Expected a string literal for the href of a link".to_string(),
                p.cursor.location(),
            ));
        };
        p.cursor.expect(TokenKind::Comma)?;
        let TokenKind::StringLiteral(idx) = p.cursor.cur_tok() else {
            return Err(ParseError::new(
                "Expected a string literal for the content of a link".to_string(),
                p.cursor.location(),
            ));
        };
        let Some(content) = p.cursor.get_string(*idx).cloned() else {
            return Err(ParseError::new(
                "Expected a string literal for the content of a link".to_string(),
                p.cursor.location(),
            ));
        };
        p.cursor.expect(TokenKind::RightBracket)?;

        Ok(Self {
            href: href.content,
            content: content.content,
            attributes,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for SectionElem {
    /// Parses a section element, e.g.
    ///
    /// `@Section("title") [ @text("content") [] ]` -> `SectionElem { title: "title", children: [child1, child2], attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Section)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let elements = match p.parse_until::<DocElem>(TokenKind::RightBracket) {
            Ok(elements) => elements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBracket)?;

        Ok(Self {
            elements,
            attributes,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for ChildrenElem {
    /// if this isnt present render children defaults to false
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Children)?;
        Ok(Self {
            render_childen: true,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}
