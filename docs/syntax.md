# Pyrus Syntax

Pyrus is a small document language built around a separation between structure,
layout, and content. An element declares which fields exist, a layout decides
where those fields appear, and an instance supplies their values.

The syntax is intentionally narrow. Pyrus is currently aimed at polished
resumes rather than general-purpose programming or web-style page design.

## Document configuration

An optional `document` block configures compiler-defined page settings:

```pyrus
document {
    type: A4
    margin: 4
}
```

If the block is omitted, Pyrus uses its default page format and margins. A
source file may contain at most one `document` block.

## Element declarations

An `elem` declaration names the fields accepted by an element:

```pyrus
elem entry {
    title,
    company,
    date,
    content,
}
```

Fields contain text and do not have user-visible types. Every declared field
must be provided exactly once when the element is used. Missing, unknown, and
duplicate fields are errors.

The special `content` field allows an instance to contain freeform body text.
It must be declared explicitly if the element accepts such a body.

## Layout declarations

A `layout` controls the order, alignment, and size of an element's fields:

```pyrus
layout entry {
    title   < | > date
    company
    content

    title: lg
    date: sm
}
```

A bare field occupies a full row. The `< | >` form creates a split row with
content aligned to its left and right edges. Each declared field must appear in
the layout exactly once.

The layout operators are:

| Symbol | Meaning |
| --- | --- |
| `<` | Align left |
| `>` | Align right |
| `|` | Split a row |
| `<<` | Override left alignment when wrapping |
| `>>` | Override right alignment when wrapping |

Each element has one canonical layout. Instances cannot select another layout
or override the layout locally.

## Element instances

An element is instantiated by prefixing its name with `@`:

```pyrus
@entry {
    title: Software Engineer
    company: ACME
    date: 2025–2026

    - Built a compiler in Rust
    - Improved document layout
}
```

Named fields may be supplied in any order. When `content` is declared, text
after the named fields becomes that field's value. Content currently consists
of plain paragraphs and flat bullet lists.

Top-level instances form a flat document flow. User-defined elements do not
arbitrarily nest other elements.

## Inline text

The current inline forms are deliberately small:

```pyrus
**bold**
*italic*
`nerd-font text`
```

Normal text uses the embedded monospaced serif font. Backticks switch to the
embedded Nerd Font. Links are a compiler-provided inline primitive rather than
a general styling mechanism.

Single-line comments begin with `//`. Only escapes required to write Pyrus
syntax are planned initially, including escaped backticks, backslashes, and
braces.

## Source order

Declarations appear before their uses. The conventional order is:

```text
document configuration
element declarations
layout declarations
element instances
```

Pyrus has no forward references. This keeps validation and compilation simple
and predictable.

## Language boundaries

The current syntax does not include variables, functions, control flow,
imports, selectors, classes, IDs, or per-instance styling. It also excludes
tables, images, nested lists, and fenced code blocks.

These limits keep the source readable as a document and prevent the layout
language from growing into a general-purpose UI system. New syntax should be
added only when a real document exposes a concrete limitation.
