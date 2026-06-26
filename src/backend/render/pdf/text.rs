use printpdf::{Op, PdfFontHandle, Pt, TextItem, font::ParsedFont};

use crate::layout::LayoutEngine;

pub struct TextRun<'a> {
    pub text: &'a str,
    pub font: PdfFontHandle,
    pub parsed_font: Option<&'a ParsedFont>,
}

impl<'a> TextRun<'a> {
    pub fn new(text: &'a str, font: PdfFontHandle, parsed_font: Option<&'a ParsedFont>) -> Self {
        Self {
            text,
            font,
            parsed_font,
        }
    }

    pub fn show_text_item(&self) -> TextItem {
        match &self.font {
            PdfFontHandle::Builtin(_) => TextItem::Text(sanitize_builtin_text(self.text)),
            PdfFontHandle::External(_) => TextItem::Text(self.text.to_string()),
        }
    }
}

pub fn measure_text_width(text: &str, font_size: f32, parsed_font: Option<&ParsedFont>) -> f32 {
    let Some(font) = parsed_font else {
        return LayoutEngine::estimate_text_width(text, font_size);
    };

    let units_per_em = font.font_metrics.units_per_em as f32;
    if units_per_em <= 0.0 {
        return LayoutEngine::estimate_text_width(text, font_size);
    }

    let width_units = text
        .chars()
        .filter_map(|ch| font.lookup_glyph_index(ch as u32))
        .map(|glyph_id| font.get_horizontal_advance(glyph_id) as f32)
        .sum::<f32>();

    width_units * font_size / units_per_em
}

pub fn ascent_pt(font_size: f32, parsed_font: Option<&ParsedFont>) -> f32 {
    let Some(font) = parsed_font else {
        return font_size * 0.8;
    };

    let units_per_em = font.font_metrics.units_per_em as f32;
    if units_per_em <= 0.0 {
        return font_size * 0.8;
    }

    font.font_metrics.ascent as f32 * font_size / units_per_em
}

pub fn wrap_text_with_measure<F>(
    content: &str,
    max_width: f32,
    font_size: f32,
    nowrap: bool,
    mut measure: F,
) -> Vec<String>
where
    F: FnMut(&str, f32) -> f32,
{
    if content.is_empty() {
        return vec![String::new()];
    }

    if nowrap || max_width <= 0.0 {
        return vec![content.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in content.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };

        if measure(&candidate, font_size) <= max_width {
            current = candidate;
        } else {
            if !current.is_empty() {
                lines.push(current);
            }
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        vec![content.to_string()]
    } else {
        lines
    }
}

pub fn set_font_ops(ops: &mut Vec<Op>, font: PdfFontHandle, font_size: f32, line_height: f32) {
    ops.push(Op::SetFont {
        font,
        size: Pt(font_size),
    });
    ops.push(Op::SetLineHeight {
        lh: Pt(line_height),
    });
}

pub fn sanitize_builtin_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '\u{2013}' | '\u{2014}' | '\u{2011}' | '\u{2010}' => '-',
            '\u{00b7}' | '\u{2022}' => '|',
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{201c}' | '\u{201d}' => '"',
            '\u{00a0}' => ' ',
            _ => ch,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use printpdf::{BuiltinFont, PdfFontHandle};

    use super::{TextRun, sanitize_builtin_text};

    #[test]
    fn sanitize_builtin_text_replaces_unicode_punctuation() {
        assert_eq!(
            sanitize_builtin_text("September 2025 – Present · Toronto"),
            "September 2025 - Present | Toronto"
        );
    }

    #[test]
    fn builtin_text_path_sanitizes_unicode_punctuation() {
        let run = TextRun::new(
            "September 2025 – Present · Toronto",
            PdfFontHandle::Builtin(BuiltinFont::Helvetica),
            None,
        );

        assert_eq!(
            run.show_text_item(),
            printpdf::TextItem::Text("September 2025 - Present | Toronto".to_string())
        );
    }

    #[test]
    fn external_text_without_font_metrics_preserves_unicode_punctuation() {
        let run = TextRun::new(
            "September 2025 – Present · Toronto",
            PdfFontHandle::External(printpdf::FontId::new()),
            None,
        );

        assert_eq!(
            run.show_text_item(),
            printpdf::TextItem::Text("September 2025 – Present · Toronto".to_string())
        );
    }
}
