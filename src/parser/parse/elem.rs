use std::collections::HashMap;

use crate::{
    ast::{
        ArgType, CallElem, ChildrenElem, DocElem, DocElemKind, Expr, ImageElem, LinkElem, ListElem,
        SectionElem, TableElem, TextElem,
    },
    diagnostic::SyntaxError,
    lexer::TokenKind,
    parser::{Parser, parse::Parse},
};

impl Parse for DocElem {
    /// Parses a document element, e.g. `@Text("hello")`.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        let location = p.cursor.location();
        let node = DocElemKind::parse(p)?;
        Ok(Self { node, location })
    }
}

impl Parse for DocElemKind {
    /// Parses a document element.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
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
                return Err(SyntaxError::UnexpectedToken {
                    location: p.cursor.location(),
                    expected: vec![
                        TokenKind::Text,
                        TokenKind::Image,
                        TokenKind::Table,
                        TokenKind::List,
                        TokenKind::Identifier,
                        TokenKind::Link,
                        TokenKind::Section,
                        TokenKind::Children,
                    ],
                    found: p.cursor.cur_tok().clone(),
                });
            }
        }
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
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
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
}

impl Parse for ImageElem {
    /// Parses an image element,
    ///
    /// `@Image("path/to/image.png")` -> `ImageElem { src: "path/to/image.png", attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Image)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let src = Expr::parse(p)?.to_string();
        p.cursor.expect(TokenKind::RightBracket)?;
        Ok(Self { src, attributes })
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
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Table)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let table = TableElem::parse_table(p)?;
        p.cursor.expect(TokenKind::RightBracket)?;

        Ok(Self { table, attributes })
    }
}

impl TableElem {
    fn parse_table(p: &mut Parser) -> Result<Vec<Vec<DocElem>>, SyntaxError> {
        let mut table = Vec::new();
        // header
        let header: Vec<_> = match p.parse_split_on(
            |p: &mut Parser| {
                p.cursor.cur_tok() == &TokenKind::Pipe
                    && p.cursor.peek_tok() == Some(&TokenKind::Pipe)
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
                    (p.cursor.cur_tok() == &TokenKind::Pipe
                        && p.cursor.peek_tok() == Some(&TokenKind::Pipe))
                        || (p.cursor.cur_tok() == &TokenKind::Pipe
                            && p.cursor.peek_tok() == Some(&TokenKind::RightBracket))
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
                return Err(SyntaxError::InvalidConstruct {
                    location: p.cursor.location(),
                    construct: "table row".to_string(),
                    reason: format!("Expected {} columns, got {}", colum_count, row.len()),
                });
            }
        }

        if p.cursor.cur_tok() == &TokenKind::Pipe
            && p.cursor.peek_tok() == Some(&TokenKind::RightBracket)
        {
            p.cursor.expect(TokenKind::Pipe)?;
        }

        Ok(table)
    }

    fn parse_divider_row(p: &mut Parser, column_count: usize) -> Result<(), SyntaxError> {
        p.cursor.expect(TokenKind::Pipe)?;
        p.cursor.expect(TokenKind::Pipe)?;

        for _ in 0..column_count {
            let mut dash_count = 0;
            while p.cursor.cur_tok() == &TokenKind::Minus {
                p.cursor.advance();
                dash_count += 1;
            }

            if dash_count == 0 {
                return Err(SyntaxError::UnexpectedToken {
                    location: p.cursor.location(),
                    expected: vec![TokenKind::Minus],
                    found: p.cursor.cur_tok().clone(),
                });
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
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::List)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let items = match p.parse_split_on::<DocElem, _, _>(
            |p| p.cursor.cur_tok() == &TokenKind::RightBracket,
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
}

impl Parse for CallElem {
    /// Parses a call element, e.g.
    ///
    /// `@Call("func", [arg1, arg2])` -> `CallElem { name: "func", args: [arg1, arg2], children: [] }`.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        let name = p.cursor.cur_text().to_owned();
        p.cursor.expect(TokenKind::Identifier)?;

        p.cursor.expect(TokenKind::LeftParen)?;
        let args = match p.parse_split_on::<ArgType, _, _>(
            |p| p.cursor.cur_tok() == &TokenKind::RightParen,
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
                |p| p.cursor.cur_tok() == &TokenKind::RightBracket,
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
}

impl Parse for LinkElem {
    /// Parses a link element, e.g.
    ///
    /// `@Link("https://example.com", "Example")` -> `LinkElem { href: "https://example.com", content: "Example", attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Link)?;

        let attributes = DocElemKind::parse_style_attributes(p);

        p.cursor.expect(TokenKind::LeftBracket)?;
        let TokenKind::StringLiteral(idx) = p.cursor.cur_tok().clone() else {
            return Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::StringLiteral(0)],
                found: p.cursor.cur_tok().clone(),
            });
        };
        p.cursor.advance();
        let Some(href) = p.cursor.get_string(idx).cloned() else {
            return Err(SyntaxError::MissingToken {
                location: p.cursor.location(),
                expected: TokenKind::StringLiteral(0),
            });
        };
        p.cursor.expect(TokenKind::Comma)?;
        let TokenKind::StringLiteral(idx) = p.cursor.cur_tok().clone() else {
            return Err(SyntaxError::UnexpectedToken {
                location: p.cursor.location(),
                expected: vec![TokenKind::StringLiteral(0)],
                found: p.cursor.cur_tok().clone(),
            });
        };
        let Some(content) = p.cursor.get_string(idx).cloned() else {
            return Err(SyntaxError::MissingToken {
                location: p.cursor.location(),
                expected: TokenKind::StringLiteral(0),
            });
        };
        p.cursor.advance();
        p.cursor.expect(TokenKind::RightBracket)?;

        Ok(Self {
            href: href.content,
            content: content.content,
            attributes,
        })
    }
}

impl Parse for SectionElem {
    /// Parses a section element, e.g.
    ///
    /// `@Section("title") [ @text("content") [] ]` -> `SectionElem { title: "title", children: [child1, child2], attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
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
}

impl Parse for ChildrenElem {
    /// if this isnt present render children defaults to false
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError> {
        p.cursor.expect(TokenKind::Children)?;
        Ok(Self {
            render_childen: true,
        })
    }
}
