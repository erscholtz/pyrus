//! Tests for HLIR lowering (AST → HLIR) and validation
//!
//! These tests define the expected behavior of the lowering pass and
//! the validation pass that should catch errors.

use pyrus::hir::{FuncId, HIRModule, Id, Op, Type};
use pyrus::hir::{lower, resolve_styles};
use pyrus::lexer::lex;
use pyrus::parser::parse;

// ============================================================================
// Basic Lowering Tests
// ============================================================================

#[test]
fn test_lower_empty_document() {
    let source = r#"
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // Should have the implicit __document function
    assert_eq!(
        hlir.functions.len(),
        1,
        "Should have one function (__document)"
    );
    assert!(hlir.functions.values().any(|f| f.name == "__document"));
    assert!(hlir.globals.is_empty());
    assert!(hlir.elements.is_empty());
}

#[test]
fn test_lower_global_const() {
    let source = r#"
template {
    const PI = 3.14
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    assert_eq!(hlir.globals.len(), 1, "Should have one global");
    assert!(hlir.globals.values().any(|g| g.name == "PI"));
}

#[test]
fn test_lower_global_var() {
    let source = r#"
template {
    var counter = 0
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    assert_eq!(hlir.globals.len(), 1);
    assert!(hlir.globals.values().any(|g| g.name == "counter"));
}

#[test]
fn test_lower_multiple_globals() {
    let source = r#"
template {
    const TITLE = "My Doc"
    const AUTHOR = "Me"
    var page_num = 1
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    assert_eq!(hlir.globals.len(), 3);
}

#[test]
fn test_lower_simple_function() {
    let source = r#"
template {
    func greeting() -> DocElement {
        return text { "Hello" }
    }
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // Should have __document + greeting
    assert_eq!(hlir.functions.len(), 2);
    assert!(hlir.functions.values().any(|f| f.name == "greeting"));

    let greeting = hlir
        .functions
        .values()
        .find(|f| f.name == "greeting")
        .unwrap();
    assert_eq!(greeting.args.len(), 0);
    assert_eq!(greeting.return_type, Some(Type::DocElement));
}

#[test]
fn test_lower_function_with_args() {
    let source = r#"
template {
    func section_with_title(title: String) -> DocElement {
        return section {
            text { title }
        }
    }
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    let func = hlir
        .functions
        .values()
        .find(|f| f.name == "section_with_title")
        .unwrap();
    assert_eq!(func.args.len(), 1);
    assert_eq!(func.args[0], Type::String);
}

#[test]
fn test_lower_function_with_multiple_args() {
    let source = r#"
template {
    func formatted_number(value: Int, prefix: String) -> DocElement {
        return text { prefix }
    }
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    let func = hlir
        .functions
        .values()
        .find(|f| f.name == "formatted_number")
        .unwrap();
    assert_eq!(func.args.len(), 2);
    assert_eq!(func.args[0], Type::Int);
    assert_eq!(func.args[1], Type::String);
}

// ============================================================================
// Document Element Lowering Tests
// ============================================================================

#[test]
fn test_lower_text_element() {
    let source = r#"
document {
    text { "Hello World" }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    assert_eq!(hlir.elements.len(), 1);
    assert_eq!(hlir.element_metadata.len(), 1);
    assert_eq!(hlir.element_metadata[0].element_type, "text");
}

#[test]
fn test_lower_section_with_children() {
    let source = r#"
document {
    section {
        text { "Child 1" }
        text { "Child 2" }
    }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // Should have: 1 section + 2 text elements
    assert_eq!(hlir.elements.len(), 3);

    // Find section metadata
    let section_meta = hlir
        .element_metadata
        .iter()
        .find(|m| m.element_type == "section")
        .expect("Should have section metadata");

    // Children should point to section as parent
    let children: Vec<_> = hlir
        .element_metadata
        .iter()
        .filter(|m| m.parent == Some(0)) // section is index 0
        .collect();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_lower_element_with_id_and_class() {
    let source = r#"
document {
    text (id="header", class="large bold") { "Title" }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    let meta = &hlir.element_metadata[0];
    assert_eq!(meta.id, Some("header".to_string()));
    assert_eq!(meta.classes, vec!["large", "bold"]);
}

// ============================================================================
// CSS Style Integration Tests
// ============================================================================

#[test]
fn test_lower_preserves_css_rules() {
    let source = r#"
document {
    text { "Content" }
}
style {
    text {
        font-size: 14pt;
    }
    .highlight {
        color: red;
    }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    assert_eq!(hlir.css_rules.len(), 2);
}

// ============================================================================
// Function Call Lowering Tests
// ============================================================================

#[test]
fn test_lower_function_call_in_document() {
    let source = r#"
template {
    func header() -> DocElement {
        return text { "Header" }
    }
}
document {
    header()
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // Find __document function body
    let doc_func = hlir
        .functions
        .values()
        .find(|f| f.name == "__document")
        .expect("Should have __document function");

    // Should have a Call operation
    let has_call = doc_func
        .body
        .ops
        .iter()
        .any(|op| matches!(op, Op::Call { .. }));
    assert!(has_call, "Should generate Call op for function call");
}

#[test]
fn test_lower_function_call_with_args() {
    let source = r#"
template {
    func greet(name: String) -> DocElement {
        return text { name }
    }
}
document {
    greet("World")
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    let doc_func = hlir
        .functions
        .values()
        .find(|f| f.name == "__document")
        .unwrap();

    // Find the call operation and check it has args
    let call_op = doc_func.body.ops.iter().find_map(|op| match op {
        Op::Call { func, args, .. } => Some((func, args)),
        _ => None,
    });

    assert!(call_op.is_some(), "Should have Call op");
    let (_, args) = call_op.unwrap();
    assert_eq!(args.len(), 1, "Call should have 1 argument");
}

// ============================================================================
// Op Sequence Tests
// ============================================================================

#[test]
fn test_lower_generates_doc_element_emit_ops() {
    let source = r#"
document {
    text { "First" }
    text { "Second" }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    let doc_func = hlir
        .functions
        .values()
        .find(|f| f.name == "__document")
        .unwrap();

    let emit_ops: Vec<_> = doc_func
        .body
        .ops
        .iter()
        .filter(|op| matches!(op, Op::HlirElementEmit { .. }))
        .collect();

    assert_eq!(emit_ops.len(), 2, "Should have 2 DocElementEmit ops");
}

#[test]
fn test_lower_const_generates_const_op() {
    let source = r#"
template {
    const VALUE = 42
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // The global should have been created with the right value
    let global = hlir
        .globals
        .values()
        .find(|g| g.name == "VALUE")
        .expect("Should have VALUE global");

    // Check the init literal
    match &global.init {
        pyrus::hir::Literal::Int(42) => {}
        _ => panic!("Expected Int(42), got {:?}", global.init),
    }
}

// ============================================================================
// Validation Tests (These will fail until validation is implemented)
// ============================================================================

// TODO: Uncomment these tests once validation module is implemented
// The validation module should catch these errors at compile time

/*
#[test]
#[should_panic(expected = "Type mismatch")]
fn test_validation_catches_type_mismatch_in_assignment() {
    let source = r#"
template {
    var x: Int = "not an int"
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // This should fail validation - assigning String to Int variable
    pyrus::hlir::validate(&hlir).expect("Should catch type mismatch");
}

#[test]
#[should_panic(expected = "Function not found")]
fn test_validation_catches_undefined_function_call() {
    let source = r#"
document {
    nonexistent_func()
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch undefined function");
}

#[test]
#[should_panic(expected = "Wrong number of arguments")]
fn test_validation_catches_wrong_arg_count() {
    let source = r#"
template {
    func needs_two(a: Int, b: Int) -> DocElement {
        return text { "ok" }
    }
}
document {
    needs_two(1)  // Missing second argument
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch wrong argument count");
}

#[test]
#[should_panic(expected = "Type mismatch in argument")]
fn test_validation_catches_wrong_arg_type() {
    let source = r#"
template {
    func expects_int(x: Int) -> DocElement {
        return text { "ok" }
    }
}
document {
    expects_int("string instead of int")
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch wrong argument type");
}

#[test]
#[should_panic(expected = "Return type mismatch")]
fn test_validation_catches_return_type_mismatch() {
    let source = r#"
template {
    func returns_doc() -> DocElement {
        return 42  // Should return DocElement, not Int
    }
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch return type mismatch");
}

#[test]
#[should_panic(expected = "Missing return statement")]
fn test_validation_catches_missing_return() {
    let source = r#"
template {
    func should_return() -> DocElement {
        // No return statement
    }
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch missing return");
}

#[test]
#[should_panic(expected = "Duplicate symbol")]
fn test_validation_catches_duplicate_global() {
    let source = r#"
template {
    const X = 1
    const X = 2  // Duplicate!
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch duplicate symbol");
}

#[test]
#[should_panic(expected = "Duplicate function")]
fn test_validation_catches_duplicate_function() {
    let source = r#"
template {
    func foo() -> DocElement { return text { "a" } }
    func foo() -> DocElement { return text { "b" } }
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch duplicate function");
}

#[test]
#[should_panic(expected = "Binary operation type mismatch")]
fn test_validation_catches_binary_op_type_error() {
    let source = r#"
template {
    const RESULT = 1 + "string"  // Can't add Int + String
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    pyrus::hlir::validate(&hlir).expect("Should catch binary op type error");
}

#[test]
fn test_validation_passes_valid_code() {
    let source = r#"
template {
    const PI = 3.14
    func circle_area(radius: Float) -> Float {
        return PI * radius * radius
    }
}
document {
    text { "Hello" }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // This should succeed
    pyrus::hlir::validate(&hlir).expect("Valid code should pass validation");
}
*/

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_lower_preserves_element_order() {
    let source = r#"
document {
    text { "First" }
    section {
        text { "Nested" }
    }
    text { "Last" }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // Elements should be in document order
    assert_eq!(hlir.element_metadata[0].element_type, "text");
    assert_eq!(hlir.element_metadata[1].element_type, "section");
    assert_eq!(hlir.element_metadata[2].element_type, "text");
    assert_eq!(hlir.element_metadata[3].element_type, "text"); // Nested
}

#[test]
fn test_lower_empty_template_is_ok() {
    let source = r#"
template {
}
document {
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    assert!(hlir.globals.is_empty());
    assert_eq!(hlir.functions.len(), 1); // Just __document
}

#[test]
fn test_lower_nested_sections() {
    let source = r#"
document {
    section {
        section {
            text { "Deeply nested" }
        }
    }
}
"#;
    let tokens = lex(source).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower(&ast);

    // Should have: outer section (0), inner section (1), text (2)
    assert_eq!(hlir.elements.len(), 3);

    // Inner section's parent should be outer section
    assert_eq!(hlir.element_metadata[1].parent, Some(0));
    // Text's parent should be inner section
    assert_eq!(hlir.element_metadata[2].parent, Some(1));
}
