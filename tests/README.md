# Test Suite Layout

The integration tests follow the compiler pipeline and are grouped into one
Cargo test target per stage:

- `lexer/`: token production, strings, text bodies, and lexer diagnostics.
- `parser/`: AST construction for roots, expressions, statements, elements,
  functions, string interpolation, and style syntax.
- `hir/`: AST-to-HIR lowering, document elements, calls, and control flow.
- `style/`: style resolution, selector matching, cascade, inheritance, and
  typed properties.
- `layout/`: document-flow layout, lists, separators, rows, and box geometry.

`support/mod.rs` contains shared integration-test helpers. Tests for the
private parser cursor live beside its implementation in
`src/parser/parser_cursor/tests.rs`, because cursor traversal is an internal
invariant rather than a public pipeline contract.

## Running Tests

Run all tests:

```sh
cargo test
```

Run one compiler stage:

```sh
cargo test --test lexer
cargo test --test parser
cargo test --test hir
cargo test --test style
cargo test --test layout
```

Run one module or case within a stage:

```sh
cargo test --test parser expressions::
cargo test --test layout rows::
```

## Deferred Coverage

Validation-pass cases remain deferred while `ValidationPass` is a stub.
Tests do not establish semantics for `while`, `for`, comparison operators, or
modulo parsing until those language features are implemented.
