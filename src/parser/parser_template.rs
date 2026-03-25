use crate::ast::Statement;
use crate::lexer::TokenKind;
use crate::parser::parser::Parser;
use crate::parser::parser_err::ParseError;

impl Parser {
    pub fn parse_template_block(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        while self.idx < self.toks.kinds.len() {
            match self.current_token_kind() {
                TokenKind::RightBrace => {
                    self.advance(); // exit block
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
        match self.current_token_kind() {
            TokenKind::Identifier => {
                let varname = self.current_text();
                self.advance();
                self.expect(TokenKind::Equals);
                let expr = self.parse_expression();
                Statement::DefaultSet {
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
                Statement::VarAssign {
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
                Statement::ConstAssign {
                    name: varname,
                    value: expr,
                }
            }
            TokenKind::At => {
                let element = self.parse_document_element();
                Statement::DocElementEmit { element }
            }
            TokenKind::Return => {
                self.advance(); // consume 'return'
                match self.current_token_kind() {
                    // TODO add the other types of return types later, for rigt now only returning DocElements
                    _ => {
                        let return_value = self.parse_document_element();
                        Statement::Return {
                            doc_element: return_value,
                        }
                    }
                }
            }
            TokenKind::Children => {
                self.advance(); // consume 'children'
                Statement::Children {
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
                ));
                Statement::ErrorLocation {
                    line: self.current_token_line(),
                    col: self.current_token_col(),
                }
            }
        }
    }

    fn parse_func_decl(&mut self) -> Statement {
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

        Statement::FunctionDecl {
            name,
            args,
            body,
            return_type: return_type,
        }
    }

    fn parse_element_decl(&mut self) -> Statement {
        self.expect(TokenKind::Element);

        self.expect(TokenKind::Identifier);
        let name = self.toks.source[self.toks.ranges[self.idx - 1].clone()].to_string();

        self.expect(TokenKind::LeftParen);
        let args = self.parse_args();

        self.expect(TokenKind::LeftBrace);
        let body = self.parse_decl_body();

        Statement::ElementDecl { name, args, body }
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
                    ));
                    panic!("{:?}", self.errors.last().unwrap());
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
                    ));
                    panic!("{:?}", self.errors.last().unwrap());
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
