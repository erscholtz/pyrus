use pyrus::ast::{BinaryOp, DocElement, Expression, InterpPart, Statement, UnaryOp};
use pyrus::lexer::lex;
use pyrus::parser::parse;

#[test]
fn test_parse_empty_document() {
    let source = "document { }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    assert!(ast.document.is_some());
    assert!(ast.template.is_none());
    assert!(ast.style.is_none());

    let doc = ast.document.unwrap();
    assert_eq!(doc.elements.len(), 0);
}

#[test]
fn test_parse_empty_template() {
    let source = "template { }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    assert!(ast.template.is_some());
    assert!(ast.document.is_none());
    assert!(ast.style.is_none());

    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 0);
}

#[test]
fn test_parse_empty_style() {
    let source = "style { }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    assert!(ast.style.is_some());
    assert!(ast.template.is_none());
    assert!(ast.document.is_none());
}

#[test]
fn test_parse_all_blocks() {
    let source = "template { } document { } style { }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    assert!(ast.template.is_some());
    assert!(ast.document.is_some());
    assert!(ast.style.is_some());
}

#[test]
fn test_parse_variable_assignment() {
    let source = "template { let x = \"hello\" }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 1);

    match &template.statements[0] {
        Statement::VarAssign { name, value } => {
            assert_eq!(name, "x");
            match value {
                Expression::StringLiteral(s) => assert_eq!(s, "hello"),
                _ => panic!("Expected StringLiteral expression"),
            }
        }
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_const_assignment() {
    let source = "template { const PI = \"3.14\" }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 1);

    match &template.statements[0] {
        Statement::ConstAssign { name, value } => {
            assert_eq!(name, "PI");
            match value {
                Expression::StringLiteral(s) => assert_eq!(s, "3.14"),
                _ => panic!("Expected StringLiteral expression"),
            }
        }
        _ => panic!("Expected ConstAssign statement"),
    }
}

#[test]
fn test_parse_unary_negation() {
    let source = "template { let x = - 42 }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { name, value } => {
            assert_eq!(name, "x");
            match value {
                Expression::Unary {
                    operator,
                    expression: _,
                } => {
                    match operator {
                        UnaryOp::Negate => {}
                        _ => panic!("Expected Negate operator"),
                    }
                    // Inner expression should be parsed
                }
                _ => panic!("Expected Unary expression"),
            }
        }
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_addition() {
    let source = "template { let sum = x + y }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { name, value } => {
            assert_eq!(name, "sum");
            match value {
                Expression::Binary {
                    left,
                    operator,
                    right,
                } => {
                    match operator {
                        BinaryOp::Add => {}
                        _ => panic!("Expected Add operator"),
                    }
                    match (&**left, &**right) {
                        (Expression::Identifier(l), Expression::Identifier(r)) => {
                            assert_eq!(l, "x");
                            assert_eq!(r, "y");
                        }
                        _ => panic!("Expected identifier expressions"),
                    }
                }
                _ => panic!("Expected Binary expression"),
            }
        }
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_subtraction() {
    let source = "template { let diff = a - b }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::Binary { operator, .. } => match operator {
                BinaryOp::Subtract => {}
                _ => panic!("Expected Subtract operator"),
            },
            _ => panic!("Expected Binary expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_multiplication() {
    let source = "template { let product = a * b }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::Binary { operator, .. } => match operator {
                BinaryOp::Multiply => {}
                _ => panic!("Expected Multiply operator"),
            },
            _ => panic!("Expected Binary expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_division() {
    let source = "template { let quotient = a / b }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::Binary { operator, .. } => match operator {
                BinaryOp::Divide => {}
                _ => panic!("Expected Divide operator"),
            },
            _ => panic!("Expected Binary expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_equals() {
    let source = "template { let result = a = b }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::Binary { operator, .. } => match operator {
                BinaryOp::Equals => {}
                _ => panic!("Expected Equals operator"),
            },
            _ => panic!("Expected Binary expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_literal() {
    let source = "template { let msg = \"Hello, World!\" }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::StringLiteral(s) => assert_eq!(s, "Hello, World!"),
            _ => panic!("Expected StringLiteral expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_with_escaped_quote() {
    let source = r#"template { let msg = "foo\"bar" }"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::StringLiteral(s) => assert_eq!(
                s, "foo\"bar",
                "Escaped quote should be preserved as literal quote"
            ),
            _ => panic!("Expected StringLiteral expression, got {:?}", value),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_lex_unterminated_string() {
    let source = r#"template { let msg = "unterminated }"#;
    let tokens = lex(source).expect("Lexing failed");

    assert!(
        !tokens.errors.is_empty(),
        "Should report error for unterminated string"
    );
    assert_eq!(
        tokens.errors[0].message, "Unterminated string literal",
        "Error message should indicate unterminated string"
    );
}

#[test]
fn test_parse_integer_literal() {
    let source = "template { let num = 42 }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::Int(n) => assert_eq!(*n, 42),
            _ => panic!("Expected Int expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_float_literal() {
    let source = "template { let pi = 3.14 }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_return_statement() {
    // Return requires a document element (like text { ... }), not a plain string
    let source = "template { return text { done } }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::Return { doc_element } => match doc_element {
            DocElement::Text { content, .. } => match content {
                Expression::StringLiteral(s) => assert_eq!(s, "done"),
                _ => panic!("Expected StringLiteral content"),
            },
            _ => panic!("Expected Text DocElement in return"),
        },
        _ => panic!("Expected Return statement"),
    }
}

#[test]
fn test_parse_function_declaration() {
    // Function body needs proper document element in return
    let source = "template { func greet(name: string) { return text { Hello } } }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 1);

    match &template.statements[0] {
        Statement::FunctionDecl {
            name,
            args,
            body,
            return_type,
        } => {
            assert_eq!(name, "greet");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].ty, "string");
            assert!(body.len() > 0);
        }
        _ => panic!("Expected FunctionDecl statement"),
    }
}

#[test]
fn test_parse_function_call_no_args() {
    let source = "document { greet() }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let doc = ast.document.unwrap();

    match &doc.elements[0] {
        DocElement::Call { name, args, .. } => {
            assert_eq!(name, "greet");
            assert_eq!(args.len(), 0);
        }
        _ => panic!("Expected Call DocElement"),
    }
}

#[test]
fn test_parse_function_call_with_args() {
    let source = "document { print(\"hello\", \"world\") }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let doc = ast.document.unwrap();

    match &doc.elements[0] {
        DocElement::Call { name, args, .. } => {
            assert_eq!(name, "print");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Call DocElement"),
    }
}

#[test]
fn test_parse_default_set() {
    let source = "template { width = 100 }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::DefaultSet { key, value } => {
            assert_eq!(key, "width");
            match value {
                Expression::Int(n) => assert_eq!(*n, 100),
                _ => panic!("Expected Int expression"),
            }
        }
        _ => panic!("Expected DefaultSet statement"),
    }
}

#[test]
fn test_parse_multiple_statements() {
    let source = "template { let x = 1 let y = 2 let z = 3 }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 3);
}

#[test]
fn test_parse_mixed_statements() {
    let source = "template { let x = 10 const MAX = 100 width = 50 }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 3);

    match &template.statements[0] {
        Statement::VarAssign { .. } => {}
        _ => panic!("Expected VarAssign"),
    }
    match &template.statements[1] {
        Statement::ConstAssign { .. } => {}
        _ => panic!("Expected ConstAssign"),
    }
    match &template.statements[2] {
        Statement::DefaultSet { .. } => {}
        _ => panic!("Expected DefaultSet"),
    }
}

#[test]
fn test_parse_dollar_sign_interpolation() {
    let source = "template { let msg = $ x $ }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::Identifier(id) => {
                assert_eq!(id, "x");
            }
            _ => panic!("Expected Identifier expression"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_nested_template_and_document() {
    let source = "template { func render() { return text { html } } } document { greet() }";
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    assert!(ast.template.is_some());
    assert!(ast.document.is_some());

    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 1);

    let doc = ast.document.unwrap();
    assert_eq!(doc.elements.len(), 1);
}

#[test]
fn test_parse_string_interpolation_simple() {
    let source = r#"template { let msg = "Hello, {name}!" }"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::InterpolatedString(parts) => {
                assert_eq!(parts.len(), 3);
                match &parts[0] {
                    InterpPart::Text(text) => assert_eq!(text, "Hello, "),
                    _ => panic!("Expected Text part"),
                }
                match &parts[1] {
                    InterpPart::Expression(expr) => match expr {
                        Expression::Identifier(id) => assert_eq!(id, "name"),
                        _ => panic!("Expected Identifier expression"),
                    },
                    _ => panic!("Expected Expression part"),
                }
                match &parts[2] {
                    InterpPart::Text(text) => assert_eq!(text, "!"),
                    _ => panic!("Expected Text part"),
                }
            }
            _ => panic!("Expected InterpolatedString, got {:?}", value),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_interpolation_multiple() {
    let source = r#"template { let msg = "{greeting}, {name}!" }"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::InterpolatedString(parts) => {
                assert_eq!(parts.len(), 4);
                // {greeting}
                match &parts[0] {
                    InterpPart::Expression(expr) => match expr {
                        Expression::Identifier(id) => assert_eq!(id, "greeting"),
                        _ => panic!("Expected Identifier"),
                    },
                    _ => panic!("Expected Expression part"),
                }
                // ,
                match &parts[1] {
                    InterpPart::Text(text) => assert_eq!(text, ", "),
                    _ => panic!("Expected Text part"),
                }
                // {name}
                match &parts[2] {
                    InterpPart::Expression(expr) => match expr {
                        Expression::Identifier(id) => assert_eq!(id, "name"),
                        _ => panic!("Expected Identifier"),
                    },
                    _ => panic!("Expected Expression part"),
                }
                // !
                match &parts[3] {
                    InterpPart::Text(text) => assert_eq!(text, "!"),
                    _ => panic!("Expected Text part"),
                }
            }
            _ => panic!("Expected InterpolatedString"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_interpolation_with_number() {
    let source = r#"template { let msg = "Count: {count}" }"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::InterpolatedString(parts) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    InterpPart::Text(text) => assert_eq!(text, "Count: "),
                    _ => panic!("Expected Text part"),
                }
                match &parts[1] {
                    InterpPart::Expression(expr) => match expr {
                        Expression::Identifier(id) => assert_eq!(id, "count"),
                        _ => panic!("Expected Identifier expression"),
                    },
                    _ => panic!("Expected Expression part"),
                }
            }
            _ => panic!("Expected InterpolatedString"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_without_interpolation() {
    // Plain strings without {} should remain as StringLiteral
    let source = r#"template { let msg = "Hello, World!" }"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0] {
        Statement::VarAssign { value, .. } => match value {
            Expression::StringLiteral(s) => assert_eq!(s, "Hello, World!"),
            _ => panic!("Expected StringLiteral for plain string, got {:?}", value),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}
