# pyrus

A domain-specific language (DSL) for creating styled documents. Pyrus aims to be an alternative to LaTeX and Typst, giving you fine-grained control over document styling using CSS while compiling to PDF. It is also possible to seperate out content into component-like structures that can be defined by the user.

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
| phf | 0.11 | for better hashes in maps |
| printpdf | 0.9.1 | PDF generation backend |
| taffy | 0.9.2 | CSS-style layout engine |

## Project Status

The compiler pipeline is currently implemented through HIR (High-Level IR):

It is posible to fully use the Pyrus compiler in its current form and create documents.

- [x] Lexer 
- [x] Parser 
- [x] HIR 
- [x] CSS Layout Engine and Attribute tree
- [x] PDF Rendering Backend

## Future Work

- [ ] Turing complete scripting
- [ ] Default styling and settings
- [ ] Formula support
- [ ] `--watch` and incremental compilation
- [ ] LLVM's MLIR passes
- [ ] multiple file and project support
- [ ] embedding support as framework for other projects 

## Example

See `resume.ink` for a sample document:

```bash
cargo run -- temp.ink
```

## License

See [LICENSE](LICENSE) for details.
