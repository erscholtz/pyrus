use std::collections::HashMap;
use std::process::Command;

use printpdf::{BuiltinFont, FontId, PdfDocument, PdfFontHandle, font::ParsedFont};
use rust_fontconfig::{FcFontCache, FcPattern, FcWeight, PatternMatch};

use crate::backend::render::pdf::style::StyleLookup;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct FontKey {
    family: String,
    weight: u16,
    italic: bool,
}

pub struct FontFace {
    pub id: FontId,
    pub parsed: ParsedFont,
}

pub struct ResolvedFont<'a> {
    pub handle: PdfFontHandle,
    pub face: Option<&'a FontFace>,
}

pub struct FontRegistry {
    fontconfig: FcFontCache,
    faces: HashMap<FontKey, FontFace>,
}

impl FontRegistry {
    pub fn new() -> Self {
        Self {
            fontconfig: FcFontCache::build(),
            faces: HashMap::new(),
        }
    }

    pub fn resolve<'a>(
        &'a mut self,
        doc: &mut PdfDocument,
        style: StyleLookup<'_>,
    ) -> ResolvedFont<'a> {
        let family = style.font_family();
        let weight = style.font_weight();
        let italic = style.value("font-style").as_deref() == Some("italic");
        let key = FontKey {
            family: family.to_ascii_lowercase(),
            weight,
            italic,
        };

        if !self.faces.contains_key(&key) {
            if let Some(face) = self.load_face(doc, &family, weight, italic) {
                self.faces.insert(key.clone(), face);
            }
        }

        if let Some(face) = self.faces.get(&key) {
            return ResolvedFont {
                handle: PdfFontHandle::External(face.id.clone()),
                face: Some(face),
            };
        }

        ResolvedFont {
            handle: PdfFontHandle::Builtin(builtin_font(&family, weight)),
            face: None,
        }
    }

    fn load_face(
        &self,
        doc: &mut PdfDocument,
        family: &str,
        weight: u16,
        italic: bool,
    ) -> Option<FontFace> {
        if let Some(face) = load_face_from_system_fontconfig(doc, family, weight, italic) {
            return Some(face);
        }

        for candidate in self.family_candidates(family) {
            let mut trace = Vec::new();
            let Some(matched) = self.query_candidate(candidate, weight, italic, &mut trace) else {
                continue;
            };
            let Some(bytes) = self.fontconfig.get_font_bytes(&matched.id) else {
                continue;
            };

            let mut warnings = Vec::new();
            let Some(parsed) = ParsedFont::from_bytes(&bytes, 0, &mut warnings) else {
                continue;
            };
            if rejected_latin_substitute(&parsed) {
                continue;
            }

            let id = doc.add_font(&parsed);
            return Some(FontFace { id, parsed });
        }

        None
    }

    fn query_candidate(
        &self,
        candidate: &str,
        weight: u16,
        italic: bool,
        trace: &mut Vec<rust_fontconfig::TraceMsg>,
    ) -> Option<rust_fontconfig::FontMatch> {
        self.fontconfig.query(
            &FcPattern {
                name: Some(candidate.to_string()),
                family: Some(candidate.to_string()),
                weight: FcWeight::from_u16(weight),
                bold: PatternMatch::DontCare,
                italic: if italic {
                    PatternMatch::True
                } else {
                    PatternMatch::DontCare
                },
                ..Default::default()
            },
            trace,
        )
    }

    fn family_candidates<'a>(&self, family: &'a str) -> Vec<&'a str> {
        let fallback: &[&str] = match family.trim_matches('"').to_ascii_lowercase().as_str() {
            "georgia" | "times" | "times new roman" | "serif" => &[
                "Noto Serif Display",
                "Noto Serif",
                "DejaVu Serif",
                "Liberation Serif",
                "FreeSerif",
                "serif",
            ],
            "courier" | "courier new" | "monospace" => &[
                "Noto Sans Mono",
                "DejaVu Sans Mono",
                "Liberation Mono",
                "monospace",
            ],
            _ => &[
                "Noto Sans",
                "DejaVu Sans",
                "Liberation Sans",
                "Arial",
                "sans-serif",
            ],
        };

        let mut candidates = Vec::with_capacity(fallback.len() + 1);
        candidates.push(family);
        candidates.extend(fallback);
        candidates
    }
}

fn load_face_from_system_fontconfig(
    doc: &mut PdfDocument,
    family: &str,
    weight: u16,
    italic: bool,
) -> Option<FontFace> {
    let style = match (weight >= 600, italic) {
        (true, true) => "Bold Italic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (false, false) => "Regular",
    };
    let query = format!("{}:style={style}", family.trim_matches('"'));
    let output = Command::new("fc-match")
        .args(["-f", "%{file}\n", &query])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?;
    let path = path.lines().next()?.trim();
    if path.is_empty() {
        return None;
    }

    let bytes = std::fs::read(path).ok()?;
    let mut warnings = Vec::new();
    let parsed = ParsedFont::from_bytes(&bytes, 0, &mut warnings)?;
    let id = doc.add_font(&parsed);
    Some(FontFace { id, parsed })
}

fn rejected_latin_substitute(parsed: &ParsedFont) -> bool {
    parsed.font_name.as_ref().is_some_and(|name| {
        let name = name.to_ascii_lowercase();
        name.contains("hentaigana")
            || name.contains("cjk")
            || name.contains("arabic")
            || name.contains("hebrew")
            || name.contains("devanagari")
            || name.contains("thai")
    })
}

pub fn builtin_font(family: &str, weight: u16) -> BuiltinFont {
    let family = family.trim_matches('"').to_ascii_lowercase();
    let is_bold = weight >= 600;
    match family.as_str() {
        "times" | "times new roman" | "serif" | "georgia" if is_bold => BuiltinFont::TimesBold,
        "times" | "times new roman" | "serif" | "georgia" => BuiltinFont::TimesRoman,
        "courier" | "courier new" | "monospace" if is_bold => BuiltinFont::CourierBold,
        "courier" | "courier new" | "monospace" => BuiltinFont::Courier,
        _ if is_bold => BuiltinFont::HelveticaBold,
        _ => BuiltinFont::Helvetica,
    }
}
