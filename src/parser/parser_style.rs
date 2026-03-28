use super::parser::Parser;

use crate::ast::{Expression, KeyValue, Selector, StyleRule};
use crate::lexer::TokenKind;

impl Parser {
    pub fn parse_style_block(&mut self) -> Vec<StyleRule> {
        let mut rules: Vec<StyleRule> = Vec::new();
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
                    self.advance(); // exit block
                    break;
                }
                TokenKind::Eof => break,
                _ => {
                    let statement = self.parse_style_rule();
                    rules.push(statement);
                }
            }
        }
        rules
    }
    pub fn parse_style_rule(&mut self) -> StyleRule {
        let selectors = self.parse_selector_list();
        let declarations = self.parse_style_declarations();
        StyleRule::new(selectors, declarations)
    }

    pub fn parse_selector_list(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        while self.idx < self.toks.kinds.len() {
            match self.current_token_kind() {
                TokenKind::Comma => {
                    self.advance(); // skip comma
                }
                TokenKind::LeftBrace => {
                    self.advance(); // exit selector list
                    break;
                }
                TokenKind::Eof => break,
                _ => {
                    let selector = match self.current_text().as_str() {
                        "." => {
                            self.advance();
                            Selector::Class(self.current_text().to_string())
                        }
                        "#" => {
                            self.advance();
                            Selector::Id(self.current_text().to_string())
                        }
                        _ => {
                            // TODO: have a check to make sure the type is valid CSS type
                            Selector::Type(self.current_text().to_string())
                        }
                    };
                    selectors.push(selector);
                    self.advance();
                }
            }
        }
        selectors
    }

    pub fn parse_style_declarations(&mut self) -> Vec<KeyValue> {
        let mut declarations = Vec::new();
        while self.idx < self.toks.kinds.len() {
            match self.current_token_kind() {
                TokenKind::Semicolon => {
                    self.advance(); // skip semicolon
                }
                TokenKind::RightBrace => {
                    self.advance(); // exit declaration block
                    break;
                }
                TokenKind::Eof => break,
                _ => {
                    // Parse property name (can be hyphenated like "font-size")
                    let mut property: String = String::new();
                    while self.current_token_kind() != TokenKind::Colon {
                        property.push_str(&self.current_text().to_string());
                        self.advance();
                    }
                    self.advance(); // skip colon

                    let value = self.parse_css_value();

                    declarations.push(KeyValue {
                        key: property.trim().to_string(),
                        value,
                    });
                }
            }
        }
        declarations
    }

    fn parse_css_value(&mut self) -> crate::ast::Expression {
        let mut parts = Vec::new();

        while self.idx < self.toks.kinds.len() {
            match self.current_token_kind() {
                TokenKind::Semicolon | TokenKind::RightBrace => break,
                TokenKind::Eof => break,
                _ => {
                    parts.push(self.current_text().to_string());
                    self.advance();
                }
            }
        }

        if parts.is_empty() {
            return Expression::StringLiteral(String::new());
        }

        let mut result = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                // Check if previous part ends with digit and current is a unit
                let prev = &parts[i - 1];
                let prev_ends_digit = prev
                    .chars()
                    .last()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false);
                let is_unit =
                    ["px", "pt", "mm", "cm", "in", "em", "rem", "%"].contains(&part.as_str());

                if prev_ends_digit && is_unit {
                } else {
                    result.push(' ');
                }
            }
            result.push_str(part);
        }

        Expression::StringLiteral(result)
    }
}
