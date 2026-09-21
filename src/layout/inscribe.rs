use crate::diagnostic::CompilerDiagnostic;
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
    pub glyphs: Vec<Glyph>,
    pub name: String,
    pub offset: usize,
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
    pub fn new(glyphs: Vec<Glyph>, name: String, offset: usize) -> Self {
        Self {
            glyphs,
            name,
            offset,
        }
    }
}

pub fn inscribe(
    elem: (&String, &Vec<Content>),
    layout: &Layout,
) -> Result<GlyphRow, CompilerDiagnostic> {
    let size = &layout.sizing[elem.0.as_str()];
    let mut glyphs: Vec<Glyph> = Vec::new();
    for content in elem.1 {
        glyphs.extend(match content {
            Content::Text(text) => Glyph::inscribe_text(&text, size),
            Content::Bold(text) => Glyph::inscribe_bold(&text, size),
            Content::Italic(text) => Glyph::inscribe_italic(&text, size),
            Content::NerdFont(text) => Glyph::inscribe_nerd(&text, size),
            _ => Vec::new(),
        });
    }
    let len = glyphs.len();
    if len == 0 {
        // TODO proper error here
        // return Err(CompilerDiagnostic::Fatal(FatalError::new(
        //     "inscribe: no glyphs generated",
        // )));
    }
    Ok(GlyphRow::new(glyphs, elem.0.clone(), len))
}
