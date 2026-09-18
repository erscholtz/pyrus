mod allocate; // allocate glyphs to rows
mod compose; // compose wrapped rows to ResolvedRow(s)
mod inscribe; // make glyphs
mod paginate; // add ResolvedRows to Pages
mod wrap; // wrap rows to fit in margins

use crate::{hir::hir_types::HIR, layout::inscribe::GlyphRow};

// pages laid out with coordinates and all that for super easy rendering
struct ResolvedDocument {
    pages: Vec<ResolvedPage>,
}

// spacing out what rows we have given the top down margins and remaing rows we
// got on a page
struct ResolvedPage {
    rows: ResolvedRow,
}

struct Coordinate {
    x: usize,
    y: usize,
}

struct Extent {
    width: usize,
    height: usize,
}

// given all the stuff about margins, sizing all that, here are the exact
// coordinates to put on page for small chunk of page
struct ResolvedRow {
    coordinate: Coordinate,
    extent: Extent,
    row: ContentRow,
}

enum ContentRow {
    Single(ResolvedField),
    Split {
        left: ResolvedField,
        right: ResolvedField,
    },
}

enum Alignment {
    Center,
    Left,
    Right,
}

struct ResolvedField {
    content: Vec<ResolvedLine>,
    alignment: Alignment,
}

// where wrapping is dealt with
struct ResolvedLine {
    text: GlyphRow,
}

pub fn layout(hir: &HIR) -> Vec<GlyphRow> {
    let mut glyph_rows: Vec<GlyphRow> = Vec::new();
    for invoke in &hir.invokes {
        println!("inscribing {}", invoke.name);
        glyph_rows
            .extend(inscribe::inscribe(&invoke, &hir.layout[&invoke.name]));
    }
    glyph_rows
}
