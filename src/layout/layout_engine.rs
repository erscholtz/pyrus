use std::collections::HashMap;
use taffy::style::{AvailableSpace, Dimension};
use taffy::style_helpers::{FromLength, FromPercent, TaffyAuto};
use taffy::{LengthPercentage, LengthPercentageAuto, NodeId, Rect, Size, Style, TaffyTree};

use crate::hir::hir_types::{FuncId, HIRModule, HirElementOp, Op, StyleAttributes};

pub fn setup_layout(hlir_module: &HIRModule) -> LayoutEngine {
    let layout = LayoutEngine::build_from_hlir_module(hlir_module);
    layout
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
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub element_index: usize,
    pub marker: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct BoxEdges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl BoxEdges {
    fn horizontal(self) -> f32 {
        self.left + self.right
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
        let mut edges = BoxEdges::default();

        if let Some(value) = shorthand {
            edges = BoxEdges {
                top: value,
                right: value,
                bottom: value,
                left: value,
            };
        }

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
            .and_then(|value| LayoutEngine::parse_css_length(value))
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
                // Document element - has metadata and computed styles
                let element = hlir_module.elements.get(*index).expect("element not found");
                let attributes_ref = match element {
                    HirElementOp::Section { attributes, .. } => *attributes,
                    HirElementOp::List { attributes, .. } => *attributes,
                    HirElementOp::Text { attributes, .. } => *attributes,
                    HirElementOp::Image { attributes, .. } => *attributes,
                    HirElementOp::Table { attributes, .. } => *attributes,
                };
                self.create_node_from_metadata(*index, attributes_ref, hlir_module)
            }
            Op::FuncCall { func, .. } => {
                // Template function call - element is in function's returned_element_ref
                if let Some(function) = hlir_module.functions.get(func) {
                    if let Some(element_id) = function.body.returned_element_ref {
                        // Template element - extract styles directly from DocElement
                        return self.create_node_from_element(element_id, hlir_module);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn create_node_from_metadata(
        &mut self,
        element_index: usize,
        attributes_ref: usize,
        hlir_module: &HIRModule,
    ) -> Option<NodeId> {
        let attributes = match hlir_module.attributes.find_node(attributes_ref) {
            Some(a) => &a.computed,
            None => return None,
        };

        let style = Self::attr_to_style(attributes);
        let node_id = match self.tree.new_leaf(style) {
            Ok(id) => id,
            Err(_) => return None,
        };

        if element_index < self.element_to_node.len() {
            self.element_to_node[element_index] = Some(node_id);
        }

        if let Some(id) = &attributes.id {
            self.id_to_node.insert(id.clone(), node_id);
        }

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

        // Get attributes_ref from the element and look up computed styles
        let attributes_ref = match element {
            HirElementOp::Section { attributes, .. } => *attributes,
            HirElementOp::List { attributes, .. } => *attributes,
            HirElementOp::Text { attributes, .. } => *attributes,
            HirElementOp::Image { attributes, .. } => *attributes,
            HirElementOp::Table { attributes, .. } => *attributes,
        };

        let attributes = match hlir_module.attributes.find_node(attributes_ref) {
            Some(node) => &node.computed,
            None => return None,
        };

        let style = Self::attr_to_style(attributes);

        let node_id = match self.tree.new_leaf(style) {
            Ok(id) => id,
            Err(_) => return None,
        };

        if element_index < self.element_to_node.len() {
            self.element_to_node[element_index] = Some(node_id);
        }

        if let Some(id) = &attributes.id {
            self.id_to_node.insert(id.clone(), node_id);
        }

        // Handle children (for Section, List, etc.)
        self.process_element_children(element, hlir_module, node_id);

        Some(node_id)
    }

    fn process_element_children(
        &mut self,
        element: &HirElementOp,
        hlir_module: &HIRModule,
        parent_node: NodeId,
    ) {
        // HlirElement stores children as indices into hlir_module.elements
        let children = match element {
            HirElementOp::Section { children, .. } => children,
            HirElementOp::List { children, .. } => children,
            HirElementOp::Text { .. } => return,  // No children
            HirElementOp::Image { .. } => return, // No children
            HirElementOp::Table { .. } => return, // No children,
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
            .style
            .get("width")
            .and_then(|v| Self::parse_css_length(v))
            .map(Dimension::from_length)
            .unwrap_or(Dimension::AUTO);

        // Parse height from style map
        let height = attributes
            .style
            .get("height")
            .and_then(|v| Self::parse_css_length(v))
            .map(Dimension::from_length)
            .unwrap_or(Dimension::AUTO);

        // Parse display property
        let display = attributes
            .style
            .get("display")
            .map(|v| v.as_str())
            .and_then(|v| match v {
                "block" => Some(taffy::style::Display::Block),
                "flex" => Some(taffy::style::Display::Flex),
                "none" => Some(taffy::style::Display::None),
                _ => Some(taffy::style::Display::Block), // Default to block
            })
            .unwrap_or(taffy::style::Display::Block);

        // Parse flex-direction
        let flex_direction = attributes
            .style
            .get("flex-direction")
            .map(|v| v.as_str())
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
            .style
            .get("justify-content")
            .map(|v| v.as_str())
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
            .style
            .get("align-items")
            .map(|v| v.as_str())
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

    /// Parse a CSS length value like "15pt", "20px", "10mm", or just "15"
    /// Returns the numeric value in points (pt)
    fn parse_css_length(value: &str) -> Option<f32> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }

        // Handle "auto" keyword
        if value.eq_ignore_ascii_case("auto") {
            return None;
        }

        // Find where the number ends and unit begins
        let num_end = value
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(value.len());

        let num_str = &value[..num_end];
        let unit_str = &value[num_end..].trim().to_lowercase();

        let num: f32 = num_str.parse().ok()?;

        // Convert to points based on unit
        match unit_str.as_str() {
            "pt" => Some(num),
            "px" => Some(num * 0.75),    // 1px ≈ 0.75pt
            "mm" => Some(num * 2.83465), // 1mm ≈ 2.83465pt
            "cm" => Some(num * 28.3465), // 1cm ≈ 28.3465pt
            "in" => Some(num * 72.0),    // 1in = 72pt
            "" => Some(num),             // No unit, assume points
            _ => {
                // Unknown unit, try to parse as number anyway
                num_str.parse().ok()
            }
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
            let attributes_ref = Self::element_attributes_ref(element);
            let attrs = hlir
                .attributes
                .find_node(attributes_ref)
                .map(|node| &node.computed);
            let element_box = attrs.map(ElementBox::from_attributes).unwrap_or_default();

            *current_y += element_box.margin.top + element_box.padding.top;

            let content_x = x_offset + element_box.margin.left + element_box.padding.left;
            let content_width =
                (page_width - element_box.margin.horizontal() - element_box.padding.horizontal())
                    .max(0.0);

            match element {
                HirElementOp::Text { content, .. } => {
                    let font_size = attrs
                        .and_then(Self::parse_font_size)
                        .unwrap_or(DEFAULT_FONT_SIZE_PT);
                    let line_height = attrs
                        .and_then(|attrs| Self::parse_line_height(attrs, font_size))
                        .unwrap_or(font_size * DEFAULT_LINE_HEIGHT_MULTIPLIER);
                    let line_count = Self::wrap_text(content, content_width, font_size)
                        .len()
                        .max(1) as f32;
                    let height = line_count * line_height;

                    layouts.push(ComputedLayout {
                        x: content_x,
                        y: *current_y,
                        width: content_width,
                        height,
                        element_index,
                        marker,
                    });

                    *current_y += height + element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::List { children, .. } => {
                    for (item_idx, child_idx) in children.iter().enumerate() {
                        let marker = attrs.and_then(|attrs| Self::list_marker(attrs, item_idx));
                        self.process_element_for_layout(
                            *child_idx,
                            hlir,
                            layouts,
                            current_y,
                            (content_width - 20.0).max(0.0),
                            content_x + 20.0,
                            marker,
                        );
                    }

                    *current_y += element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::Section { children, .. } if Self::is_flex_row(attrs) => {
                    self.process_row_children(
                        children,
                        hlir,
                        layouts,
                        current_y,
                        content_width,
                        content_x,
                    );

                    *current_y += element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::Section { children, .. } => {
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
                    *current_y += 10.0 + element_box.padding.bottom + element_box.margin.bottom;
                }
                HirElementOp::Table { .. } => {
                    // TODO this is wrong fix in the future, should adjust cursor based on table content
                    *current_y += 10.0 + element_box.padding.bottom + element_box.margin.bottom;
                }
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
    ) {
        let Some((&right_child, left_children)) = children.split_last() else {
            return;
        };

        let gap = 12.0;
        let right_width = self
            .text_measurement(right_child, hlir)
            .map(|(text, font_size)| Self::estimate_text_width(text, font_size))
            .unwrap_or(0.0)
            .min((content_width - gap).max(0.0));
        let right_x = content_x + (content_width - right_width).max(0.0);
        let left_width = (right_x - content_x - gap).max(0.0);

        let mut row_height: f32 = 0.0;
        for child_idx in left_children {
            row_height = row_height.max(self.push_text_layout(
                *child_idx,
                hlir,
                layouts,
                content_x,
                *current_y,
                left_width,
            ));
        }

        row_height = row_height.max(self.push_text_layout(
            right_child,
            hlir,
            layouts,
            right_x,
            *current_y,
            right_width,
        ));

        *current_y += row_height;
    }

    fn push_text_layout(
        &self,
        element_index: usize,
        hlir: &HIRModule,
        layouts: &mut Vec<ComputedLayout>,
        x: f32,
        y: f32,
        width: f32,
    ) -> f32 {
        let Some(HirElementOp::Text { content, .. }) = hlir.elements.get(element_index) else {
            return 0.0;
        };

        let attrs = hlir
            .attributes
            .find_node(Self::element_attributes_ref(&hlir.elements[element_index]))
            .map(|node| &node.computed);
        let font_size = attrs
            .and_then(Self::parse_font_size)
            .unwrap_or(DEFAULT_FONT_SIZE_PT);
        let line_height = attrs
            .and_then(|attrs| Self::parse_line_height(attrs, font_size))
            .unwrap_or(font_size * DEFAULT_LINE_HEIGHT_MULTIPLIER);
        let line_count = Self::wrap_text(content, width, font_size).len().max(1) as f32;
        let height = line_count * line_height;

        layouts.push(ComputedLayout {
            x,
            y,
            width,
            height,
            element_index,
            marker: None,
        });

        height
    }

    fn text_measurement<'a>(
        &self,
        element_index: usize,
        hlir: &'a HIRModule,
    ) -> Option<(&'a str, f32)> {
        let element = hlir.elements.get(element_index)?;
        let HirElementOp::Text { content, .. } = element else {
            return None;
        };

        let attrs = hlir
            .attributes
            .find_node(Self::element_attributes_ref(element))
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

    fn element_attributes_ref(element: &HirElementOp) -> usize {
        match element {
            HirElementOp::Section { attributes, .. }
            | HirElementOp::List { attributes, .. }
            | HirElementOp::Text { attributes, .. }
            | HirElementOp::Image { attributes, .. }
            | HirElementOp::Table { attributes, .. } => *attributes,
        }
    }

    fn parse_font_size(attributes: &StyleAttributes) -> Option<f32> {
        attributes
            .style
            .get("font-size")
            .and_then(|value| Self::parse_css_length(value))
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

        Self::parse_css_length(value)
    }

    pub(crate) fn wrap_text(content: &str, max_width: f32, font_size: f32) -> Vec<String> {
        if content.is_empty() {
            return vec![String::new()];
        }

        if max_width <= 0.0 {
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
            element_index,
            marker: None,
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
            element_index,
            marker: None,
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
                    element_index: idx,
                    marker: None,
                })
            })
    }
}
