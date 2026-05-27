use pyrus::diagnostic::{CompilerDiagnostic, SyntaxError};
use pyrus::lexer;

#[test]
fn records_plain_and_interpolated_string_flags() {
    let tokens = lexer::lex(r#""plain" "Hello, ${name}!""#, "strings").unwrap();
    assert!(!tokens.string_table[0].has_interpolation);
    assert!(tokens.string_table[1].has_interpolation);
}

#[test]
fn records_unterminated_string_diagnostic() {
    let tokens = lexer::lex(r#"template { let msg = "unterminated }"#, "test.ink").unwrap();
    assert!(matches!(
        tokens.errors.first(),
        Some(CompilerDiagnostic::Syntax(SyntaxError::UnterminatedDelimiter {
            delimiter,
            ..
        })) if delimiter == "\""
    ));
}
