use crate::backend::render::pdf::style::StyleLookup;
use crate::backend::render::pdf::text;
use crate::hir::hir_types::{
    FuncId, HIRModule, HirElementOp, Op as HirOp, ReturnSummary, StyleAttributes,
};
use crate::layout::ComputedLayout;

const PAGE_WIDTH_PT: f32 = 595.0;

pub trait PdfTextMeasure {
    fn measure_element_text(
        &mut self,
        hlir: &HIRModule,
        element_index: usize,
        text: &str,
        font_size: f32,
    ) -> f32;
}

pub struct PdfLayoutEngine<'a, M> {
    measure: &'a mut M,
}

#[derive(Debug, Clone, Copy, Default)]
struct BoxEdges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct ElementBox {
    margin: BoxEdges,
    padding: BoxEdges,
}

impl<'a, M: PdfTextMeasure> PdfLayoutEngine<'a, M> {
    pub fn new(measure: &'a mut M) -> Self {
        Self { measure }
    }

    pub fn compute_document_flow(&mut self, hlir: &HIRModule) -> Vec<ComputedLayout> {
        let document_box = ElementBox::from_attributes(&hlir.document_styles);
        let mut layouts = Vec::new();
        let mut current_y = document_box.margin.top + document_box.padding.top;
        let x_offset = document_box.margin.left + document_box.padding.left;
        let page_width =
            (PAGE_WIDTH_PT - document_box.margin.horizontal() - document_box.padding.horizontal())
                .max(0.0);

        let document_id = FuncId(hlir.functions.len().saturating_sub(1));
        if let Some(document) = hlir.functions.get(&document_id) {
            for op in &document.body.items {
                match op {
                    HirOp::HirElementEmit { index } => self.process_element(
                        *index,
                        hlir,
                        &mut layouts,
                        &mut current_y,
                        page_width,
                        x_offset,
                        None,
                    ),
                    HirOp::FuncCall { func, .. } => {
                        if let Some(function) = hlir.functions.get(func) {
                            if let ReturnSummary::SingleElem(element_id) = function.return_summary {
                                self.process_element(
                                    element_id,
                                    hlir,
                                    &mut layouts,
                                    &mut current_y,
                                    page_width,
                                    x_offset,
                                    None,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        layouts
    }

    fn process_element(
        &mut self,
        element_index: usize,
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        current_y: &mut f32,
        page_width: f32,
        x_offset: f32,
        marker: Option<String>,
    ) {
        let Some(element) = hlir.elements.get(element_index) else {
            return;
        };
        let attrs = hlir
            .attributes
            .find_node(element.attributes_ref())
            .map(|node| &node.computed);
        let element_box = attrs.map(ElementBox::from_attributes).unwrap_or_default();
        let content_x = x_offset + element_box.margin.left + element_box.padding.left;
        let content_width =
            (page_width - element_box.margin.horizontal() - element_box.padding.horizontal())
                .max(0.0);

        match element {
            HirElementOp::Text { .. } | HirElementOp::Link { .. } => {
                let text_height = self.push_text_layout(
                    element_index,
                    hlir,
                    layouts,
                    x_offset,
                    *current_y,
                    page_width,
                    marker,
                );
                *current_y += text_height;
            }
            HirElementOp::List { children, .. } => {
                *current_y += element_box.margin.top + element_box.padding.top;
                let style =
                    attrs.map(|attrs| StyleLookup::with_fallback(attrs, &hlir.document_styles));
                let marker_width = style.and_then(StyleLookup::marker_width).unwrap_or(10.0);
                let marker_gap = style.and_then(StyleLookup::marker_gap).unwrap_or(4.0);
                let list_padding_left = style
                    .and_then(|style| style.length("list-indent"))
                    .unwrap_or(0.0);
                let item_x = content_x + list_padding_left + marker_width + marker_gap;
                let marker_x = content_x + list_padding_left;
                let item_width =
                    (content_width - list_padding_left - marker_width - marker_gap).max(0.0);

                for (item_idx, child_idx) in children.iter().enumerate() {
                    let marker = style.and_then(|style| style.list_marker(item_idx));
                    self.process_list_item(
                        *child_idx, hlir, layouts, current_y, item_width, item_x, marker, marker_x,
                    );
                }

                *current_y += element_box.padding.bottom + element_box.margin.bottom;
            }
            HirElementOp::Section { children, .. }
                if attrs.is_some_and(|attrs| {
                    StyleLookup::with_fallback(attrs, &hlir.document_styles).is_flex_row()
                }) =>
            {
                *current_y += element_box.margin.top + element_box.padding.top;
                let style =
                    attrs.map(|attrs| StyleLookup::with_fallback(attrs, &hlir.document_styles));
                if self.is_packed_flex_row(children, style) {
                    self.process_packed_row_children(
                        children,
                        hlir,
                        layouts,
                        current_y,
                        content_width,
                        content_x,
                        style,
                    );
                } else {
                    self.process_row_children(
                        children,
                        hlir,
                        layouts,
                        current_y,
                        content_width,
                        content_x,
                        style,
                    );
                }
                *current_y += element_box.padding.bottom + element_box.margin.bottom;
            }
            HirElementOp::Section { children, .. } => {
                *current_y += element_box.margin.top + element_box.padding.top;
                for child_idx in children {
                    self.process_element(
                        *child_idx,
                        hlir,
                        layouts,
                        current_y,
                        content_width,
                        content_x,
                        None,
                    );
                }
                *current_y += element_box.padding.bottom + element_box.margin.bottom;
            }
            HirElementOp::Image { .. } | HirElementOp::Table { .. } => {
                *current_y += element_box.margin.top + element_box.padding.top;
                *current_y += 10.0 + element_box.padding.bottom + element_box.margin.bottom;
            }
            HirElementOp::Separator { .. } => {
                *current_y += element_box.margin.top + element_box.padding.top;
                let height = attrs
                    .map(StyleLookup::new)
                    .and_then(StyleLookup::separator_height)
                    .unwrap_or(1.0);

                layouts.push(ComputedLayout {
                    x: content_x,
                    y: *current_y,
                    width: content_width,
                    height,
                    box_x: content_x,
                    box_y: *current_y,
                    box_width: content_width,
                    box_height: height,
                    element_index,
                    marker,
                    marker_x: None,
                    marker_y: None,
                    nowrap: false,
                });

                *current_y += height + element_box.padding.bottom + element_box.margin.bottom;
            }
        }
    }

    fn process_list_item(
        &mut self,
        element_index: usize,
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        current_y: &mut f32,
        page_width: f32,
        x_offset: f32,
        marker: Option<String>,
        marker_x: f32,
    ) {
        let before_len = layouts.len();
        self.process_element(
            element_index,
            hlir,
            layouts,
            current_y,
            page_width,
            x_offset,
            marker,
        );

        for layout in &mut layouts[before_len..] {
            if layout.marker.is_some() {
                layout.marker_x = Some(marker_x);
                layout.marker_y = Some(layout.y);
            }
        }
    }

    fn process_row_children(
        &mut self,
        children: &[usize],
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        current_y: &mut f32,
        content_width: f32,
        content_x: f32,
        style: Option<StyleLookup<'_>>,
    ) {
        let Some((&right_child, left_children)) = children.split_last() else {
            return;
        };

        let gap = style.and_then(StyleLookup::gap).unwrap_or(12.0);
        let right_width = self
            .natural_outer_width(right_child, hlir, content_width)
            .unwrap_or(0.0)
            .min((content_width - gap).max(0.0));
        let right_x = content_x + (content_width - right_width).max(0.0);
        let left_width = (right_x - content_x - gap).max(0.0);

        let mut row_height = 0.0_f32;
        for child_idx in left_children {
            row_height = row_height.max(self.push_row_child_layout(
                *child_idx, hlir, layouts, content_x, *current_y, left_width,
            ));
        }
        row_height = row_height.max(self.push_row_child_layout(
            right_child,
            hlir,
            layouts,
            right_x,
            *current_y,
            right_width,
        ));

        *current_y += row_height;
    }

    fn process_packed_row_children(
        &mut self,
        children: &[usize],
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        current_y: &mut f32,
        content_width: f32,
        content_x: f32,
        style: Option<StyleLookup<'_>>,
    ) {
        let gap = style.and_then(StyleLookup::gap).unwrap_or(12.0);
        let (child_widths, total_width) =
            self.measure_packed_row_children(children, hlir, content_width, gap);
        let row_right = content_x + content_width;
        let mut child_x = if style.is_some_and(StyleLookup::is_flex_end) {
            row_right - total_width.min(content_width)
        } else {
            content_x
        };

        let mut row_height = 0.0_f32;
        for (idx, (child_idx, child_width)) in children.iter().zip(child_widths).enumerate() {
            if idx > 0 {
                child_x += gap;
            }
            row_height = row_height.max(self.push_row_child_layout(
                *child_idx,
                hlir,
                layouts,
                child_x,
                *current_y,
                child_width,
            ));
            child_x += child_width;
        }

        *current_y += row_height;
    }

    fn push_row_child_layout(
        &mut self,
        element_index: usize,
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        x: f32,
        y: f32,
        width: f32,
    ) -> f32 {
        let before_len = layouts.len();
        let mut child_y = y;
        self.process_element(element_index, hlir, layouts, &mut child_y, width, x, None);

        let emitted_height = layouts[before_len..]
            .iter()
            .map(|layout| layout.box_y + layout.box_height - y)
            .fold(0.0_f32, f32::max);

        emitted_height.max(child_y - y)
    }

    fn push_text_layout(
        &mut self,
        element_index: usize,
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        x: f32,
        y: f32,
        width: f32,
        marker: Option<String>,
    ) -> f32 {
        let Some(HirElementOp::Text { content, .. } | HirElementOp::Link { content, .. }) =
            hlir.elements.get(element_index)
        else {
            return 0.0;
        };
        let Some(attrs) = hlir
            .attributes
            .find_node(hlir.elements[element_index].attributes_ref())
            .map(|node| &node.computed)
        else {
            return 0.0;
        };
        let style = StyleLookup::with_fallback(attrs, &hlir.document_styles);
        let font_size = style.font_size();
        let line_height = style.line_height(font_size);
        let element_box = ElementBox::from_attributes(attrs);
        let box_x = x + element_box.margin.left;
        let box_y = y + element_box.margin.top;
        let content_x = box_x + element_box.padding.left;
        let content_y = box_y + element_box.padding.top;
        let content_width =
            (width - element_box.margin.horizontal() - element_box.padding.horizontal()).max(0.0);
        let nowrap = style.is_nowrap();
        let lines = text::wrap_text_with_measure(
            content,
            content_width,
            font_size,
            nowrap,
            |candidate, size| {
                self.measure
                    .measure_element_text(hlir, element_index, candidate, size)
            },
        );
        let line_count = lines.len().max(1) as f32;
        let height = line_count * line_height;
        let box_height = height + element_box.padding.vertical();
        let rendered_width = lines
            .iter()
            .map(|line| {
                self.measure
                    .measure_element_text(hlir, element_index, line, font_size)
            })
            .fold(0.0_f32, f32::max)
            .min(content_width);
        let layout_x = if style.is_text_align_right() {
            content_x + (content_width - rendered_width).max(0.0)
        } else {
            content_x
        };
        let layout_width = if style.is_text_align_right() && nowrap {
            rendered_width
        } else {
            content_width
        };

        layouts.push(ComputedLayout {
            x: layout_x,
            y: content_y,
            width: layout_width,
            height,
            box_x,
            box_y,
            box_width: (content_width + element_box.padding.horizontal()).max(0.0),
            box_height,
            element_index,
            marker,
            marker_x: None,
            marker_y: None,
            nowrap,
        });

        element_box.margin.top + box_height + element_box.margin.bottom
    }

    fn measure_packed_row_children(
        &mut self,
        children: &[usize],
        hlir: &HIRModule,
        content_width: f32,
        gap: f32,
    ) -> (Vec<f32>, f32) {
        let mut child_widths = Vec::with_capacity(children.len());
        let mut total_child_width = 0.0;

        for child_idx in children {
            let remaining_width = (content_width - total_child_width).max(0.0);
            let child_width = self
                .natural_outer_width(*child_idx, hlir, remaining_width)
                .unwrap_or(remaining_width);
            child_widths.push(child_width);
            total_child_width += child_width;
        }

        let total_gap = gap * children.len().saturating_sub(1) as f32;
        (child_widths, total_child_width + total_gap)
    }

    fn natural_outer_width(
        &mut self,
        element_index: usize,
        hlir: &HIRModule,
        available_width: f32,
    ) -> Option<f32> {
        let element = hlir.elements.get(element_index)?;
        match element {
            HirElementOp::Text { content, .. } | HirElementOp::Link { content, .. } => {
                let attrs = hlir
                    .attributes
                    .find_node(element.attributes_ref())?
                    .computed
                    .clone();
                let style = StyleLookup::with_fallback(&attrs, &hlir.document_styles);
                let font_size = style.font_size();
                let element_box = ElementBox::from_attributes(&attrs);
                let explicit_width = style.length("width");
                let content_width = explicit_width.unwrap_or_else(|| {
                    self.measure
                        .measure_element_text(hlir, element_index, content, font_size)
                });
                Some(
                    (content_width
                        + element_box.margin.horizontal()
                        + element_box.padding.horizontal())
                    .min(available_width),
                )
            }
            HirElementOp::Section { children, .. } => {
                let attrs = hlir
                    .attributes
                    .find_node(element.attributes_ref())?
                    .computed
                    .clone();
                let style = StyleLookup::with_fallback(&attrs, &hlir.document_styles);
                let element_box = ElementBox::from_attributes(&attrs);
                if let Some(explicit_width) = style.length("width") {
                    return Some(
                        (explicit_width
                            + element_box.margin.horizontal()
                            + element_box.padding.horizontal())
                        .min(available_width),
                    );
                }

                let child_available = (available_width
                    - element_box.margin.horizontal()
                    - element_box.padding.horizontal())
                .max(0.0);
                let child_width = if self.is_packed_flex_row(children, Some(style)) {
                    let gap = style.gap().unwrap_or(12.0);
                    self.natural_packed_row_width(children, hlir, child_available, gap)?
                } else {
                    children
                        .iter()
                        .filter_map(|child_idx| {
                            self.natural_outer_width(*child_idx, hlir, child_available)
                        })
                        .fold(None, |max_width: Option<f32>, width| {
                            Some(max_width.map_or(width, |current| current.max(width)))
                        })?
                };

                Some(
                    (child_width
                        + element_box.margin.horizontal()
                        + element_box.padding.horizontal())
                    .min(available_width),
                )
            }
            _ => None,
        }
    }

    fn natural_packed_row_width(
        &mut self,
        children: &[usize],
        hlir: &HIRModule,
        available_width: f32,
        gap: f32,
    ) -> Option<f32> {
        let mut total_child_width = 0.0;
        for child_idx in children {
            let remaining_width = (available_width - total_child_width).max(0.0);
            total_child_width += self.natural_outer_width(*child_idx, hlir, remaining_width)?;
        }

        Some(total_child_width + gap * children.len().saturating_sub(1) as f32)
    }

    fn is_packed_flex_row(&self, children: &[usize], style: Option<StyleLookup<'_>>) -> bool {
        style.is_some_and(|style| {
            style.is_flex_row() && children.len() > 2 && !style.is_space_between()
        })
    }
}

impl BoxEdges {
    fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    fn horizontal(self) -> f32 {
        self.left + self.right
    }

    fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

impl ElementBox {
    fn from_attributes(attributes: &StyleAttributes) -> Self {
        Self {
            margin: Self::edges_from_style(attributes.margin, attributes, "margin"),
            padding: Self::edges_from_style(attributes.padding, attributes, "padding"),
        }
    }

    fn edges_from_style(
        shorthand: Option<f32>,
        attributes: &StyleAttributes,
        property: &str,
    ) -> BoxEdges {
        let style = StyleLookup::new(attributes);
        let mut edges = style
            .raw(property)
            .and_then(Self::parse_edge_shorthand)
            .or_else(|| shorthand.map(BoxEdges::all))
            .unwrap_or_default();

        if let Some(value) = side_value(style, property, "top") {
            edges.top = value;
        }
        if let Some(value) = side_value(style, property, "right") {
            edges.right = value;
        }
        if let Some(value) = side_value(style, property, "bottom") {
            edges.bottom = value;
        }
        if let Some(value) = side_value(style, property, "left") {
            edges.left = value;
        }

        edges
    }

    fn parse_edge_shorthand(value: &str) -> Option<BoxEdges> {
        let parts: Vec<_> = value
            .split_whitespace()
            .filter_map(crate::backend::render::pdf::style::parse_css_length)
            .collect();

        match parts.as_slice() {
            [] => None,
            [all] => Some(BoxEdges::all(*all)),
            [vertical, horizontal] => Some(BoxEdges {
                top: *vertical,
                right: *horizontal,
                bottom: *vertical,
                left: *horizontal,
            }),
            [top, horizontal, bottom] => Some(BoxEdges {
                top: *top,
                right: *horizontal,
                bottom: *bottom,
                left: *horizontal,
            }),
            [top, right, bottom, left, ..] => Some(BoxEdges {
                top: *top,
                right: *right,
                bottom: *bottom,
                left: *left,
            }),
        }
    }
}

fn side_value(style: StyleLookup<'_>, property: &str, suffix: &str) -> Option<f32> {
    let key = format!("{property}-{suffix}");
    style.length(&key)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    struct FixedMeasure;

    impl PdfTextMeasure for FixedMeasure {
        fn measure_element_text(
            &mut self,
            _: &HIRModule,
            _: usize,
            text: &str,
            font_size: f32,
        ) -> f32 {
            text.len() as f32 * font_size
        }
    }

    #[test]
    fn measurement_trait_can_drive_right_aligned_widths() {
        let mut measure = FixedMeasure;
        assert_eq!(
            measure.measure_element_text(&empty_module(), 0, "abc", 2.0),
            6.0
        );
    }

    fn empty_module() -> HIRModule {
        HIRModule {
            file: String::new(),
            globals: HashMap::new(),
            functions: HashMap::new(),
            element_decls: HashMap::new(),
            attributes: crate::hir::hir_types::AttributeTree::new(),
            css_rules: Vec::new(),
            document_styles: StyleAttributes::default(),
            elements: Vec::new(),
            element_metadata: Vec::new(),
        }
    }
}
