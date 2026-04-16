use std::collections::HashMap;

use printpdf::bridge::display_list_to_printpdf_ops_with_margins;

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
        Err(ParseError::new(
            "TableElem parsing is not implemented yet".to_string(),
            p.cursor.location(),
        ))
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
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
        p.cursor.expect(TokenKind::Minus)?; // first item in the list
        let items = match p.parse_split_on::<DocElem, _>(TokenKind::RightBracket, |p| {
            p.cursor.cur_tok() == &TokenKind::Minus
        }) {
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
        let args = match p.parse_split_on::<ArgType, _>(TokenKind::RightParen, |p| {
            p.cursor.cur_tok() == &TokenKind::Comma
        }) {
            Ok(args) => args,
            Err(err) => return Err(err.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightParen)?;

        p.cursor.expect(TokenKind::LeftBracket)?;
        let children = match p.parse_split_on::<DocElem, _>(TokenKind::RightBracket, |p| {
            p.cursor.cur_tok() == &TokenKind::Comma
        }) {
            Ok(children) => children,
            Err(err) => return Err(err.into_iter().next().unwrap()),
        };
        p.cursor.expect(TokenKind::RightBracket)?;

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
        Err(ParseError::new(
            "LinkElem parsing is not implemented yet".to_string(),
            p.cursor.location(),
        ))
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
