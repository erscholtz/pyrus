use std::collections::HashMap;
use std::io::SeekFrom;

use crate::ast::{ArgType, DocElement, Expression, TemplateBlock};
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
        println!("{:?}", self.current_token_kind());
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
                todo!("Table support not implemented");
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
                println!("{:?}", self.current_token_kind());
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
        return DocElement::Call {
            name: func_name,
            args: args,
        };
    }
}
