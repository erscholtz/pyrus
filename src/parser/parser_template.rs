use super::parser::Parser;
use super::parser_err::ParseError;
use crate::ast::{Statement, StatementKind};
use crate::diagnostic::SourceLocation;
use crate::lexer::TokenKind;
use crate::util::Spanned;

const TEMPLATE_SYNC: &[TokenKind] = &[
    TokenKind::RightBrace,
    TokenKind::Func,
    TokenKind::Element,
    TokenKind::Eof,
];

impl Parser {
    pub fn parse_template_block(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
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
                    break;
                }
                TokenKind::Func => {
                    let statement = self.parse_func_decl();
                    statements.push(statement);
                }
                TokenKind::Element => {
                    let statement = self.parse_element_decl();
                    statements.push(statement);
                }
                TokenKind::Eof => break,
                _ => {
                    let statement = self.parse_statement();
                    statements.push(statement);
                }
            }
        }
        statements
    }

    fn parse_statement(&mut self) -> Statement {
        let start_line = self.current_token_line();
        let start_col = self.current_token_col();

        let kind = match self.current_token_kind() {
            TokenKind::Identifier => {
                let varname = self.current_text();
                self.advance();
                self.expect(TokenKind::Equals);
                let expr = self.parse_expression();
                StatementKind::DefaultSet {
                    key: varname,
                    value: expr,
                }
            }
            TokenKind::Let => {
                self.advance();
                let varname = self.current_text();
                self.advance();
                self.expect(TokenKind::Equals);
                let expr = self.parse_expression();
                StatementKind::VarAssign {
                    name: varname,
                    value: expr,
                }
            }
            TokenKind::Const => {
                self.advance();
                let varname = self.current_text();
                self.advance();
                self.expect(TokenKind::Equals);
                let expr = self.parse_expression();
                StatementKind::ConstAssign {
                    name: varname,
                    value: expr,
                }
            }
            TokenKind::At => {
                let element = self.parse_document_element();
                StatementKind::DocElementEmit { element }
            }
            TokenKind::Return => {
                self.advance(); // consume 'return'
                match self.current_token_kind() {
                    // TODO add the other types of return types later, for rigt now only returning DocElements
                    _ => {
                        let return_value = self.parse_document_element();
                        StatementKind::Return {
                            doc_element: return_value,
                        }
                    }
                }
            }
            TokenKind::Children => {
                self.advance(); // consume 'children'
                StatementKind::Children {
                    children: "RENDER_CHILDREN".to_string(),
                }
            }
            // TODO handle if statements
            // TODO handle for loops
            // TODO handle while loops
            _ => {
                self.errors.push(ParseError::new(
                    format!(
                        "Parse error: unexpected token parsing statement. Found: {:?} at {}:{}",
                        self.current_token_kind(),
                        self.current_token_line(),
                        self.current_token_col()
                    ),
                    self.current_token_line(),
                    self.current_token_col(),
                    self.file.clone(),
                ));
                self.synchronize(TEMPLATE_SYNC);
                StatementKind::ErrorLocation {
                    line: self.current_token_line(),
                    col: self.current_token_col(),
                }
            }
        };

        let location = SourceLocation::new(start_line, start_col, self.file.clone());
        Spanned::new(kind, location)
    }

    fn parse_func_decl(&mut self) -> Statement {
        let start_line = self.current_token_line();
        let start_col = self.current_token_col();

        self.expect(TokenKind::Func);

        self.expect(TokenKind::Identifier);
        let name = self.toks.source[self.toks.ranges[self.idx - 1].clone()].to_string();

        self.expect(TokenKind::LeftParen);
        let args = self.parse_args();

        // Optional return type annotation: -> Type
        let return_type = if self.current_token_kind() == TokenKind::Minus
            && self.peek() == Some(TokenKind::Greater)
        {
            self.advance(); // consume -
            self.advance(); // consume >
            let ty = self.current_text().to_string();
            self.advance(); // consume type
            Some(ty)
        } else {
            None
        };

        self.expect(TokenKind::LeftBrace);
        let body = self.parse_decl_body();
        self.expect(TokenKind::RightBrace); // consume function's closing brace

        let kind = StatementKind::FunctionDecl {
            name,
            args,
            body,
            return_type: return_type,
        };

        let location = SourceLocation::new(start_line, start_col, self.file.clone());
        Spanned::new(kind, location)
    }

    fn parse_element_decl(&mut self) -> Statement {
        let start_line = self.current_token_line();
        let start_col = self.current_token_col();

        self.expect(TokenKind::Element);

        self.expect(TokenKind::Identifier);
        let name = self.toks.source[self.toks.ranges[self.idx - 1].clone()].to_string();

        self.expect(TokenKind::LeftParen);
        let args = self.parse_args();

        self.expect(TokenKind::LeftBrace);
        let body = self.parse_decl_body();
        self.expect(TokenKind::RightBrace); // consume element's closing brace

        let kind = StatementKind::ElementDecl { name, args, body };

        let location = SourceLocation::new(start_line, start_col, self.file.clone());
        Spanned::new(kind, location)
    }

    fn parse_args(&mut self) -> Vec<crate::ast::FuncParam> {
        let mut params = Vec::new();
        loop {
            // I dont really like the loop keyword, but I like warnings even less
            match self.current_token_kind() {
                TokenKind::RightParen => break,
                TokenKind::Identifier => {
                    let param_name = self.parse_expression();
                    self.expect(TokenKind::Colon);
                    let param_type = self.current_text();
                    self.advance();
                    params.push(crate::ast::FuncParam {
                        ty: param_type,
                        value: param_name,
                    });
                    self.match_kind(TokenKind::Comma);
                }
                _ => {
                    self.errors.push(ParseError::new(
                        format!(
                            "Parse error: unexpected token parsing function argument. Found: {:?} at {}:{}",
                            self.current_token_kind(),
                            self.current_token_line(),
                            self.current_token_col()
                        ),
                        self.current_token_line(),
                        self.current_token_col(),
                        self.file.clone(),
                    ));
                    self.synchronize(TEMPLATE_SYNC);
                    break;
                }
            }
        }
        self.expect(TokenKind::RightParen);
        params
    }

    fn parse_decl_body(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        while self.current_token_kind() != TokenKind::RightBrace {
            match self.current_token_kind() {
                TokenKind::RightBrace => {
                    self.expect(TokenKind::RightBrace);
                    break;
                }
                TokenKind::Eof => {
                    self.errors.push(ParseError::new(
                        format!(
                            "Parse error: unexpected end of file while parsing block at {}:{}",
                            self.current_token_line(),
                            self.current_token_col()
                        ),
                        self.current_token_line(),
                        self.current_token_col(),
                        self.file.clone(),
                    ));
                    self.synchronize(TEMPLATE_SYNC);
                    break;
                }
                _ => {
                    let statement = self.parse_statement();
                    statements.push(statement);
                }
            }
        }
        statements
    }
}
