use pyrus::ast::{BinOp, DocElemKind, ExprKind, StmtKind, UnaryOp};
use pyrus::lexer::lex;
use pyrus::parser::parse;

#[test]
fn test_parse_empty_document() {
    let source = "document { }";
    let tokens = lex(source, "test_parse_empty_document").expect("Lexing failed");
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
    let tokens = lex(source, "test_parse_empty_template").expect("Lexing failed");
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
    let tokens = lex(source, "test_parse_empty_style").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    assert!(ast.style.is_some());
    assert!(ast.template.is_none());
    assert!(ast.document.is_none());
}

#[test]
fn test_parse_all_blocks() {
    let source = "template { } document { } style { }";
    let tokens = lex(source, "test_parse_all_blocks").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    assert!(ast.template.is_some());
    assert!(ast.document.is_some());
    assert!(ast.style.is_some());
}

#[test]
fn test_parse_variable_assignment() {
    let source = "template { let x = \"hello\" }";
    let tokens = lex(source, "test_parse_variable_assignment").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 1);

    match &template.statements[0].node {
        StmtKind::VarAssign { name, value } => {
            assert_eq!(name, "x");
            match &value.node {
                ExprKind::StringLiteral(s) => assert_eq!(s, "hello"),
                _ => panic!("Expected StringLiteral expr"),
            }
        }
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_const_assignment() {
    let source = "template { const PI = \"3.14\" }";
    let tokens = lex(source, "test_parse_const_assignment").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 1);

    match &template.statements[0].node {
        StmtKind::ConstAssign { name, value } => {
            assert_eq!(name, "PI");
            match &value.node {
                ExprKind::StringLiteral(s) => assert_eq!(s, "3.14"),
                _ => panic!("Expected StringLiteral expr"),
            }
        }
        _ => panic!("Expected ConstAssign statement"),
    }
}

#[test]
fn test_parse_unary_negation() {
    let source = "template { let x = - 42 }";
    let tokens = lex(source, "test_parse_unary_negation").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { name, value } => {
            assert_eq!(name, "x");
            match &value.node {
                ExprKind::Unary { operator, expr: _ } => {
                    match operator {
                        UnaryOp::Negate => {}
                        _ => panic!("Expected Negate operator"),
                    }
                    // Inner expr should be parsed
                }
                _ => panic!("Expected Unary expr"),
            }
        }
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_addition() {
    let source = "template { let sum = x + y }";
    let tokens = lex(source, "test_parse_binary_addition").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { name, value } => {
            assert_eq!(name, "sum");
            match &value.node {
                ExprKind::Binary {
                    left,
                    operator,
                    right,
                } => {
                    match operator {
                        BinOp::Add => {}
                        _ => panic!("Expected Add operator"),
                    }
                    match (&left.node, &right.node) {
                        (ExprKind::Identifier(l), ExprKind::Identifier(r)) => {
                            assert_eq!(l, "x");
                            assert_eq!(r, "y");
                        }
                        _ => panic!("Expected identifier exprs"),
                    }
                }
                _ => panic!("Expected Bin expr"),
            }
        }
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_subtraction() {
    let source = "template { let diff = a - b }";
    let tokens = lex(source, "test_parse_binary_subtraction").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::Binary { operator, .. } => match operator {
                BinOp::Subtract => {}
                _ => panic!("Expected Subtract operator"),
            },
            _ => panic!("Expected Bin expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_multiplication() {
    let source = "template { let product = a * b }";
    let tokens = lex(source, "test_parse_binary_multiplication").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::Binary { operator, .. } => match operator {
                BinOp::Multiply => {}
                _ => panic!("Expected Multiply operator"),
            },
            _ => panic!("Expected Bin expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_division() {
    let source = "template { let quotient = a / b }";
    let tokens = lex(source, "test_parse_binary_division").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::Binary { operator, .. } => match operator {
                BinOp::Divide => {}
                _ => panic!("Expected Divide operator"),
            },
            _ => panic!("Expected Bin expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_binary_equals() {
    let source = "template { let result = a = b }";
    let tokens = lex(source, "test_parse_binary_equals").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::Binary { operator, .. } => match operator {
                BinOp::Equals => {}
                _ => panic!("Expected Equals operator"),
            },
            _ => panic!("Expected Bin expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_literal() {
    let source = "template { let msg = \"Hello, World!\" }";
    let tokens = lex(source, "test_parse_string_literal").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::StringLiteral(s) => assert_eq!(s, "Hello, World!"),
            _ => panic!("Expected StringLiteral expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_with_escaped_quote() {
    let source = r#"template { let msg = "foo\"bar" }"#;
    let tokens = lex(source, "test_parse_string_with_escaped_quote").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::StringLiteral(s) => assert_eq!(
                s, "foo\"bar",
                "Escaped quote should be preserved as literal quote"
            ),
            _ => panic!("Expected StringLiteral expr, got {:?}", value),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_lex_unterminated_string() {
    let source = r#"template { let msg = "unterminated }"#;
    let tokens = lex(source, "test_lex_unterminated_string").expect("Lexing failed");

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
    let tokens = lex(source, "test_parse_integer_literal").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::Int(n) => assert_eq!(*n, 42),
            _ => panic!("Expected Int expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_float_literal() {
    let source = "template { let pi = 3.14 }";
    let tokens = lex(source, "test_parse_float_literal").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_return_statement() {
    // FIXME rerturn type wrong
    // Return requires a document element (like @text[...]), not a plain string
    let source = "template { return @text[done] }";
    let tokens = lex(source, "test_parse_return_statement").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::Return { doc_element } => match &doc_element.node {
            DocElemKind::Text { content, .. } => match &content.node {
                ExprKind::StringLiteral(s) => assert_eq!(s, "done"),
                _ => panic!("Expected StringLiteral content"),
            },
            _ => panic!("Expected Text DocElem in return"),
        },
        _ => panic!("Expected Return statement"),
    }
}

#[test]
fn test_parse_function_declaration() {
    // Elem declaration test (element keyword replaced func)
    let source = "template { element greet(name: string) { return @text[Hello] } }";
    let tokens = lex(source, "test_parse_function_declaration").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 1);

    match &template.statements[0].node {
        StmtKind::FuncDecl {
            name,
            args,
            body,
            return_type,
        } => {
            assert_eq!(name, "greet");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].ty, "string");
            assert!(body.len() > 0);
            assert_eq!(return_type, None); // FIXME return type wrong
        }
        _ => panic!("Expected FuncDecl statement"),
    }
}

#[test]
fn test_parse_function_call_no_args() {
    let source = "document { greet() }";
    let tokens = lex(source, "test_parse_function_call_no_args").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let doc = ast.document.unwrap();

    match &doc.elements[0].node {
        DocElemKind::Call { name, args, .. } => {
            assert_eq!(name, "greet");
            assert_eq!(args.len(), 0);
        }
        _ => panic!("Expected Call DocElem"),
    }
}

#[test]
fn test_parse_function_call_with_args() {
    let source = "document { print(\"hello\", \"world\") }";
    let tokens = lex(source, "test_parse_function_call_with_args").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let doc = ast.document.unwrap();

    match &doc.elements[0].node {
        DocElemKind::Call { name, args, .. } => {
            assert_eq!(name, "print");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Call DocElem"),
    }
}

#[test]
fn test_parse_default_set() {
    let source = "template { width = 100 }";
    let tokens = lex(source, "test_parse_default_set").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::DefaultSet { key, value } => {
            assert_eq!(key, "width");
            match &value.node {
                ExprKind::Int(n) => assert_eq!(*n, 100),
                _ => panic!("Expected Int expr"),
            }
        }
        _ => panic!("Expected DefaultSet statement"),
    }
}

#[test]
fn test_parse_multiple_statements() {
    let source = "template { let x = 1 let y = 2 let z = 3 }";
    let tokens = lex(source, "test_parse_multiple_statements").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 3);
}

#[test]
fn test_parse_mixed_statements() {
    let source = "template { let x = 10 const MAX = 100 width = 50 }";
    let tokens = lex(source, "test_parse_mixed_statements").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();
    assert_eq!(template.statements.len(), 3);

    match &template.statements[0].node {
        StmtKind::VarAssign { .. } => {}
        _ => panic!("Expected VarAssign"),
    }
    match &template.statements[1].node {
        StmtKind::ConstAssign { .. } => {}
        _ => panic!("Expected ConstAssign"),
    }
    match &template.statements[2].node {
        StmtKind::DefaultSet { .. } => {}
        _ => panic!("Expected DefaultSet"),
    }
}

#[test]
fn test_parse_dollar_sign_interpolation() {
    let source = "template { let msg = $ x $ }";
    let tokens = lex(source, "test_parse_dollar_sign_interpolation").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::Identifier(id) => {
                assert_eq!(id, "x");
            }
            _ => panic!("Expected Identifier expr"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_nested_template_and_document() {
    let source = "template { element render() { return @text[html] } } document { greet() }";
    let tokens = lex(source, "test_parse_nested_template_and_document").expect("Lexing failed");
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
    // Use ${...} syntax for interpolation
    let source = r#"template { let msg = "Hello, ${name}!" }"#;
    let tokens = lex(source, "test_parse_string_interpolation_simple").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::InterpolatedString { parts } => {
                assert_eq!(parts.len(), 3);
                // First part: "Hello, " (StringLiteral)
                match &parts[0] {
                    ExprKind::StringLiteral(text) => assert_eq!(text, "Hello, "),
                    _ => panic!("Expected StringLiteral part, got {:?}", parts[0]),
                }
                // Second part: name (Identifier)
                match &parts[1] {
                    ExprKind::Identifier(id) => assert_eq!(id, "name"),
                    _ => panic!("Expected Identifier expr, got {:?}", parts[1]),
                }
                // Third part: "!" (StringLiteral)
                match &parts[2] {
                    ExprKind::StringLiteral(text) => assert_eq!(text, "!"),
                    _ => panic!("Expected StringLiteral part, got {:?}", parts[2]),
                }
            }
            _ => panic!("Expected InterpolatedString, got {:?}", value),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_interpolation_multiple() {
    // Use ${...} syntax for multiple interpolations
    let source = r#"template { let msg = "${greeting}, ${name}!" }"#;
    let tokens = lex(source, "test_parse_string_interpolation_multiple").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::InterpolatedString { parts } => {
                assert_eq!(parts.len(), 4);
                // ${greeting}
                match &parts[0] {
                    ExprKind::Identifier(id) => assert_eq!(id, "greeting"),
                    _ => panic!("Expected Identifier, got {:?}", parts[0]),
                }
                // ", "
                match &parts[1] {
                    ExprKind::StringLiteral(text) => assert_eq!(text, ", "),
                    _ => panic!("Expected StringLiteral part, got {:?}", parts[1]),
                }
                // ${name}
                match &parts[2] {
                    ExprKind::Identifier(id) => assert_eq!(id, "name"),
                    _ => panic!("Expected Identifier, got {:?}", parts[2]),
                }
                // "!"
                match &parts[3] {
                    ExprKind::StringLiteral(text) => assert_eq!(text, "!"),
                    _ => panic!("Expected StringLiteral part, got {:?}", parts[3]),
                }
            }
            _ => panic!("Expected InterpolatedString"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_interpolation_with_number() {
    let source = r#"template { let msg = "Count: ${count}" }"#;
    let tokens = lex(source, "test_parse_string_interpolation_with_number").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::InterpolatedString { parts } => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    ExprKind::StringLiteral(text) => assert_eq!(text, "Count: "),
                    _ => panic!("Expected StringLiteral part"),
                }
                match &parts[1] {
                    ExprKind::Identifier(id) => assert_eq!(id, "count"),
                    _ => panic!("Expected Identifier expr, got {:?}", parts[1]),
                }
            }
            _ => panic!("Expected InterpolatedString"),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_without_interpolation() {
    // Plain strings without ${} should remain as StringLiteral
    // Curly braces alone are NOT interpolation
    let source = r#"template { let msg = "Hello, World!" }"#;
    let tokens = lex(source, "test_parse_string_without_interpolation").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::StringLiteral(s) => assert_eq!(s, "Hello, World!"),
            _ => panic!("Expected StringLiteral for plain string, got {:?}", value),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_string_with_literal_braces() {
    // Curly braces without $ should be treated as literal text
    let source = r#"template { let msg = "Use {brackets} freely" }"#;
    let tokens = lex(source, "test_parse_string_with_literal_braces").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::StringLiteral(s) => assert_eq!(s, "Use {brackets} freely"),
            _ => panic!(
                "Expected StringLiteral for string with literal braces, got {:?}",
                value
            ),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}

#[test]
fn test_parse_double_dollar_preserved() {
    // $$ is reserved for future display math support (like LaTeX $$...$$)
    // For now, $$ is treated as literal text in strings (not an escape sequence)
    let source = r#"template { let msg = "Price: $$100" }"#;
    let tokens = lex(source, "test_parse_double_dollar").expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let template = ast.template.unwrap();

    match &template.statements[0].node {
        StmtKind::VarAssign { value, .. } => match &value.node {
            ExprKind::StringLiteral(s) => assert_eq!(s, "Price: $$100"),
            _ => panic!("Expected StringLiteral, got {:?}", value),
        },
        _ => panic!("Expected VarAssign statement"),
    }
}
