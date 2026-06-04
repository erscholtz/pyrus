use pyrus::diagnostic::{CompilerDiagnostic, SyntaxError};
use pyrus::lexer;

#[test]
fn records_plain_and_interpolated_string_flags() {
    let tokens = lexer::lex_all(r#""plain" "Hello, ${name}!""#, "strings").unwrap();
    assert!(!tokens.string_table[0].has_interpolation);
    assert!(tokens.string_table[1].has_interpolation);
}

#[test]
fn records_unterminated_string_diagnostic() {
    let errors = lexer::lex_all(r#"template { let msg = "unterminated }"#, "test.ink").unwrap_err();
    assert!(matches!(
        errors.first(),
        Some(CompilerDiagnostic::Syntax(SyntaxError::UnterminatedDelimiter {
            delimiter,
            ..
        })) if delimiter == "\""
    ));
}
