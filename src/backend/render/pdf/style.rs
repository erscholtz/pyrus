use std::collections::HashMap;

use printpdf::{Color, Rgb};

use crate::hir::hir_types::StyleAttributes;

const ALIASES: &[(&str, &str)] = &[
    ("font-size", "fontsize"),
    ("line-height", "lineheight"),
    ("font-weight", "fontweight"),
    ("font-family", "fontfamily"),
    ("font-style", "fontstyle"),
    ("text-align", "textalign"),
    ("white-space", "whitespace"),
    ("marker-width", "markerwidth"),
    ("marker-gap", "markergap"),
    ("padding-left", "paddingleft"),
    ("column-gap", "columngap"),
    ("justify-content", "justifycontent"),
    ("flex-direction", "flexdirection"),
    ("list-style-type", "liststyletype"),
    ("list-marker-width", "listmarkerwidth"),
    ("list-marker-gap", "listmarkergap"),
    ("border-bottom", "borderbottom"),
    ("margin-top", "margintop"),
    ("margin-right", "marginright"),
    ("margin-bottom", "marginbottom"),
    ("margin-left", "marginleft"),
    ("padding-top", "paddingtop"),
    ("padding-right", "paddingright"),
    ("padding-bottom", "paddingbottom"),
];

pub const DEFAULT_FONT_SIZE_PT: f32 = 12.0;
pub const DEFAULT_LINE_HEIGHT_MULTIPLIER: f32 = 1.2;

#[derive(Clone, Copy)]
pub struct StyleLookup<'a> {
    attrs: &'a StyleAttributes,
    fallback: Option<&'a StyleAttributes>,
}

#[derive(Clone)]
pub struct BorderStyle {
    pub width: f32,
    pub color: Color,
}

impl<'a> StyleLookup<'a> {
    pub fn new(attrs: &'a StyleAttributes) -> Self {
        Self {
            attrs,
            fallback: None,
        }
    }

    pub fn with_fallback(attrs: &'a StyleAttributes, fallback: &'a StyleAttributes) -> Self {
        Self {
            attrs,
            fallback: Some(fallback),
        }
    }

    pub fn attrs(self) -> &'a StyleAttributes {
        self.attrs
    }

    pub fn raw(self, property: &str) -> Option<&'a str> {
        self.attrs
            .style
            .get(property)
            .or_else(|| alias_for(property).and_then(|alias| self.attrs.style.get(alias)))
            .or_else(|| {
                inherited_property(property).then(|| {
                    self.fallback.and_then(|fallback| {
                        fallback.style.get(property).or_else(|| {
                            alias_for(property).and_then(|alias| fallback.style.get(alias))
                        })
                    })
                })?
            })
            .map(String::as_str)
    }

    pub fn value(self, property: &str) -> Option<String> {
        self.raw(property).map(normalize_css_value)
    }

    pub fn length(self, property: &str) -> Option<f32> {
        self.raw(property).and_then(parse_css_length)
    }

    pub fn font_size(self) -> f32 {
        self.length("font-size").unwrap_or(DEFAULT_FONT_SIZE_PT)
    }

    pub fn line_height(self, font_size: f32) -> f32 {
        self.raw("line-height")
            .and_then(|value| parse_line_height(value, font_size))
            .unwrap_or(font_size * DEFAULT_LINE_HEIGHT_MULTIPLIER)
    }

    pub fn font_family(self) -> String {
        self.value("font-family")
            .map(|value| value.trim_matches('"').to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "Helvetica".to_string())
    }

    pub fn font_weight(self) -> u16 {
        self.value("font-weight")
            .and_then(|value| {
                let lower = value.to_ascii_lowercase();
                match lower.as_str() {
                    "normal" => Some(400),
                    "bold" | "bolder" => Some(700),
                    _ => lower.parse::<u16>().ok(),
                }
            })
            .unwrap_or(400)
    }

    pub fn is_bold(self) -> bool {
        self.font_weight() >= 600
    }

    pub fn is_nowrap(self) -> bool {
        self.value("white-space").as_deref() == Some("nowrap")
    }

    pub fn is_text_align_right(self) -> bool {
        self.value("text-align").as_deref() == Some("right")
    }

    pub fn is_flex_row(self) -> bool {
        self.value("display").as_deref() == Some("flex")
            && self.value("flex-direction").as_deref() == Some("row")
    }

    pub fn is_space_between(self) -> bool {
        self.value("justify-content").as_deref() == Some("space-between")
    }

    pub fn is_flex_end(self) -> bool {
        matches!(
            self.value("justify-content").as_deref(),
            Some("flex-end" | "end")
        )
    }

    pub fn gap(self) -> Option<f32> {
        self.length("column-gap").or_else(|| self.length("gap"))
    }

    pub fn marker_width(self) -> Option<f32> {
        self.length("marker-width")
            .or_else(|| self.length("list-marker-width"))
    }

    pub fn marker_gap(self) -> Option<f32> {
        self.length("marker-gap")
            .or_else(|| self.length("list-marker-gap"))
    }

    pub fn list_marker(self, item_idx: usize) -> Option<String> {
        match self.value("list-style-type").as_deref() {
            Some("none") => None,
            Some("decimal" | "number" | "numbered" | "ordered") => {
                Some(format!("{}.", item_idx + 1))
            }
            Some("disc" | "bullet") | None => Some("-".to_string()),
            Some(_) => Some("-".to_string()),
        }
    }

    pub fn color(self) -> Option<Color> {
        parse_color_token(self.raw("color")?)
    }

    pub fn border(self, property: &str) -> Option<BorderStyle> {
        parse_border(self.raw(property), self)
    }

    pub fn separator_height(self) -> Option<f32> {
        self.length("height")
    }

    pub fn map(self) -> &'a HashMap<String, String> {
        &self.attrs.style
    }
}

pub fn normalize_css_value(value: &str) -> String {
    let trimmed = value.trim().trim_matches('"');
    let parts: Vec<_> = trimmed.split_whitespace().collect();

    if parts.len() == 3 && matches!(parts[1], "-" | "Sub" | "Subtract") {
        return format!("{}-{}", parts[0], parts[2]).to_ascii_lowercase();
    }

    if parts.len() > 1 {
        return parts.join("").to_ascii_lowercase();
    }

    trimmed.to_ascii_lowercase()
}

pub fn parse_css_length(value: &str) -> Option<f32> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("auto") {
        return None;
    }

    let num_end = value
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(value.len());

    let num_str = &value[..num_end];
    let unit_str = &value[num_end..].trim().to_ascii_lowercase();
    let num: f32 = num_str.parse().ok()?;

    match unit_str.as_str() {
        "pt" | "" => Some(num),
        "px" => Some(num * 0.75),
        "mm" => Some(num * 2.83465),
        "cm" => Some(num * 28.3465),
        "in" => Some(num * 72.0),
        _ => None,
    }
}

pub fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        icc_profile: None,
    })
}

fn alias_for(property: &str) -> Option<&'static str> {
    ALIASES
        .iter()
        .find_map(|(canonical, alias)| (*canonical == property).then_some(*alias))
}

fn inherited_property(property: &str) -> bool {
    matches!(
        property,
        "color"
            | "font-family"
            | "font-size"
            | "font-weight"
            | "font-style"
            | "line-height"
            | "text-align"
            | "white-space"
    )
}

fn parse_line_height(value: &str, font_size: f32) -> Option<f32> {
    let value = value.trim();
    let num_end = value
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(value.len());

    if num_end == value.len() {
        return value
            .parse::<f32>()
            .ok()
            .map(|multiple| multiple * font_size);
    }

    parse_css_length(value)
}

fn parse_border(value: Option<&str>, style: StyleLookup<'_>) -> Option<BorderStyle> {
    let value = value?.trim().trim_matches('"');
    if value.is_empty() || value.eq_ignore_ascii_case("none") {
        return None;
    }

    let mut width = None;
    let mut color = None;

    for part in value.split_whitespace() {
        if width.is_none() {
            width = parse_css_length(part);
        }

        if color.is_none() {
            color = parse_color_token(part);
        }
    }

    Some(BorderStyle {
        width: width.unwrap_or(1.0),
        color: color
            .or_else(|| style.color())
            .unwrap_or_else(|| rgb(0.0, 0.0, 0.0)),
    })
}

fn parse_color_token(value: &str) -> Option<Color> {
    let value = value.trim().trim_matches('"');
    let color = match value.to_ascii_lowercase().as_str() {
        "black" => rgb(0.0, 0.0, 0.0),
        "white" => rgb(1.0, 1.0, 1.0),
        "red" => rgb(1.0, 0.0, 0.0),
        "green" => rgb(0.0, 0.5, 0.0),
        "blue" => rgb(0.0, 0.0, 1.0),
        "gray" | "grey" => rgb(0.5, 0.5, 0.5),
        _ => return parse_hex_color(value),
    };

    Some(color)
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(rgb(red, green, blue))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::hir::hir_types::StyleAttributes;

    use super::{StyleLookup, normalize_css_value};

    #[test]
    fn style_alias_lookup_reads_dehyphenated_font_size() {
        let attrs = StyleAttributes {
            style: HashMap::from([("fontsize".to_string(), "14pt".to_string())]),
            ..StyleAttributes::default()
        };

        assert_eq!(StyleLookup::new(&attrs).font_size(), 14.0);
    }

    #[test]
    fn hyphenated_value_normalization_restores_subtract_token() {
        assert_eq!(
            normalize_css_value("space Subtract between"),
            "space-between"
        );
    }
}
