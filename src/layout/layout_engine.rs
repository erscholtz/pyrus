use std::collections::HashMap;
use taffy::style::{AvailableSpace, Dimension};
use taffy::style_helpers::{FromLength, FromPercent, TaffyAuto};
use taffy::{LengthPercentage, LengthPercentageAuto, NodeId, Rect, Size, Style, TaffyTree};

use crate::hir::{FuncId, HIRModule, HirElementOp, Id, Op, StyleAttributes};

pub fn setup_layout(hlir_module: &HIRModule) -> LayoutEngine {
    let layout = LayoutEngine::build_from_hlir_module(hlir_module);
    layout
}

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
            .get(&Id::Func(document_id))
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
        let mut current_y = 0.0;
        let page_width = 595.0; // A4 width in points

        // Get document ops in order and recursively process all elements
        let document_id = FuncId(hlir.functions.len() - 1);
        if let Some(document) = hlir.functions.get(&Id::Func(document_id)) {
            for op in &document.body.ops {
                match op {
                    Op::HirElementEmit { index } => {
                        self.process_element_for_layout(
                            *index,
                            hlir,
                            &mut layouts,
                            &mut current_y,
                            page_width,
                            0.0, // Initial x offset
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
                                    0.0,
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
    ) {
        if let Some(element) = hlir.elements.get(element_index) {
            let (height, has_children) = match element {
                HirElementOp::Text { attributes, .. } => {
                    let attrs = hlir.attributes.find_node(*attributes).map(|n| &n.computed);
                    let font_size = attrs
                        .and_then(|a| a.style.get("font-size"))
                        .and_then(|v| Self::parse_css_length(v))
                        .unwrap_or(12.0);
                    // Line height is typically 1.2x font size
                    (font_size * 1.2, false)
                }
                HirElementOp::List { children, .. } => {
                    // List container - process children with indentation
                    for child_idx in children {
                        self.process_element_for_layout(
                            *child_idx,
                            hlir,
                            layouts,
                            current_y,
                            page_width - 20.0, // Slightly narrower for list items
                            x_offset + 20.0,   // Indent list items
                        );
                    }
                    (0.0, true) // Height comes from children
                }
                HirElementOp::Section { children, .. } => {
                    // Section container - process children
                    for child_idx in children {
                        self.process_element_for_layout(
                            *child_idx, hlir, layouts, current_y, page_width, x_offset,
                        );
                    }
                    (0.0, true) // Height comes from children
                }
                HirElementOp::Image { .. } => {
                    // Image - no layout needed, but we still need to advance the cursor
                    *current_y += 10.0;
                    (0.0, false)
                }
                HirElementOp::Table { .. } => {
                    // TODO this is wrong fix in the future, should adjust cursor based on table content
                    *current_y += 10.0;
                    (0.0, false)
                }
            };

            // Only add layout for leaf elements (text) or if the element has its own height
            if !has_children {
                layouts.push(ComputedLayout {
                    x: x_offset,
                    y: *current_y,
                    width: page_width,
                    height,
                    element_index,
                });

                *current_y += height;
            }
        }
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
                })
            })
    }
}
