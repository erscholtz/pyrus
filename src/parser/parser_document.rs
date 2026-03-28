use std::collections::HashMap;

use super::parser::Parser;
use super::parser_err::ParseError;
use crate::ast::{ArgType, DocElement, Expression};
use crate::lexer::TokenKind;

const DOC_SYNC: &[TokenKind] = &[
    TokenKind::At,
    TokenKind::RightBrace, // End of block
    TokenKind::Eof,
];

impl Parser {
    pub fn parse_document_block(&mut self) -> Vec<DocElement> {
        let mut elements: Vec<DocElement> = Vec::new();
        while self.idx < self.toks.kinds.len() {
            // Skip whitespace before checking token type
            while self.current_token_kind() == TokenKind::Whitespace {
                self.advance();
            }

            if self.idx >= self.toks.kinds.len() {
                break;
            }

            match self.current_token_kind() {
                TokenKind::RightBrace => {
                    // Don't consume the brace here - let the caller handle it
                    break;
                }
                TokenKind::Eof => break,
                TokenKind::Identifier => {
                    // Check if this is a function call (Identifier followed by LeftParen)
                    if self.peek() == Some(TokenKind::LeftParen) {
                        let statement = self.parse_document_function_call();
                        elements.push(statement);
                    } else {
                        // Unexpected identifier without @
                        self.errors.push(ParseError::new(
                            format!(
                                "Parse error: expected @ before identifier at {}:{}",
                                self.current_token_line(),
                                self.current_token_col()
                            ),
                            self.current_token_line(),
                            self.current_token_col(),
                        ));
                        self.synchronize(DOC_SYNC);
                        elements.push(DocElement::ErrorLocation {
                            line: self.current_token_line(),
                            col: self.current_token_col(),
                        });
                    }
                }
                _ => {
                    let statement = self.parse_document_element();
                    elements.push(statement);
                }
            }
        }
        elements
    }

    // TODO handle text formatting properly
    // TODO handle markdown formatting properly (bold, italics, etc.)
    // TODO handle code snippets properly
    pub fn parse_document_element(&mut self) -> DocElement {
        // In document context, elements start with @
        // In template context (e.g., return statements), they may not
        if self.current_token_kind() == TokenKind::At {
            self.advance(); // consume @
        }
        match self.current_token_kind() {
            TokenKind::Text => {
                self.advance(); // consume text label
                let attributes = self.parse_style_attributes();
                // Support both [content] and {content} syntax
                let (left_bracket, right_bracket) = match self.current_token_kind() {
                    TokenKind::LeftBracket => (TokenKind::LeftBracket, TokenKind::RightBracket),
                    TokenKind::LeftBrace => (TokenKind::LeftBrace, TokenKind::RightBrace),
                    _ => {
                        self.errors.push(ParseError::new(
                            format!(
                                "Parse error: expected [ or {{ after text at {}:{}",
                                self.current_token_line(),
                                self.current_token_col()
                            ),
                            self.current_token_line(),
                            self.current_token_col(),
                        ));
                        self.synchronize(DOC_SYNC);
                        return DocElement::ErrorLocation {
                            line: self.current_token_line(),
                            col: self.current_token_col(),
                        };
                    }
                };
                self.expect(left_bracket);
                let text_content = self.parse_document_text_content_until(right_bracket);
                self.expect(right_bracket);
                DocElement::Text {
                    content: text_content,
                    attributes,
                }
            }
            TokenKind::List => {
                self.advance(); // consume list label
                let attributes = self.parse_style_attributes();
                let (left_bracket, right_bracket) = match self.current_token_kind() {
                    TokenKind::LeftBracket => (TokenKind::LeftBracket, TokenKind::RightBracket),
                    TokenKind::LeftBrace => (TokenKind::LeftBrace, TokenKind::RightBrace),
                    _ => {
                        self.errors.push(ParseError::new(
                            format!(
                                "Parse error: expected [ or {{ after list at {}:{}",
                                self.current_token_line(),
                                self.current_token_col()
                            ),
                            self.current_token_line(),
                            self.current_token_col(),
                        ));
                        self.synchronize(DOC_SYNC);
                        return DocElement::ErrorLocation {
                            line: self.current_token_line(),
                            col: self.current_token_col(),
                        };
                    }
                };
                self.expect(left_bracket);
                let (list_items, numbered) = self.parse_document_list();
                self.expect(right_bracket);
                DocElement::List {
                    items: list_items,
                    attributes,
                    numbered,
                }
            }
            TokenKind::Image => {
                self.advance(); // consume image label
                let attributes = self.parse_style_attributes();
                let (left_bracket, right_bracket) = match self.current_token_kind() {
                    TokenKind::LeftBracket => (TokenKind::LeftBracket, TokenKind::RightBracket),
                    TokenKind::LeftBrace => (TokenKind::LeftBrace, TokenKind::RightBrace),
                    _ => {
                        self.errors.push(ParseError::new(
                            format!(
                                "Parse error: expected [ or {{ after image at {}:{}",
                                self.current_token_line(),
                                self.current_token_col()
                            ),
                            self.current_token_line(),
                            self.current_token_col(),
                        ));
                        self.synchronize(DOC_SYNC);
                        return DocElement::ErrorLocation {
                            line: self.current_token_line(),
                            col: self.current_token_col(),
                        };
                    }
                };
                self.expect(left_bracket);
                let src = self.parse_document_text_content_until(right_bracket);
                self.expect(right_bracket);
                DocElement::Image { src, attributes }
            }
            TokenKind::Link => {
                todo!("Link support not implemented");
            }
            TokenKind::Table => {
                self.advance(); // consume table label
                let attributes = self.parse_style_attributes();
                let (left_bracket, right_bracket) = match self.current_token_kind() {
                    TokenKind::LeftBracket => (TokenKind::LeftBracket, TokenKind::RightBracket),
                    TokenKind::LeftBrace => (TokenKind::LeftBrace, TokenKind::RightBrace),
                    _ => {
                        self.errors.push(ParseError::new(
                            format!(
                                "Parse error: expected [ or {{ after table at {}:{}",
                                self.current_token_line(),
                                self.current_token_col()
                            ),
                            self.current_token_line(),
                            self.current_token_col(),
                        ));
                        self.synchronize(DOC_SYNC);
                        return DocElement::ErrorLocation {
                            line: self.current_token_line(),
                            col: self.current_token_col(),
                        };
                    }
                };
                self.expect(left_bracket);
                let table = self.parse_document_table();
                self.expect(right_bracket);
                DocElement::Table { table, attributes }
            }
            TokenKind::Section => {
                self.advance(); // consume section label
                let attributes = self.parse_style_attributes();
                let (left_bracket, right_bracket) = match self.current_token_kind() {
                    TokenKind::LeftBracket => (TokenKind::LeftBracket, TokenKind::RightBracket),
                    TokenKind::LeftBrace => (TokenKind::LeftBrace, TokenKind::RightBrace),
                    _ => {
                        self.errors.push(ParseError::new(
                            format!(
                                "Parse error: expected [ or {{ after section at {}:{}",
                                self.current_token_line(),
                                self.current_token_col()
                            ),
                            self.current_token_line(),
                            self.current_token_col(),
                        ));
                        self.synchronize(DOC_SYNC);
                        return DocElement::ErrorLocation {
                            line: self.current_token_line(),
                            col: self.current_token_col(),
                        };
                    }
                };
                self.expect(left_bracket);
                let section_content = self.parse_document_block();
                self.expect(right_bracket);
                DocElement::Section {
                    elements: section_content,
                    attributes,
                }
            }
            TokenKind::Identifier => {
                // function call
                self.parse_document_function_call()
            }
            _ => {
                self.errors.push(ParseError::new(
                    format!(
                        "Parse error: unexpected token while parsing document element at {}:{}: found {:?}",
                        self.current_token_line(),
                        self.current_token_col(),
                        self.current_token_kind(),
                    ),
                    self.current_token_line(),
                    self.current_token_col(),
                ));
                self.synchronize(DOC_SYNC);
                DocElement::ErrorLocation {
                    line: self.current_token_line(),
                    col: self.current_token_col(),
                }
            }
        }
    }

    fn parse_style_attributes(&mut self) -> HashMap<String, Expression> {
        let mut attributes = HashMap::new();
        if self.current_token_kind() == TokenKind::LeftParen {
            // arg attributes present
            self.advance(); // consume left paren
            while self.current_token_kind() != TokenKind::RightParen {
                let name = self.current_text();
                self.advance(); // consume identifier
                self.expect(TokenKind::Equals);
                let value = self.parse_expression();
                attributes.insert(name, value);
                if self.current_token_kind() == TokenKind::Comma {
                    self.advance(); // consume comma
                }
            }
            self.expect(TokenKind::RightParen); // consume right paren
        }
        attributes
    }

    fn parse_document_text_content(&mut self) -> Expression {
        self.parse_document_text_content_until(TokenKind::RightBracket)
    }

    fn parse_document_text_content_until(&mut self, end_token: TokenKind) -> Expression {
        let mut content = String::new();
        let mut has_interpolation = false;

        while self.current_token_kind() != end_token {
            match self.current_token_kind() {
                TokenKind::Whitespace => {
                    content.push(' ');
                    self.advance();
                }
                TokenKind::StringLiteral(idx) => {
                    let idx_usize = idx as usize;
                    let entry = self.toks.string_table[idx_usize].clone();
                    content.push_str(&entry.content);
                    if entry.has_interpolation {
                        has_interpolation = true;
                    }
                    self.advance();
                }
                TokenKind::Dollarsign => {
                    self.advance(); // consume $
                    if self.current_token_kind() != TokenKind::LeftBrace {
                        content.push('$');
                        content.push_str(&self.current_text());
                        self.advance();
                        continue;
                    }
                    // This is ${...} interpolation
                    has_interpolation = true;
                    content.push_str("${");
                    self.advance(); // consume {

                    // Collect the expression inside ${...}
                    let mut expr_content = String::new();
                    let mut brace_depth = 1;

                    while brace_depth > 0
                        && self.current_token_kind() != TokenKind::Eof
                        && self.current_token_kind() != end_token
                    {
                        match self.current_token_kind() {
                            TokenKind::LeftBrace => {
                                brace_depth += 1;
                                expr_content.push_str(&self.current_text());
                                self.advance();
                            }
                            TokenKind::RightBrace => {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    self.advance();
                                    break;
                                }
                                expr_content.push_str(&self.current_text());
                                self.advance();
                            }
                            _ => {
                                expr_content.push_str(&self.current_text());
                                self.advance();
                            }
                        }
                    }
                    content.push_str(&expr_content);
                    content.push('}');
                }
                _ => {
                    let text = self.current_text();
                    content.push_str(&text);
                    self.advance();
                }
            }
        }

        // If we found interpolation, parse it properly
        if has_interpolation {
            self.parse_string_with_interpolation(&content)
        } else {
            Expression::StringLiteral(content)
        }
    }

    fn parse_document_list(&mut self) -> (Vec<DocElement>, bool) {
        let mut items = Vec::new();
        let mut numbered = false;
        while self.current_token_kind() != TokenKind::RightBracket {
            match self.current_token_kind() {
                TokenKind::Minus => {
                    self.advance();
                    let content = self.parse_document_text_content();
                    items.push(DocElement::Text {
                        content,
                        attributes: HashMap::new(),
                    });
                }
                TokenKind::Int => {
                    self.advance();
                    self.expect(TokenKind::Dot);
                    let content = self.parse_document_text_content();
                    items.push(DocElement::Text {
                        content,
                        attributes: HashMap::new(),
                    });
                    numbered = true;
                }
                _ => {
                    self.errors.push(ParseError::new(
                        format!(
                            "Unexpected token '{}' at {}:{}",
                            self.current_text(),
                            self.current_token_line(),
                            self.current_token_col()
                        ),
                        self.current_token_line(),
                        self.current_token_col(),
                    ));
                    self.synchronize(DOC_SYNC);
                }
            }
        }
        (items, numbered)
    }

    fn parse_document_table(&mut self) -> Vec<Vec<DocElement>> {
        let mut table = Vec::new();

        while self.current_token_kind() != TokenKind::RightBracket {
            if self.current_token_kind() != TokenKind::Pipe {
                self.errors.push(ParseError::new(
                    format!(
                        "Expected '|' to start table row at {}:{}",
                        self.current_token_line(),
                        self.current_token_col()
                    ),
                    self.current_token_line(),
                    self.current_token_col(),
                ));
                self.synchronize(DOC_SYNC);
                break;
            }
            let row = self.parse_document_table_row();
            table.push(row);
        }
        table
    }

    fn parse_document_table_row(&mut self) -> Vec<DocElement> {
        let mut row = Vec::new();
        while self.current_token_kind() == TokenKind::Pipe {
            self.advance(); // consume '|'
            if self.current_token_kind() == TokenKind::Pipe
                || self.current_token_kind() == TokenKind::RightBracket
            {
                row.push(DocElement::Text {
                    content: Expression::StringLiteral(String::new()),
                    attributes: HashMap::new(),
                });
                continue;
            }
            if self.current_token_kind() == TokenKind::Minus
                || self.current_token_kind() == TokenKind::Colon
            {
                let cell_content = self.parse_table_delimiter_cell();
                row.push(DocElement::Text {
                    content: cell_content,
                    attributes: HashMap::new(),
                });
            } else if self.current_token_kind() == TokenKind::At {
                // DSL-style cell: @text[...] or other document element
                let cell = self.parse_document_element();
                row.push(cell);
            }
        }
        row
    }

    fn parse_table_delimiter_cell(&mut self) -> Expression {
        let mut content = String::new();
        if self.current_token_kind() == TokenKind::Colon {
            content.push(':');
            self.advance();
        }
        if self.current_token_kind() != TokenKind::Minus {
            return Expression::StringLiteral(content);
        }
        while self.current_token_kind() == TokenKind::Minus {
            content.push('-');
            self.advance();
        }
        if self.current_token_kind() == TokenKind::Colon {
            content.push(':');
            self.advance();
        }
        Expression::StringLiteral(content)
    }

    fn parse_document_function_call(&mut self) -> DocElement {
        if self.toks.kinds.get(self.idx + 1) != Some(&TokenKind::LeftParen) {
            // check if it is a function call
            self.errors.push(ParseError::new(
                format!(
                    "Expected '(' after function name at {}:{}",
                    self.current_token_line(),
                    self.current_token_col()
                ),
                self.current_token_line(),
                self.current_token_col(),
            ));
            self.synchronize(DOC_SYNC);
            return DocElement::ErrorLocation {
                line: self.current_token_line(),
                col: self.current_token_col(),
            };
        }
        // function call
        let func_name = self.current_text();
        self.advance(); // consume function name
        self.expect(TokenKind::LeftParen);
        let mut args: Vec<ArgType> = Vec::new();
        while self.current_token_kind() != TokenKind::RightParen {
            let name = self.current_text();
            // TODO, bad form but woirking for right now
            let ty;
            if name.starts_with('"') {
                ty = "string";
            } else if let Ok(_) = name.parse::<i32>() {
                ty = "int";
            } else if let Ok(_) = name.parse::<f64>() {
                ty = "float";
            } else {
                ty = "var";
            }
            self.advance(); // consume arg name
            if self.current_token_kind() == TokenKind::Comma {
                args.push(ArgType {
                    name: name,
                    ty: ty.to_string(),
                });
                self.advance(); // consume comma
                continue;
            } else if self.current_token_kind() == TokenKind::RightParen {
                args.push(ArgType {
                    name: name,
                    ty: ty.to_string(),
                });
                break;
            } else {
                self.errors.push(ParseError::new(
                    format!(
                        "Unexpected token '{}' at {}:{}",
                        self.current_text(),
                        self.current_token_line(),
                        self.current_token_col()
                    ),
                    self.current_token_line(),
                    self.current_token_col(),
                ));
                self.synchronize(DOC_SYNC);
                break;
            }
        }
        self.expect(TokenKind::RightParen);

        // Check if there's a children block [...] for custom elements
        let mut children = Vec::new();
        if self.current_token_kind() == TokenKind::LeftBracket {
            self.expect(TokenKind::LeftBracket);

            while self.current_token_kind() != TokenKind::RightBracket {
                children.push(self.parse_document_element());
                if self.current_token_kind() == TokenKind::Comma {
                    self.advance(); // consume comma
                }
            }
            self.expect(TokenKind::RightBracket);
            // For now, return as a Section containing the children
            // TODO: Create a new DocElement variant that includes both the call and children
        }

        return DocElement::Call {
            name: func_name,
            args: args,
            children: children,
        };
    }
}
