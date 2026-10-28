# Integration tests

`tests/` contains only smoke and acceptance coverage for complete documents.
`smoke.rs` runs the public parser → HIR → layout → PDF pipeline for:

- an empty document;
- a styled document with reusable elements, split rows, Unicode, inline
  formatting, bullets, and a PDF link;
- narrow split columns that require text wrapping.

The styled document fixture lives in `input/styled-document.pyr`. The wrapping
case was moved from the layout module and updated for the current document
configuration and explicit layout sizes.

Stage-specific and internal invariants belong in `#[cfg(test)]` modules beside
their implementations in `src/`. The obsolete `.ink` syntax suites have been
removed rather than ported to the current language.

Run everything with `cargo test`, integration smoke tests with
`cargo test --test smoke`, or module tests with `cargo test --lib`.

Tests use the repository's bundled fonts. PDFs are written under
`target/smoke-tests/` with a separate filename per test and process, then removed
after checking the PDF header and trailer. These are smoke checks, not visual
PDF comparisons.
