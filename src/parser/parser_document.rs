use std::collections::HashMap;

use crate::ast::{ArgType, DocElement, Expression};
use crate::lexer::TokenKind;
use crate::parser::parser::Parser;
use crate::parser::parser_err::ParseError;

impl Parser {
    pub fn parse_document_block(&mut self) -> Vec<DocElement> {
        let mut elements: Vec<DocElement> = Vec::new();
        while self.idx < self.toks.kinds.len() {
            match self.current_token_kind() {
                TokenKind::RightBrace => {
                    self.advance(); // exit block
                    break;
                }
                TokenKind::Eof => break,
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
        self.expect(TokenKind::At);
        match self.current_token_kind() {
            TokenKind::Text => {
                self.advance(); // consume text label
                let attributes = self.parse_style_attributes();
                self.expect(TokenKind::LeftBracket);
                let text_content = self.parse_document_text_content();
                self.expect(TokenKind::RightBracket);
                DocElement::Text {
                    content: text_content,
                    attributes,
                }
            }
            TokenKind::List => {
                self.advance(); // consume list label
                let attributes = self.parse_style_attributes();
                self.expect(TokenKind::LeftBracket);
                let (list_items, numbered) = self.parse_document_list();
                self.expect(TokenKind::RightBracket);
                DocElement::List {
                    items: list_items,
                    attributes,
                    numbered,
                }
            }
            TokenKind::Image => {
                self.advance(); // consume image label
                let attributes = self.parse_style_attributes();
                self.expect(TokenKind::LeftBracket);
                let src = self.parse_document_text_content();
                self.expect(TokenKind::RightBracket);
                DocElement::Image { src, attributes }
            }
            TokenKind::Link => {
                todo!("Link support not implemented");
            }
            TokenKind::Table => {
                self.advance(); // consume table label
                let attributes = self.parse_style_attributes();
                self.expect(TokenKind::LeftBracket);
                let table = self.parse_document_table();
                self.expect(TokenKind::RightBracket);
                DocElement::Table { table, attributes }
            }
            TokenKind::Section => {
                self.advance(); // consume section label
                let attributes = self.parse_style_attributes();
                self.expect(TokenKind::LeftBracket);
                let section_content = self.parse_document_block();
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
                println!("{:?}", self.errors.last().as_ref());
                self.advance();
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

    fn parse_document_text_content(&mut self) -> String {
        let mut content = String::new();
        while self.current_token_kind() != TokenKind::RightBracket {
            match self.current_token_kind() {
                TokenKind::Dollarsign => {
                    self.advance(); // consume $
                    if self.current_token_kind() != TokenKind::LeftBrace {
                        content.push('$');
                        content.push_str(&self.current_text());

                        self.advance();
                        continue;
                    }
                    content.push('$'); // TODO: string interpolation
                    content.push_str(&self.current_text()); // add {
                    self.advance();
                    content.push_str(&self.current_text()); // add identifier
                    self.advance(); // consume identifier
                    content.push_str(&self.current_text()); // add }
                    self.advance();
                }
                _ => {
                    let text = self.current_text();
                    content.push_str(&text);
                    self.advance();
                }
            }
        }
        content
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
                    content: String::new(),
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

    fn parse_table_delimiter_cell(&mut self) -> String {
        let mut content = String::new();
        if self.current_token_kind() == TokenKind::Colon {
            content.push(':');
            self.advance();
        }
        if self.current_token_kind() != TokenKind::Minus {
            return content;
        }
        while self.current_token_kind() == TokenKind::Minus {
            content.push('-');
            self.advance();
        }
        if self.current_token_kind() == TokenKind::Colon {
            content.push(':');
            self.advance();
        }
        content
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
            self.advance();
            return DocElement::Text {
                content: "error".to_string(),
                attributes: HashMap::new(),
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
