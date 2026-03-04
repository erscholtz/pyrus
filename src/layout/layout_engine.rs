use std::collections::HashMap;
use taffy::style::AvailableSpace;
use taffy::{LengthPercentage, LengthPercentageAuto, NodeId, Rect, Size, Style, TaffyTree};

use crate::ast::DocElement;
use crate::hlir::{FuncId, HLIRModule, Id, Op, StyleAttributes};

pub fn setup_layout(hlir_module: &HLIRModule) -> LayoutEngine {
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

    pub fn build_from_hlir_module(hlir_module: &HLIRModule) -> Self {
        let mut layout = LayoutEngine::new();

        // Create root node
        layout.root = layout
            .tree
            .new_with_children(Style::default(), &[])
            .unwrap();

        // Pre-allocate element_to_node to match elements size
        layout.element_to_node = vec![None; hlir_module.elements.len()];

        // Get the document function
        let document_id = FuncId(hlir_module.functions.len() - 1);
        let document = hlir_module
            .functions
            .get(&Id::Func(document_id))
            .expect("document function not found");

        // Build layout tree from document ops
        for op in &document.body.ops {
            layout.process_op(op, hlir_module, layout.root);
        }

        layout
    }

    fn process_op(&mut self, op: &Op, hlir_module: &HLIRModule, parent_node: NodeId) {
        match op {
            Op::DocElementEmit {
                index,
                attributes_ref,
            } => {
                // Document element - has metadata and computed styles
                self.create_node_from_metadata(*index, *attributes_ref, hlir_module, parent_node);
            }
            Op::Call { func, .. } => {
                // Template function call - element is in function's returned_element_ref
                if let Some(function) = hlir_module.functions.get(func) {
                    if let Some(element_id) = function.body.returned_element_ref {
                        // Template element - extract styles directly from DocElement
                        self.create_node_from_element(element_id, hlir_module, parent_node);
                    }
                }
            }
            _ => {}
        }
    }

    fn create_node_from_metadata(
        &mut self,
        element_index: usize,
        attributes_ref: usize,
        hlir_module: &HLIRModule,
        parent_node: NodeId,
    ) {
        let attributes = match hlir_module.attributes.find_node(attributes_ref) {
            Some(a) => &a.computed,
            None => return,
        };

        let style = Self::attr_to_style(attributes);
        let node_id = match self.tree.new_leaf(style) {
            Ok(id) => id,
            Err(_) => return,
        };

        let _ = self.tree.add_child(parent_node, node_id);

        if element_index < self.element_to_node.len() {
            self.element_to_node[element_index] = Some(node_id);
        }

        if let Some(id) = &attributes.id {
            self.id_to_node.insert(id.clone(), node_id);
        }
    }

    fn create_node_from_element(
        &mut self,
        element_index: usize,
        hlir_module: &HLIRModule,
        parent_node: NodeId,
    ) {
        let element = match hlir_module.elements.get(element_index) {
            Some(e) => e,
            None => return,
        };

        // Extract attributes directly from DocElement (no CSS resolution for templates yet)
        let attributes = extract_attributes_from_doc_element(element);
        let style = Self::attr_to_style(&attributes);

        let node_id = match self.tree.new_leaf(style) {
            Ok(id) => id,
            Err(_) => return,
        };

        let _ = self.tree.add_child(parent_node, node_id);

        if element_index < self.element_to_node.len() {
            self.element_to_node[element_index] = Some(node_id);
        }

        if let Some(id) = &attributes.id {
            self.id_to_node.insert(id.clone(), node_id);
        }

        // Handle children (for Section, List, etc.)
        self.process_element_children(element, hlir_module, node_id);
    }

    fn process_element_children(
        &mut self,
        element: &DocElement,
        hlir_module: &HLIRModule,
        parent_node: NodeId,
    ) {
        // For template elements, children are in the DocElement itself
        // But we don't have ops for them - they're already in elements vector
        // For now, just handle sections and lists
        match element {
            DocElement::Section { elements, .. }
            | DocElement::List {
                items: elements, ..
            } => {
                for child in elements {
                    // Find this child's index in hlir_module.elements
                    // This is O(n) but works for now
                    if let Some(idx) = find_element_index(hlir_module, child) {
                        self.create_node_from_element(idx, hlir_module, parent_node);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn attr_to_style(attributes: &StyleAttributes) -> Style {
        let margin_zero = LengthPercentageAuto::length(0.0);
        let padding_zero = LengthPercentage::length(0.0);

        let margin = attributes
            .margin
            .map_or(margin_zero, |m| LengthPercentageAuto::length(m));
        let padding = attributes
            .padding
            .map_or(padding_zero, |p| LengthPercentage::length(p));

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
            ..Style::default()
        }
    }

    /// Run Taffy layout computation with given available space
    pub fn compute_layout(&mut self, available_width: f32, available_height: f32) {
        let size = Size {
            width: AvailableSpace::Definite(available_width),
            height: AvailableSpace::Definite(available_height),
        };
        self.tree.compute_layout(self.root, size).unwrap();
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

/// Extract StyleAttributes from a DocElement's inline attributes
fn extract_attributes_from_doc_element(element: &DocElement) -> StyleAttributes {
    use crate::ast::Expression;

    let attrs = match element {
        DocElement::Text { attributes, .. } => attributes,
        DocElement::Section { attributes, .. } => attributes,
        DocElement::List { attributes, .. } => attributes,
        DocElement::Image { attributes, .. } => attributes,
        DocElement::Table { attributes, .. } => attributes,
        DocElement::Code { attributes, .. } => attributes,
        DocElement::Link { attributes, .. } => attributes,
        DocElement::Call { .. } => return StyleAttributes::default(),
    };

    let mut result = StyleAttributes::default();

    // Extract id
    if let Some(Expression::StringLiteral(id)) = attrs.get("id") {
        result.id = Some(id.clone());
    }

    // Extract class
    if let Some(Expression::StringLiteral(class)) = attrs.get("class") {
        result.class = class.split_whitespace().map(String::from).collect();
    }

    // Extract margin
    if let Some(expr) = attrs.get("margin") {
        result.margin = expr.to_string().parse().ok();
    }

    // Extract padding
    if let Some(expr) = attrs.get("padding") {
        result.padding = expr.to_string().parse().ok();
    }

    // Extract other style properties into the style map
    for (key, value) in attrs {
        if !matches!(key.as_str(), "id" | "class" | "margin" | "padding") {
            result.style.insert(key.clone(), value.to_string());
        }
    }

    result
}

/// Find the index of an element in hlir_module.elements by comparing content
fn find_element_index(hlir_module: &HLIRModule, target: &DocElement) -> Option<usize> {
    // This is a hack - we compare by debug representation
    // In a proper implementation, we'd have stable IDs
    let target_dbg = format!("{:?}", target);
    hlir_module
        .elements
        .iter()
        .position(|e| format!("{:?}", e) == target_dbg)
}
