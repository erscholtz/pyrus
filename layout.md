# Pyrus Layout Roadmap

## Guiding principles

- Freeze the language surface before building the layout engine.
- Keep layout in logical cells and convert to physical coordinates only in the
  PDF backend.
- Build the smallest useful document pipeline first.
- Reject unsupported behavior instead of silently inventing semantics.
- Use the real resume as the gate for all future language features.

The goal is not a large language. The goal is a small, deterministic, fast tool
with excellent output and compiler errors.

---

## 1. Freeze the language surface

Before touching layout, document the current grammar as the contract:

```pyrus
elem entry {
    title,
    company,
    date,
    role,
    content,
}

layout entry {
    title   < | > date
    company < | > role
    content

    title: lg
    date: sm
}

@entry {
    title: Software Engineer
    company: ACME
    date: 2025–2026
    role: Systems

    Built cool stuff.
}
```

Initially support only:

- `elem` declarations with named fields
- One canonical `layout` per element
- `@elem` element construction
- Anonymous/freeform bodies mapped to `content`
- Placement and typography in layouts

Explicitly defer:

- Layout overrides and composition
- Control flow and arbitrary functions
- CSS selectors, classes, and IDs

Do not redesign the language while building the layout engine unless
implementation reveals a fundamental problem.

## 2. Define the coordinate system

Everything in the layout engine operates in cells:

| Size | Cell dimensions |
| --- | --- |
| `sm` | 1 × 1 |
| `md` | 2 × 2 |
| `lg` | 4 × 4 |
| `xl` | 8 × 8 |

Use A4 as the default page size.

```rust
struct CellMetrics {
    width_px: f32,
    height_px: f32,
}

struct Page {
    width_cells: usize,
    height_cells: usize,
    margin_cells: usize,
}
```

Tune `width_px` and `height_px` during development until the embedded font looks
right. These physical measurements must not be exposed in Pyrus source.

**Invariant:** layout code thinks in cells; PDF code thinks in physical
coordinates. Convert between them only at the rendering boundary.

## 3. Establish basic font rendering

Before adding interesting layout behavior:

1. Embed the normal monospaced serif font.
2. Render UTF-8 text with it.
3. Calculate glyph advance.
4. Render one line at an `(x, y)` cell coordinate.
5. Verify that every glyph advances predictably.

Keep the initial renderer API deliberately small:

```rust
fn draw_text(
    text: &str,
    x_cell: usize,
    y_cell: usize,
    size: TextSize,
) {
    todo!()
}
```

Defer the Nerd Font and font fallback.

## 4. Build the text wrapper independently

Do not bury wrapping behavior inside the layout engine.

```rust
fn wrap(text: &str, available_cells: usize, size: TextSize) -> WrappedText {
    todo!()
}

struct WrappedLine {
    text: String,
    width_cells: usize,
}

struct WrappedText {
    lines: Vec<WrappedLine>,
    height_cells: usize,
}
```

For example, if `md` consumes two cells per glyph and 60 cells are available,
approximately 30 monospaced glyphs fit on a line.

Initial wrapping rules:

- Wrap at word boundaries.
- Never clip text.
- Mechanically split oversized tokens.
- Do not hyphenate.
- Do not add ellipses.
- Do not special-case URLs yet.

Give this component extensive unit coverage. It should become a boring,
trustworthy foundation.

## 5. Add fixed text sizes

Start with `sm`, then add the remaining fixed sizes:

```rust
enum TextSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl TextSize {
    fn scale(self) -> usize {
        match self {
            Self::Sm => 1,
            Self::Md => 2,
            Self::Lg => 4,
            Self::Xl => 8,
        }
    }
}
```

Wrapping and vertical advancement must both respect the selected scale. Do not
support arbitrary sizes.

## 6. Parse layouts into a minimal IR

Compile the layout language into only two row types:

```rust
enum LayoutRow {
    Full(FullRow),
    Split(SplitRow),
}

struct FullRow {
    field: FieldId,
}

struct SplitRow {
    left: FieldId,
    right: FieldId,
}
```

The only initial forms are:

```pyrus
content
```

and:

```pyrus
title < | > date
```

If the parser begins to require recursive layout nodes, stop and reconsider—the
project should not become a general-purpose UI framework.

## 7. Implement full-width flow

Ignore `< | >` initially. Support only full-width rows:

```pyrus
layout entry {
    title
    company
    content
}
```

Conceptually, each line becomes:

```rust
struct FullRow {
    field: FieldId,
}
```

Rendering should:

1. Calculate available width from page width minus margins.
2. Wrap words to that width.
3. Render and advance the vertical cursor.
4. Insert the universal element gap.
5. Continue with the next element.

This milestone should produce an ugly but complete resume.

## 8. Add simple split rows

Once full-width rows are reliable, implement the happy path for:

```pyrus
left < | > right
```

Expected output:

```text
Software Engineer                         2025–2026
```

Algorithm:

1. Measure both sides.
2. Determine the available row width.
3. Reserve a minimum separation.
4. Place the left value at the left boundary.
5. Place the right value against the right boundary.
6. Finish if both values fit.

Do not solve complicated wrapping at this stage.

## 9. Add split-row wrapping

Next, support wrapped split rows:

```text
Really long left-hand content
that has to wrap
       equally obnoxious right-hand content
       that also has to wrap
       down here
```

The exact indentation algorithm can evolve. Begin with an internal constant:

```rust
const SPLIT_FALLBACK_INDENT: usize = 4;
```

Do not expose this value in Pyrus source.

## 10. Validate layouts before rendering

Reject incompatible layouts early and produce actionable diagnostics:

```text
error: layout `header` references unknown field `dates`
error: element `entry` has no layout
error: element `entry` defines field `company` twice
error: unsupported split-row configuration
```

Prefer an explicit error over silently inventing layout behavior.

## 11. Add universal vertical flow

Top-level elements stack with one compiler-defined gap:

```text
element
↓
default gap
↓
element
↓
default gap
↓
element
```

```rust
const ELEMENT_GAP: usize = 1;
```

Do not add source-level `margin-bottom`, `padding`, or `gap-after` controls yet.
Tune the compiler constant until the resume looks right.

## 12. Add pagination

Only add pagination after one-page layout works. Keep it simple:

```rust
struct PageCursor {
    page: usize,
    x: usize,
    y: usize,
}
```

Before placing structured content, calculate its required height. If it does not
fit:

1. Start a new page.
2. Reset `y` to the top margin.
3. Render the element.

Freeform content may continue flowing across pages.

Defer:

- Manual `@pagebreak`
- Widow/orphan control
- Keep-with-next behavior
- Page regions
- Headers and footers

## 13. Add the second font

Once layout is stable, support both:

- Normal text
- Nerd Font text

Both fonts must follow the same logical cell model. Normalize differing native
metrics in the renderer; the layout engine should not know which font is
active.

## 14. Port the real resume

At this point, stop designing Pyrus and port the actual resume.

Classify every problem encountered as one of:

| Category | Meaning | Action |
| --- | --- | --- |
| **Bug** | Existing syntax should work | Fix it |
| **Layout deficiency** | The resume cannot express the layout | Consider it |
| **Preference** | Expressible but visually imperfect | Tune or defer |
| **Hypothetical** | Another document might need it | Delete or defer |

Only bugs and demonstrated layout deficiencies justify language changes.
Porting the resume is the hard gate before adding another language feature.

## 15. Optimize for the right kind of ridiculousness

After the resume works, focus on:

- A tiny binary
- A tiny lexer and parser
- A tiny layout IR
- Embedded fonts
- Deterministic output
- Extremely fast compilation
- No dependencies where practical
- Excellent compiler errors
- Exceptionally clean resume source

The target flex is:

```console
$ pyrus resume.pyr
compiled resume.pdf in 2.1ms
```

Not:

> Pyrus has fourteen control-flow constructs and a CSS cascade.

---

## Implementation checklist

### Language foundation

- [ ] Freeze and document the syntax
- [ ] Parse `elem` declarations
- [ ] Parse `@elem` instances
- [ ] Parse `layout` declarations

### Rendering foundation

- [ ] Add an A4 page abstraction
- [ ] Define the cell coordinate system
- [ ] Embed the serif monospaced font
- [ ] Render UTF-8 text at cell coordinates
- [ ] Implement word wrapping and its unit tests
- [ ] Add `sm`, `md`, `lg`, and `xl`

### Layout

- [ ] Compile layouts into the minimal row IR
- [ ] Render full-width rows
- [ ] Add the universal element gap
- [ ] Render simple `< | >` rows
- [ ] Render wrapped `< | >` rows
- [ ] Validate layouts and report errors

### Pagination and fonts

- [ ] Add automatic multi-page flow
- [ ] Flow freeform content across pages
- [ ] Embed the Nerd Font

### Product validation

- [ ] Port the actual resume
- [ ] Add nothing else until the resume exposes a real deficiency
