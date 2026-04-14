use std::collections::HashMap;

use crate::{
    ast::{
        CallElem, ChildrenElem, DocElem, DocElemEmitStmt, DocElemKind, Expr, ImageElem,
        InterpolatedStringExpr, LinkElem, ListElem, SectionElem, TableElem, TextElem,
    },
    lexer::TokenKind,
    parser::parse::Parse,
    parser::parser::Parser,
    parser::parser_err::ParseError,
    util::Spanned,
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

impl Parse for TextElem {
    /// Parses a text element,
    ///
    /// `@Text() [ hello ]` -> `TextElem { content: "hello", attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        let location = p.cursor.location();
        p.cursor.expect(TokenKind::Text)?;
        let content = Expr::parse(p)?;
        Ok(Self {
            content,
            attributes: HashMap::new(),
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

        // attributes

        p.cursor.expect(TokenKind::LeftBracket)?;
        let src = Expr::parse(p)?.to_string();
        let attributes = HashMap::new();
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

        //attributes

        p.cursor.expect(TokenKind::LeftBracket)?;

        let items = match p.parse_until::<DocElem>(TokenKind::RightBracket) {
            Ok(items) => items,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };

        Ok(Self {
            items,
            attributes: HashMap::new(),
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
        Err(ParseError::new(
            "CallElem parsing is not implemented yet".to_string(),
            p.cursor.location(),
        ))
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
    /// `@Section("title", [child1, child2])` -> `SectionElem { title: "title", children: [child1, child2], attributes: HashMap::new() }`.
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        p.cursor.expect(TokenKind::Section)?;

        //attributes

        p.cursor.expect(TokenKind::LeftBracket)?;

        let elements = match p.parse_until::<DocElem>(TokenKind::RightBracket) {
            Ok(elements) => elements,
            Err(errors) => return Err(errors.into_iter().next().unwrap()),
        };

        Ok(Self {
            elements,
            attributes: HashMap::new(),
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}

impl Parse for ChildrenElem {
    // if this isnt present render children defaults to false
    fn parse(p: &mut Parser) -> Result<Self, ParseError> {
        Ok(Self {
            render_childen: true,
        })
    }

    fn try_parse(p: &mut Parser) -> Option<Self> {
        Self::parse(p).ok()
    }
}
