use std::collections::HashMap;
use taffy::style::{AvailableSpace, Dimension};
use taffy::style_helpers::{FromLength, FromPercent, TaffyAuto};
use taffy::{LengthPercentage, LengthPercentageAuto, NodeId, Rect, Size, Style, TaffyTree};

use crate::hir::hir_types::{FuncId, HIRModule, HirElementOp, Op, StyleAttributes};

pub fn setup_layout(hlir_module: &HIRModule) -> LayoutEngine {
    LayoutEngine::build_from_hlir_module(hlir_module)
}

const PAGE_WIDTH_PT: f32 = 595.0;
const DEFAULT_FONT_SIZE_PT: f32 = 12.0;
const DEFAULT_LINE_HEIGHT_MULTIPLIER: f32 = 1.2;

#[derive(Debug)]
pub struct LayoutEngine {
    tree: TaffyTree,
    root: NodeId,
    /// Maps element index (in hlir_module.elements) -> Taffy NodeId
    element_to_node: Vec<Option<NodeId>>,
    /// Maps CSS id -> Taffy NodeId
    id_to_node: HashMap<String, NodeId>,
}

#[derive(Debug, Clone)]
pub struct ComputedLayout {
    /// Content rectangle used for text placement and wrapping.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Border-box rectangle used for PDF painting and link annotations.
    pub box_x: f32,
    pub box_y: f32,
    pub box_width: f32,
    pub box_height: f32,
    pub element_index: usize,
    pub marker: Option<String>,
    pub marker_x: Option<f32>,
    pub marker_y: Option<f32>,
    pub nowrap: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct BoxEdges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
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

#[derive(Debug, Clone, Copy, Default)]
struct ElementBox {
    margin: BoxEdges,
    padding: BoxEdges,
}

impl ElementBox {
    fn from_attributes(attributes: &StyleAttributes) -> Self {
        Self {
            margin: Self::edges_from_style(attributes.margin, &attributes.style, "margin"),
            padding: Self::edges_from_style(attributes.padding, &attributes.style, "padding"),
        }
    }

    fn edges_from_style(
        shorthand: Option<f32>,
        style: &HashMap<String, String>,
        property: &str,
    ) -> BoxEdges {
        let mut edges = style
            .get(property)
            .and_then(|value| Self::parse_edge_shorthand(value))
            .or_else(|| shorthand.map(BoxEdges::all))
            .unwrap_or_default();

        if let Some(value) = Self::side_value(style, property, "top") {
            edges.top = value;
        }
        if let Some(value) = Self::side_value(style, property, "right") {
            edges.right = value;
        }
        if let Some(value) = Self::side_value(style, property, "bottom") {
            edges.bottom = value;
        }
        if let Some(value) = Self::side_value(style, property, "left") {
            edges.left = value;
        }

        edges
    }

    fn side_value(style: &HashMap<String, String>, property: &str, suffix: &str) -> Option<f32> {
        let key = format!("{property}-{suffix}");
        style
            .get(&key)
            .and_then(|value| StyleAttributes::parse_css_length(value))
    }

    fn parse_edge_shorthand(value: &str) -> Option<BoxEdges> {
        let parts: Vec<_> = value
            .split_whitespace()
            .filter_map(StyleAttributes::parse_css_length)
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

impl LayoutEngine {
    pub fn new() -> Self {
        LayoutEngine {
            tree: TaffyTree::new(),
            root: NodeId::new(0),
            element_to_node: Vec::new(),
            id_to_node: HashMap::new(),
        }
    }

    pub fn build_from_hlir_module(hlir_module: &HIRModule) -> Self {
        let mut layout = LayoutEngine::new();

        // Pre-allocate element_to_node to match elements size
        layout.element_to_node = vec![None; hlir_module.elements.len()];

        // Create root node with flex column layout for document flow
        let root_style = Style {
            display: taffy::style::Display::Flex,
            flex_direction: taffy::style::FlexDirection::Column,
            size: Size {
                width: Dimension::from_percent(1.0), // 100% width
                height: Dimension::AUTO,             // Auto height
            },
            ..Style::default()
        };
        layout.root = layout.tree.new_with_children(root_style, &[]).unwrap();

        // Get the document function
        let document_id = FuncId(hlir_module.functions.len() - 1);
        let document = hlir_module
            .functions
            .get(&document_id)
            .expect("document function not found");

        // Collect all child nodes first
        let mut child_nodes = Vec::new();
        for op in &document.body.ops {
            if let Some(node_id) = layout.process_op_and_get_node(op, hlir_module) {
                child_nodes.push(node_id);
            }
        }

        // Add all children to root
        for child_id in child_nodes {
            let _ = layout.tree.add_child(layout.root, child_id);
        }

        layout
    }

    fn process_op_and_get_node(&mut self, op: &Op, hlir_module: &HIRModule) -> Option<NodeId> {
        match op {
            Op::HirElementEmit { index } => {
                let element = hlir_module.elements.get(*index).expect("element not found");
                self.create_leaf_for_element(*index, element.attributes_ref(), hlir_module)
            }
            Op::FuncCall { func, .. } => {
                if let Some(function) = hlir_module.functions.get(func) {
                    if let Some(element_id) = function.body.returned_element_ref {
                        return self.create_node_from_element(element_id, hlir_module);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn create_leaf_for_element(
        &mut self,
        element_index: usize,
        attributes_ref: usize,
        hlir_module: &HIRModule,
    ) -> Option<NodeId> {
        let attributes = self.computed_attributes(attributes_ref, hlir_module)?;

        let style = Self::attr_to_style(attributes);
        let node_id = self.tree.new_leaf(style).ok()?;

        self.track_node(element_index, node_id, attributes);

        Some(node_id)
    }

    fn create_node_from_element(
        &mut self,
        element_index: usize,
        hlir_module: &HIRModule,
    ) -> Option<NodeId> {
        let element = match hlir_module.elements.get(element_index) {
            Some(e) => e,
            None => return None,
        };

        let attributes = self.computed_attributes(element.attributes_ref(), hlir_module)?;

        let style = Self::attr_to_style(attributes);
        let node_id = self.tree.new_leaf(style).ok()?;
        self.track_node(element_index, node_id, attributes);

        self.process_element_children(element, hlir_module, node_id);

        Some(node_id)
    }

    fn computed_attributes<'hir>(
        &self,
        attributes_ref: usize,
        hlir_module: &'hir HIRModule,
    ) -> Option<&'hir StyleAttributes> {
        hlir_module
            .attributes
            .find_node(attributes_ref)
            .map(|node| &node.computed)
    }

    fn track_node(&mut self, element_index: usize, node_id: NodeId, attributes: &StyleAttributes) {
        if element_index < self.element_to_node.len() {
            self.element_to_node[element_index] = Some(node_id);
        }

        if let Some(id) = &attributes.id {
            self.id_to_node.insert(id.clone(), node_id);
        }
    }

    fn process_element_children(
        &mut self,
        element: &HirElementOp,
        hlir_module: &HIRModule,
        parent_node: NodeId,
    ) {
        let Some(children) = element.child_elements() else {
            return;
        };

        for child_idx in children {
            if let Some(child_node) = self.create_node_from_element(*child_idx, hlir_module) {
                let _ = self.tree.add_child(parent_node, child_node);
            }
        }
    }

    pub fn attr_to_style(attributes: &StyleAttributes) -> Style {
        let margin_zero = LengthPercentageAuto::length(0.0);
        let padding_zero = LengthPercentage::length(0.0);

        // Parse margin from computed styles
        let margin = attributes
            .margin
            .map_or(margin_zero, |m| LengthPercentageAuto::length(m));

        // Parse padding from computed styles
        let padding = attributes
            .padding
            .map_or(padding_zero, |p| LengthPercentage::length(p));

        // Parse width from style map (e.g., "width: 100pt")
        let width = attributes
            .style_length("width")
            .map(Dimension::from_length)
            .unwrap_or(Dimension::AUTO);

        // Parse height from style map
        let height = attributes
            .style_length("height")
            .map(Dimension::from_length)
            .unwrap_or(Dimension::AUTO);

        // Parse display property
        let display = attributes
            .style_value("display")
            .and_then(|v| match v {
                "block" => Some(taffy::style::Display::Block),
                "flex" => Some(taffy::style::Display::Flex),
                "none" => Some(taffy::style::Display::None),
                _ => Some(taffy::style::Display::Block), // Default to block
            })
            .unwrap_or(taffy::style::Display::Block);

        // Parse flex-direction
        let flex_direction = attributes
            .style_value("flex-direction")
            .and_then(|v| match v {
                "row" => Some(taffy::style::FlexDirection::Row),
                "row-reverse" => Some(taffy::style::FlexDirection::RowReverse),
                "column" => Some(taffy::style::FlexDirection::Column),
                "column-reverse" => Some(taffy::style::FlexDirection::ColumnReverse),
                _ => None,
            })
            .unwrap_or(taffy::style::FlexDirection::Column); // Default to column for documents

        // Parse justify-content
        let justify_content = attributes
            .style_value("justify-content")
            .and_then(|v| match v {
                "flex-start" => Some(taffy::style::JustifyContent::FlexStart),
                "flex-end" => Some(taffy::style::JustifyContent::FlexEnd),
                "center" => Some(taffy::style::JustifyContent::Center),
                "space-between" => Some(taffy::style::JustifyContent::SpaceBetween),
                "space-around" => Some(taffy::style::JustifyContent::SpaceAround),
                "space-evenly" => Some(taffy::style::JustifyContent::SpaceEvenly),
                _ => None,
            })
            .unwrap_or(taffy::style::JustifyContent::FlexStart);

        // Parse align-items
        let align_items = attributes
            .style_value("align-items")
            .and_then(|v| match v {
                "flex-start" => Some(taffy::style::AlignItems::FlexStart),
                "flex-end" => Some(taffy::style::AlignItems::FlexEnd),
                "center" => Some(taffy::style::AlignItems::Center),
                "stretch" => Some(taffy::style::AlignItems::Stretch),
                "baseline" => Some(taffy::style::AlignItems::Baseline),
                _ => None,
            })
            .unwrap_or(taffy::style::AlignItems::Stretch);

        Style {
            margin: Rect {
                left: margin,
                right: margin,
                top: margin,
                bottom: margin,
            },
            padding: Rect {
                left: padding,
                right: padding,
                top: padding,
                bottom: padding,
            },
            size: Size { width, height },
            display,
            flex_direction,
            justify_content: Some(justify_content),
            align_items: Some(align_items),
            ..Style::default()
        }
    }

    /// Run Taffy layout computation with given available space
    pub fn compute_layout(&mut self, available_width: f32, _available_height: f32) {
        let size = Size {
            width: AvailableSpace::Definite(available_width),
            height: AvailableSpace::Definite(0.0), // Let height be determined by content
        };
        self.tree.compute_layout(self.root, size).unwrap();
    }

    /// Compute a simple document flow layout for elements that Taffy can't measure
    /// This stacks elements vertically with proper spacing based on font-size
    pub fn compute_document_flow(&self, hlir: &HIRModule) -> Vec<ComputedLayout> {
        let mut layouts = Vec::new();
        let document_box = ElementBox::from_attributes(&hlir.document_styles);
        let mut current_y = document_box.margin.top + document_box.padding.top;
        let x_offset = document_box.margin.left + document_box.padding.left;
        let page_width =
            (PAGE_WIDTH_PT - document_box.margin.horizontal() - document_box.padding.horizontal())
                .max(0.0);

        // Get document ops in order and recursively process all elements
        let document_id = FuncId(hlir.functions.len() - 1);
        if let Some(document) = hlir.functions.get(&document_id) {
            for op in &document.body.ops {
                match op {
                    Op::HirElementEmit { index } => {
                        self.process_element_for_layout(
                            *index,
                            hlir,
                            &mut layouts,
                            &mut current_y,
                            page_width,
                            x_offset,
                            None,
                        );
                    }
                    Op::FuncCall { func, .. } => {
                        if let Some(function) = hlir.functions.get(func) {
                            if let Some(element_id) = function.body.returned_element_ref {
                                self.process_element_for_layout(
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

    /// Recursively process an element and its children for layout
    fn process_element_for_layout(
        &self,
        element_index: usize,
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        current_y: &mut f32,
        page_width: f32,
        x_offset: f32,
        marker: Option<String>,
    ) {
        if let Some(element) = hlir.elements.get(element_index) {
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

                    let marker_width = attrs
                        .and_then(Self::parse_list_marker_width)
                        .unwrap_or(10.0);
                    let marker_gap = attrs.and_then(Self::parse_list_marker_gap).unwrap_or(4.0);
                    let list_padding_left = attrs
                        .and_then(|attrs| attrs.style_length("list-indent"))
                        .unwrap_or(0.0);
                    let item_x = content_x + list_padding_left + marker_width + marker_gap;
                    let marker_x = content_x + list_padding_left;
                    let item_width =
                        (content_width - list_padding_left - marker_width - marker_gap).max(0.0);

                    for (item_idx, child_idx) in children.iter().enumerate() {
                        let marker = attrs.and_then(|attrs| Self::list_marker(attrs, item_idx));
                        self.process_list_item_for_layout(
                            *child_idx, hlir, layouts, current_y, item_width, item_x, marker,
                            marker_x,
                        );
                    }

                    *current_y += element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::Section { children, .. } if Self::is_flex_row(attrs) => {
                    *current_y += element_box.margin.top + element_box.padding.top;

                    if Self::is_packed_flex_row(children, attrs) {
                        self.process_packed_row_children(
                            children,
                            hlir,
                            layouts,
                            current_y,
                            content_width,
                            content_x,
                            attrs,
                        );
                    } else {
                        self.process_row_children(
                            children,
                            hlir,
                            layouts,
                            current_y,
                            content_width,
                            content_x,
                            attrs,
                        );
                    }

                    *current_y += element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::Section { children, .. } => {
                    *current_y += element_box.margin.top + element_box.padding.top;

                    for child_idx in children {
                        self.process_element_for_layout(
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
                HirElementOp::Image { .. } => {
                    *current_y += element_box.margin.top + element_box.padding.top;
                    *current_y += 10.0 + element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::Table { .. } => {
                    *current_y += element_box.margin.top + element_box.padding.top;
                    // TODO this is wrong fix in the future, should adjust cursor based on table content
                    *current_y += 10.0 + element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::Separator { .. } => {
                    *current_y += element_box.margin.top + element_box.padding.top;

                    let height = attrs.and_then(Self::parse_separator_height).unwrap_or(1.0);

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
    }

    fn process_list_item_for_layout(
        &self,
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
        self.process_element_for_layout(
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
        &self,
        children: &[usize],
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        current_y: &mut f32,
        content_width: f32,
        content_x: f32,
        attrs: Option<&StyleAttributes>,
    ) {
        let Some((&right_child, left_children)) = children.split_last() else {
            return;
        };

        let gap = attrs.and_then(Self::parse_gap).unwrap_or(12.0);
        let right_width = self
            .natural_outer_width(right_child, hlir, content_width)
            .unwrap_or(0.0)
            .min((content_width - gap).max(0.0));
        let right_x = content_x + (content_width - right_width).max(0.0);
        let left_width = (right_x - content_x - gap).max(0.0);

        let mut row_height: f32 = 0.0;
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
        &self,
        children: &[usize],
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        current_y: &mut f32,
        content_width: f32,
        content_x: f32,
        attrs: Option<&StyleAttributes>,
    ) {
        let gap = attrs.and_then(Self::parse_gap).unwrap_or(12.0);
        let mut row_height: f32 = 0.0;
        let (child_widths, total_width) =
            self.measure_packed_row_children(children, hlir, content_width, gap);

        let row_right = content_x + content_width;
        let mut child_x = if attrs.is_some_and(Self::is_flex_end) {
            row_right - total_width.min(content_width)
        } else {
            content_x
        };

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

    fn measure_packed_row_children(
        &self,
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

    fn push_row_child_layout(
        &self,
        element_index: usize,
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        x: f32,
        y: f32,
        width: f32,
    ) -> f32 {
        let before_len = layouts.len();
        let mut child_y = y;

        self.process_element_for_layout(element_index, hlir, layouts, &mut child_y, width, x, None);

        let emitted_height = layouts[before_len..]
            .iter()
            .map(|layout| layout.box_y + layout.box_height - y)
            .fold(0.0_f32, f32::max);

        emitted_height.max(child_y - y)
    }

    fn push_text_layout(
        &self,
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

        let attrs = hlir
            .attributes
            .find_node(hlir.elements[element_index].attributes_ref())
            .map(|node| &node.computed);
        let font_size = attrs
            .and_then(Self::parse_font_size)
            .unwrap_or(DEFAULT_FONT_SIZE_PT);
        let line_height = attrs
            .and_then(|attrs| Self::parse_line_height(attrs, font_size))
            .unwrap_or(font_size * DEFAULT_LINE_HEIGHT_MULTIPLIER);
        let element_box = attrs.map(ElementBox::from_attributes).unwrap_or_default();
        let box_x = x + element_box.margin.left;
        let box_y = y + element_box.margin.top;
        let content_x = box_x + element_box.padding.left;
        let content_y = box_y + element_box.padding.top;
        let content_width =
            (width - element_box.margin.horizontal() - element_box.padding.horizontal()).max(0.0);
        let nowrap = attrs.is_some_and(Self::is_nowrap);
        let lines = Self::wrap_text_with_mode(content, content_width, font_size, nowrap);
        let line_count = lines.len().max(1) as f32;
        let height = line_count * line_height;
        let box_height = height + element_box.padding.vertical();
        let rendered_width = lines
            .iter()
            .map(|line| Self::estimate_text_width(line, font_size))
            .fold(0.0_f32, f32::max)
            .min(content_width);
        let text_align_right = attrs.is_some_and(Self::is_text_align_right);
        let layout_x = if text_align_right {
            content_x + (content_width - rendered_width).max(0.0)
        } else {
            content_x
        };
        let layout_width = if text_align_right && nowrap {
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

    fn outer_text_width(
        &self,
        element_index: usize,
        hlir: &HIRModule,
        measurement: (&str, f32),
        available_width: f32,
    ) -> f32 {
        let attrs = hlir
            .elements
            .get(element_index)
            .and_then(|element| hlir.attributes.find_node(element.attributes_ref()))
            .map(|node| &node.computed);
        let element_box = attrs.map(ElementBox::from_attributes).unwrap_or_default();
        let explicit_width = attrs.and_then(|attrs| attrs.style_length("width"));
        let content_width = explicit_width
            .unwrap_or_else(|| Self::estimate_text_width(measurement.0, measurement.1));

        (content_width + element_box.margin.horizontal() + element_box.padding.horizontal())
            .min(available_width)
    }

    fn natural_outer_width(
        &self,
        element_index: usize,
        hlir: &HIRModule,
        available_width: f32,
    ) -> Option<f32> {
        let element = hlir.elements.get(element_index)?;

        match element {
            HirElementOp::Text { .. } | HirElementOp::Link { .. } => self
                .text_measurement(element_index, hlir)
                .map(|measurement| {
                    self.outer_text_width(element_index, hlir, measurement, available_width)
                }),
            HirElementOp::Section { children, .. } => {
                let attrs = hlir
                    .attributes
                    .find_node(element.attributes_ref())
                    .map(|node| &node.computed);
                let element_box = attrs.map(ElementBox::from_attributes).unwrap_or_default();
                if let Some(explicit_width) = attrs.and_then(|attrs| attrs.style_length("width")) {
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
                let child_width = if Self::is_packed_flex_row(children, attrs) {
                    let gap = attrs.and_then(Self::parse_gap).unwrap_or(12.0);
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
        &self,
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

    fn text_measurement<'a>(
        &self,
        element_index: usize,
        hlir: &'a HIRModule,
    ) -> Option<(&'a str, f32)> {
        let element = hlir.elements.get(element_index)?;
        let (HirElementOp::Text { content, .. } | HirElementOp::Link { content, .. }) = element
        else {
            return None;
        };

        let attrs = hlir
            .attributes
            .find_node(element.attributes_ref())
            .map(|node| &node.computed);
        let font_size = attrs
            .and_then(Self::parse_font_size)
            .unwrap_or(DEFAULT_FONT_SIZE_PT);

        Some((content.as_str(), font_size))
    }

    fn is_flex_row(attrs: Option<&StyleAttributes>) -> bool {
        let Some(attrs) = attrs else {
            return false;
        };

        attrs
            .style
            .get("display")
            .is_some_and(|value| value.trim() == "flex")
            && attrs
                .style
                .get("flex-direction")
                .is_some_and(|value| value.trim() == "row")
    }

    fn is_nowrap(attrs: &StyleAttributes) -> bool {
        attrs
            .style
            .get("white-space")
            .is_some_and(|value| value.trim().trim_matches('"') == "nowrap")
    }

    fn is_packed_flex_row(children: &[usize], attrs: Option<&StyleAttributes>) -> bool {
        Self::is_flex_row(attrs) && children.len() > 2 && !attrs.is_some_and(Self::is_space_between)
    }

    fn is_space_between(attrs: &StyleAttributes) -> bool {
        attrs
            .style
            .get("justify-content")
            .is_some_and(|value| Self::css_keyword(value) == "space-between")
    }

    fn is_flex_end(attrs: &StyleAttributes) -> bool {
        attrs
            .style
            .get("justify-content")
            .is_some_and(|value| matches!(Self::css_keyword(value).as_str(), "flex-end" | "end"))
    }

    fn is_text_align_right(attrs: &StyleAttributes) -> bool {
        attrs
            .style
            .get("text-align")
            .is_some_and(|value| Self::css_keyword(value) == "right")
    }

    fn css_keyword(value: &str) -> String {
        let trimmed = value.trim().trim_matches('"');
        let parts: Vec<_> = trimmed.split_whitespace().collect();
        if parts.len() == 3 && matches!(parts[1], "-" | "Sub" | "Subtract") {
            return format!("{}-{}", parts[0], parts[2]).to_lowercase();
        }

        parts.join("").to_lowercase()
    }

    fn parse_gap(attrs: &StyleAttributes) -> Option<f32> {
        attrs
            .style_length("column-gap")
            .or_else(|| attrs.style_length("gap"))
    }

    fn parse_list_marker_width(attrs: &StyleAttributes) -> Option<f32> {
        attrs
            .style_length("marker-width")
            .or_else(|| attrs.style_length("list-marker-width"))
    }

    fn parse_list_marker_gap(attrs: &StyleAttributes) -> Option<f32> {
        attrs
            .style_length("marker-gap")
            .or_else(|| attrs.style_length("list-marker-gap"))
    }

    fn list_marker(attrs: &StyleAttributes, item_idx: usize) -> Option<String> {
        match attrs
            .style
            .get("list-style-type")
            .map(|value| value.trim().trim_matches('"').to_lowercase())
            .as_deref()
        {
            Some("none") => None,
            Some("decimal" | "number" | "numbered" | "ordered") => {
                Some(format!("{}.", item_idx + 1))
            }
            Some("disc" | "bullet") | None => Some("-".to_string()),
            Some(_) => Some("-".to_string()),
        }
    }

    fn parse_font_size(attributes: &StyleAttributes) -> Option<f32> {
        attributes.style_length("font-size")
    }

    fn parse_line_height(attributes: &StyleAttributes, font_size: f32) -> Option<f32> {
        let value = attributes.style.get("line-height")?.trim();
        let num_end = value
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(value.len());

        if num_end == value.len() {
            return value
                .parse::<f32>()
                .ok()
                .map(|multiple| multiple * font_size);
        }

        StyleAttributes::parse_css_length(value)
    }

    fn parse_separator_height(attributes: &StyleAttributes) -> Option<f32> {
        attributes.style_length("height")
    }

    pub(crate) fn wrap_text_with_mode(
        content: &str,
        max_width: f32,
        font_size: f32,
        nowrap: bool,
    ) -> Vec<String> {
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

            if Self::estimate_text_width(&candidate, font_size) <= max_width {
                current = candidate;
                continue;
            }

            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }

            if Self::estimate_text_width(word, font_size) <= max_width {
                current = word.to_string();
            } else {
                let mut piece = String::new();
                for ch in word.chars() {
                    let candidate = format!("{piece}{ch}");
                    if !piece.is_empty()
                        && Self::estimate_text_width(&candidate, font_size) > max_width
                    {
                        lines.push(piece);
                        piece = ch.to_string();
                    } else {
                        piece = candidate;
                    }
                }
                current = piece;
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }

        lines
    }

    pub(crate) fn estimate_text_width(text: &str, font_size: f32) -> f32 {
        text.chars()
            .map(|ch| {
                let width = match ch {
                    ' ' => 0.28,
                    'i' | 'l' | 'I' | '|' | '.' | ',' | ':' | ';' | '\'' => 0.25,
                    'm' | 'w' | 'M' | 'W' => 0.85,
                    'A'..='Z' => 0.62,
                    '0'..='9' => 0.55,
                    _ => 0.5,
                };
                width * font_size
            })
            .sum()
    }

    /// Get computed layout for an element by index
    pub fn get_element_layout(&self, element_index: usize) -> Option<ComputedLayout> {
        let node_id = self.element_to_node.get(element_index).copied()??;
        let layout = self.tree.layout(node_id).ok()?;

        Some(ComputedLayout {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
            box_x: layout.location.x,
            box_y: layout.location.y,
            box_width: layout.size.width,
            box_height: layout.size.height,
            element_index,
            marker: None,
            marker_x: None,
            marker_y: None,
            nowrap: false,
        })
    }

    /// Get layout for an element by its CSS ID
    pub fn get_layout_by_id(&self, id: &str) -> Option<ComputedLayout> {
        let node_id = self.id_to_node.get(id)?;
        let layout = self.tree.layout(*node_id).ok()?;
        let element_index = self
            .element_to_node
            .iter()
            .position(|&n| n == Some(*node_id))?;

        Some(ComputedLayout {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
            box_x: layout.location.x,
            box_y: layout.location.y,
            box_width: layout.size.width,
            box_height: layout.size.height,
            element_index,
            marker: None,
            marker_x: None,
            marker_y: None,
            nowrap: false,
        })
    }

    /// Iterate over all elements with their computed layouts
    pub fn iter_layouts(&self) -> impl Iterator<Item = ComputedLayout> + '_ {
        self.element_to_node
            .iter()
            .enumerate()
            .filter_map(|(idx, opt_node_id)| {
                let node_id = opt_node_id.as_ref()?;
                let layout = self.tree.layout(*node_id).ok()?;
                Some(ComputedLayout {
                    x: layout.location.x,
                    y: layout.location.y,
                    width: layout.size.width,
                    height: layout.size.height,
                    box_x: layout.location.x,
                    box_y: layout.location.y,
                    box_width: layout.size.width,
                    box_height: layout.size.height,
                    element_index: idx,
                    marker: None,
                    marker_x: None,
                    marker_y: None,
                    nowrap: false,
                })
            })
    }
}
