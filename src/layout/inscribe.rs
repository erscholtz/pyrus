use crate::hir::hir_types::Content;
use crate::hir::hir_types::Invoke;
use crate::hir::hir_types::Layout;
use crate::hir::hir_types::Sizing;

#[derive(Debug, Clone, Copy)]
pub enum Sizes {
    Sm,
    Md,
    Lg,
    Xl,
}

#[derive(Debug, Clone, Copy)]
pub enum Font {
    Serif,
    Bold,
    Italic,
    NerdFont,
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    char: char,
    size: Sizes,
    font: Font,
}

#[derive(Debug, Clone)]
pub struct GlyphRow {
    glyphs: Vec<Glyph>,
    offset: usize,
}

impl Glyph {
    fn new(char: char, size: &Sizing, font: Font) -> Self {
        let layout_size = match size {
            Sizing::Sm => Sizes::Sm,
            Sizing::Md => Sizes::Md,
            Sizing::Lg => Sizes::Lg,
            Sizing::Xl => Sizes::Xl,
        };
        Self {
            char,
            size: layout_size,
            font,
        }
    }

    fn inscribe_text(text: &str, size: &Sizing) -> Vec<Glyph> {
        text.chars()
            .map(|c| Glyph::new(c, size, Font::Serif))
            .collect()
    }

    fn inscribe_bold(text: &str, size: &Sizing) -> Vec<Glyph> {
        text.chars()
            .map(|c| Glyph::new(c, size, Font::Bold))
            .collect()
    }

    fn inscribe_italic(text: &str, size: &Sizing) -> Vec<Glyph> {
        text.chars()
            .map(|c| Glyph::new(c, size, Font::Italic))
            .collect()
    }

    fn inscribe_nerd(text: &str, size: &Sizing) -> Vec<Glyph> {
        text.chars()
            .map(|c| Glyph::new(c, size, Font::NerdFont))
            .collect()
    }
}

impl GlyphRow {
    pub fn new(glyphs: Vec<Glyph>, offset: usize) -> Self {
        Self { glyphs, offset }
    }
}

pub fn inscribe(invoke: &Invoke, layout: &Layout) -> Vec<GlyphRow> {
    let mut glyph_rows: Vec<GlyphRow> = Vec::new();
    for elem in &invoke.elements {
        let size = &layout.sizing[elem.0.as_str()];
        for content in elem.1 {
            let glyphs = match content {
                Content::Text(text) => Glyph::inscribe_text(text, size),
                Content::Bold(text) => Glyph::inscribe_bold(text, size),
                Content::Italic(text) => Glyph::inscribe_italic(text, size),
                Content::NerdFont(text) => Glyph::inscribe_nerd(text, size),

                _ => Vec::new(),
            };
            let len = glyphs.len();
            let row = GlyphRow::new(glyphs, len);
            glyph_rows.push(row);
        }
    }
    glyph_rows
}
