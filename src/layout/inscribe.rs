use harfrust::Buffer;
use harfrust::Font as HarfFont;
use harfrust::ShapeOptions;
use harfrust::ShaperFont;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::Severity;
use crate::diagnostic::SourceLocation;
use crate::hir::hir_types::TextType;
use crate::layout::allocate::ContentRow;

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
    Nerd,
}

#[derive(Debug)]
pub struct GlyphRun {
    pub buffer: Buffer,
    pub units_per_em: f32,
}

pub fn inscribe(rows: &mut Vec<ContentRow>) -> Result<(), Diagnostic> {
    let serif_bytes =
        std::fs::read("fonts/newsreader/static/Newsreader_9pt-Light.ttf")
            .expect("Failed to read font file"); // Replace with your diagnostic handling later.
    let serif_bold_bytes =
        std::fs::read("fonts/newsreader/static/Newsreader_9pt-Medium.ttf")
            .expect("Failed to read font file"); // Replace with your diagnostic handling later.
    let italic_bytes =
        std::fs::read("fonts/newsreader/static/Newsreader_9pt-LightItalic.ttf")
            .expect("Failed to read font file"); // Replace with your diagnostic handling later.
    let nerd_bytes = std::fs::read(
        "fonts/inconsolata-nerd-font-mono/InconsolataNerdFontMono-Regular.ttf",
    )
    .expect("Failed to read font file"); // Replace with your diagnostic handling later.

    let serif_font = HarfFont::new(serif_bytes, 0).expect("valid font"); // Replace with your diagnostic handling later.
    let bold_font = HarfFont::new(serif_bold_bytes, 0).expect("valid font"); // Replace with your diagnostic handling later.
    let italic_font = HarfFont::new(italic_bytes, 0).expect("valid font"); // Replace with your diagnostic handling later.
    let nerd_font = HarfFont::new(nerd_bytes, 0).expect("valid font"); // Replace with your diagnostic handling later.

    let serif_shaping_font = ShaperFont::new(&serif_font);
    let bold_shaping_font = ShaperFont::new(&bold_font);
    let italic_shaping_font = ShaperFont::new(&italic_font);
    let nerd_shaping_font = ShaperFont::new(&nerd_font);

    for row in rows {
        let fields = match row {
            ContentRow::Single(field) => vec![field],
            ContentRow::Split { left, right } => vec![left, right],
        };
        for field in fields {
            let mut buffers = Vec::new();
            for op in &field.content {
                for text_ty in &op.content {
                    let (text, font) = match text_ty {
                        TextType::Text(text) => (text, Font::Serif),
                        TextType::Bold(text) => (text, Font::Bold),
                        TextType::Italic(text) => (text, Font::Italic),
                        TextType::Nerd(text) => (text, Font::Nerd),
                        TextType::Link { label, .. } => (label, Font::Serif),
                    };
                    let mut buffer = Buffer::new();
                    buffer.push_str(text);
                    buffer.guess_segment_properties();
                    let shaping_font = match font {
                        Font::Serif => &serif_shaping_font,
                        Font::Bold => &bold_shaping_font,
                        Font::Italic => &italic_shaping_font,
                        Font::Nerd => &nerd_shaping_font,
                    };
                    harfrust::shape(
                        shaping_font,
                        &mut buffer,
                        ShapeOptions::default(),
                    )
                    .map_err(|err| Diagnostic {
                        severity: Severity::Fatal,
                        message: format!("Failed to shape text: {err:?}"),
                        location: SourceLocation::new(0, 0, ""),
                        span: None,
                        help: None,
                    })?;
                    buffers.push(GlyphRun {
                        buffer,
                        units_per_em: shaping_font.units_per_em() as f32,
                    });
                }
            }
            field.glyph_run = buffers;
        }
    }
    Ok(())
}
