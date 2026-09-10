# Pyrus Language Specification

## Purpose

Pyrus is a tiny, opinionated resume compiler. It is not intended to compete
with Typst, LaTeX, HTML/CSS, or general-purpose document systems.

> Make one highly polished resume easy to write, fast to compile, and keep the
> entire language small enough to understand.

If a feature is not required by the real resume, it probably does not belong in
Pyrus.

---

## 1. Language boundaries

Pyrus source describes document structure and content. It is not a
general-purpose programming language.

The language has no:

- Control flow (`if`, `while`, or `for`)
- Variables, constants, or mutable state
- User-defined functions
- Arbitrary expressions or runtime computation
- String interpolation
- User-visible type system
- Imports, includes, or modules
- Inheritance or mixins

### One file, one document

Each `.pyr` file produces exactly one PDF.

Pyrus does not support:

- Multi-file projects
- Import or include paths
- Module resolution
- Multiple document outputs

A resume should remain small enough to fit comfortably in one source file.

---

## 2. Document configuration

`document` is an optional, built-in configuration block:

```pyrus
document {
    type: A4
    margin: 4
}
```

If omitted, the compiler uses predefined defaults. A4 is the default page
format.

The `document` block:

- Uses only compiler-defined keys
- Does not participate in document flow
- Is not a user-defined element
- Does not accept arbitrary properties, variables, or expressions
- May appear at most once

---

## 3. Element declarations

An `elem` declaration defines the shape of a document element:

```pyrus
elem entry {
    title,
    company,
    role,
    date,
    content,
}
```

Conceptually, an element definition resembles a Rust struct whose ordinary
fields contain text. It declares:

- The valid field names
- Whether trailing `content` is accepted

It does not contain behavior, rendering code, control flow, defaults, or layout.

### Ordinary fields

All ordinary fields contain text. Pyrus does not distinguish among strings,
integers, booleans, dates, enums, or user-defined types.

Valid:

```pyrus
elem entry {
    company,
    role,
    date,
}
```

Unsupported:

```pyrus
elem entry {
    company: String,
    year: Int,
}
```

### Strict field rules

Every declared field must be supplied exactly once.

```pyrus
@entry {
    company: ACME
    role: Software Engineer
    date: 2025–2026
}
```

The compiler rejects:

- Missing fields
- Unknown fields
- Duplicate fields
- Optional fields
- Default values
- Aliases
- Positional arguments
- Null values

> If you declare it, use it.

### Field order

Field order in an `elem` declaration has no rendering meaning. The declaration
defines a set of valid fields; its canonical `layout` determines rendering
order.

Instances may provide named fields in any order because the compiler collects
and validates them before rendering.

### Zero-field elements

Zero-field elements need no special prohibition:

```pyrus
elem marker {
}
```

An unused zero-field definition can be discarded like any other unused
declaration. An instantiated zero-field element remains significant because
its layout may still produce a visual result.

Do not build special features around zero-field elements without a demonstrated
resume requirement.

---

## 4. Element instances

Instantiate an element with `@name`:

```pyrus
@entry {
    title: Software Engineer
    company: ACME
    role: Systems Engineer
    date: 2025–2026

    - Built a compiler in Rust
    - Reduced build times
}
```

An instance supplies data only. It cannot:

- Override layout or styling
- Declare variables
- Execute code
- Define nested layouts

### Flat document flow

Top-level elements form a flat sequence:

```pyrus
@separator {
    title: Experience
}

@entry {
    // ...
}

@entry {
    // ...
}
```

User-defined elements cannot arbitrarily contain other document elements:

```pyrus
// Unsupported
@section {
    @entry {
        @something {
            // ...
        }
    }
}
```

Inline primitives, such as links, may still appear inside text or content.

> The source should read like a document, not a UI tree.

---

## 5. Trailing content

`content` is a special declared field:

```pyrus
elem entry {
    title,
    company,
    content,
}
```

When declared, everything after the named fields becomes one trailing `Content`
value:

```pyrus
@entry {
    title: Software Engineer
    company: ACME

    First paragraph.

    Second paragraph.

    - Bullet one
    - Bullet two
}
```

Although the body may contain multiple paragraphs or bullets, it remains one
`Content` value internally.

Rules:

- `content` must be explicitly declared.
- Elements without `content` reject trailing body text.
- Declared `content` is required and must be non-empty.
- The element's layout must consume `content` exactly once.

### Supported block content

Content initially supports only:

```text
Plain text

Multiple paragraphs

- Flat bullet
- Flat bullet
- Flat bullet
```

It does not support:

- Nested or numbered lists
- Task lists
- Tables
- Fenced code blocks
- Arbitrary block elements

---

## 6. Inline text

The inline formatting vocabulary is deliberately small:

```pyrus
**bold**
*italic*
`nerd-font text`
```

Normal text uses the embedded monospaced serif font. Backticks switch to the
embedded Nerd Font.

Initially unsupported:

- Nested formatting
- Underline or strikethrough
- Highlighting
- Arbitrary font selection or colors
- Spans or inline CSS

Only add nested formatting if the real resume demonstrates a need.

### Links

Links are a built-in inline primitive and may have a compiler-defined visual
identity. Normal text remains black, while links may use color.

A future PDF interaction could display a hovered link as:

```text
[ github.com/example ]
```

with a dark background and light text. This behavior is an optional enhancement;
the static PDF must remain readable without it.

### Comments

Only single-line comments are supported:

```pyrus
// Experience section
```

Pyrus has no block, nested, or documentation comments.

### Escapes

Only syntax-required escape sequences are supported initially:

```text
\`
\\
\{
\}
```

Do not add a generalized escape system without a demonstrated requirement.

---

## 7. Canonical layouts

Every instantiated element requires exactly one canonical layout:

```pyrus
elem entry {
    title,
    company,
    date,
}

layout entry {
    title < | > date
    company
}
```

Element instances cannot override their layout.

Pyrus has no:

- Classes, IDs, or selectors
- Per-instance style overrides
- Alternate layouts
- Layout inheritance or composition
- CSS cascade

Duplicate layout definitions are compile errors.

### Exact field consumption

A layout must consume every declared field exactly once.

Given:

```pyrus
elem entry {
    title,
    company,
    date,
}
```

This layout is invalid because `company` is unused:

```pyrus
layout entry {
    title < | > date
}
```

This layout is invalid because `title` is consumed twice:

```pyrus
layout entry {
    title
    company
    title < | > date
}
```

Unknown layout fields are also compile errors.

```text
declared fields == layout fields
```

Each field must appear exactly once on both sides of that invariant.

---

## 8. Declaration and usage rules

### Declaration order

A declaration must appear before it is used. The preferred source order is:

```pyrus
document {
    // ...
}

elem entry {
    // ...
}

layout entry {
    // ...
}

@entry {
    // ...
}
```

Pyrus has no forward references. This keeps compilation simple and friendly to
a one-pass implementation.

### Duplicate definitions

Every language object has one unambiguous definition. The compiler rejects:

```text
duplicate elem entry
duplicate layout entry
duplicate document block
duplicate field inside elem
duplicate field inside instance
```

### Unused definitions

Unused declarations are legal but produce warnings:

```text
warning: element `project` is defined but never used
```

The compiler may discard unused declarations before rendering. They should not
survive into render-oriented HIR.

---

## 9. Explicitly excluded features

### Tables

Pyrus has no Markdown tables, table primitive, or grid/table layout subsystem.
Reconsider table-like behavior only if the actual resume requires it.

### Multiline code blocks

Only inline Nerd Font syntax remains:

```pyrus
`Rust`
```

There are no fenced code blocks, language identifiers, or syntax highlighting.

### Images

Pyrus has no general-purpose image support. Images would introduce asset paths,
decoding, sizing, placement, PDF embedding, and ATS concerns.

Consider any resume-specific visual artifact as a compiler-owned feature before
introducing a generic image primitive.

---

## 10. Backend-only concerns

Some features may improve output without expanding the source language.

### Build provenance

A future compiler-owned flourish could render provenance such as:

```text
Built from resume.pyr · Pyrus 0.4.2 · 9f83a1c
```

Potential goals:

- Visible and visually distinctive to humans
- Optionally excluded from the primary semantic text representation
- Does not interfere with ATS parsing

Do not implement provenance until the core resume works.

### ATS compatibility

Do not redesign the language around assumptions about applicant tracking
systems. Once the compiler works, test actual PDF output against parsers.

Evaluate:

- Text extraction and semantic reading order
- Hyperlink extraction
- Headers, footers, and metadata
- Font embedding
- Real PDF text versus text rendered as glyphs
- Multi-column extraction
- Build/provenance marks
- Accessibility and tagged PDF structure

Treat ATS compatibility as a layout/backend concern wherever possible, not as a
source-language feature.

---

## 11. Compiler architecture

Keep the Rust compiler pipeline aggressively small:

```text
source
  ↓
lexer
  ↓
parser
  ↓
AST
  ↓
validation
  ↓
HIR
  ↓
layout
  ↓
PDF
```

Definitions such as `elem` and `layout` are compile-time metadata. Only
instantiated elements need to survive into render-oriented HIR.

A conceptual Rust model:

```rust
struct ElementDef {
    name: Symbol,
    fields: Vec<FieldDef>,
    has_content: bool,
}

struct ElementInstance {
    element: ElementId,
    fields: Vec<(FieldId, String)>,
    content: Option<Content>,
}
```

Keep these structures boring. Introduce abstractions only when duplication or
correctness demands them.

---

## 12. Diagnostics

Use the existing diagnostics system aggressively.

### Errors

Compile errors should cover:

- Unknown element or field
- Missing or duplicate field
- Duplicate element or layout
- Missing layout
- Unused or duplicate layout field
- Trailing content without declared `content`
- Missing required content
- Forward references
- Unsupported syntax

### Warnings

Warnings should identify legal but probably accidental behavior:

- Unused element
- Unused layout

Possible future warnings:

- Raw URL should use the link primitive
- Text may parse poorly in ATS software
- Unsupported glyph

Prefer actionable messages, for example:

```text
error: layout `entry` does not consume field `company`
error: element `entry` supplies field `date` twice
warning: element `project` is defined but never used
```

---

## 13. Implementation roadmap

### Remove out-of-scope language features

- [ ] Remove control-flow tokens and parser paths
- [ ] Remove variables and constants
- [ ] Remove function semantics
- [ ] Remove expression evaluation
- [ ] Remove interpolation
- [ ] Remove type handling
- [ ] Remove imports and modules
- [ ] Remove CSS and selectors
- [ ] Remove tables
- [ ] Remove fenced code blocks

### Finalize document syntax

- [ ] Finalize `document` grammar
- [ ] Finalize `elem` grammar
- [ ] Finalize `@elem` instance grammar
- [ ] Add strict field validation
- [ ] Add explicit `content` declaration
- [ ] Parse trailing `Content`
- [ ] Add paragraphs
- [ ] Add flat bullet lists

### Finalize inline syntax

- [ ] Add bold
- [ ] Add italic
- [ ] Keep backtick/Nerd Font mode
- [ ] Keep the link primitive
- [ ] Keep `//` comments
- [ ] Keep minimal escapes

### Finalize layout semantics

- [ ] Require one layout per instantiated element
- [ ] Require every field exactly once in its layout
- [ ] Reject duplicate layouts
- [ ] Warn about unused declarations

### Lower and render

- [ ] Lower only instantiated elements into render-oriented HIR
- [ ] Dead-strip unused definitions
- [ ] Port the actual resume

---

## 14. Hard feature gate

Once the roadmap works, stop adding language features and port the real resume.

Classify every missing capability:

| Category | Meaning | Action |
| --- | --- | --- |
| **Bug** | Existing syntax should support this | Fix it |
| **Necessary feature** | The resume cannot express this | Consider syntax |
| **Backend feature** | Syntax works; output does not | Improve backend |
| **Polish** | It works but could look better | Tune or defer |
| **Hypothetical** | An imaginary future document might need this | Reject it |

Only bugs and demonstrated language deficiencies justify expanding the syntax.
Backend improvements may proceed without growing the language.

---

## 15. Target end state

A complete Pyrus source file should remain roughly this complex:

```pyrus
document {
    type: A4
    margin: 4
}

elem header {
    name,
    contact,
}

elem separator {
    title,
}

elem entry {
    company,
    role,
    date,
    content,
}

layout header {
    name < | > contact

    name: xl
    contact: sm
}

layout separator {
    title

    title: lg
}

layout entry {
    company < | > date
    role
    content

    company: md
    date: sm
    role: sm
    content: sm
}

@header {
    name: Erik Example
    contact: `github` · `email`
}

@separator {
    title: Experience
}

@entry {
    company: ACME
    role: Software Engineer
    date: 2025–2026

    - Built **Pyrus**, a tiny resume compiler in `Rust`
    - Worked on low-level document layout and PDF generation
}
```

This is the language's approximate complexity ceiling. Its entire surface area
should remain small enough to understand at a glance.
