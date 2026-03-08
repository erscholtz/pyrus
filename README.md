# pyrus

A domain-specific language (DSL) for creating styled documents. Pyrus aims to be an alternative to LaTeX and Typst, giving you fine-grained control over document styling while compiling to PDF.

## Quick Start

### Building

```bash
cargo build --release
```

### Running

```bash
# Run with the test input file
cargo run -- temp.ink

# Or run the compiled binary directly
./target/release/pyrus temp.ink
```

### Testing

```bash
cargo test
```

## Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| printpdf | 0.9.1 | PDF generation backend |
| taffy | 0.9.2 | CSS-style layout engine |

## Project Status

The compiler pipeline is currently implemented through HLIR (High-Level IR):

- [x] Lexer — Tokenizes source code
- [x] Parser — Builds AST
- [x] HLIR — First intermediate representation
- [ ] Layout Engine — Taffy integration (in progress)
- [ ] Backend — PDF rendering (basic implementation)

## Future Work: MLIR Backend

One of the major goals for pyrus is migrating the backend to **MLIR** (Multi-Level Intermediate Representation). This will enable:

- **Better optimization passes** — Leverage LLVM's optimization infrastructure
- **Multiple output targets** — PDF, WASM, and potentially others from the same IR
- **Performance** — Lower-level control over the compilation pipeline
- **Extensibility** — Easier to add new backends and transformations

The MLIR migration is planned after the current layout engine and basic PDF backend are stabilized.

## Example

See `temp.ink` for a sample document:

```bash
cargo run -- temp.ink
```

## License

See [LICENSE](LICENSE) for details.
