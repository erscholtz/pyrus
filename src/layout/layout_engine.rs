use std::collections::HashMap;
use taffy::style::AvailableSpace;
use taffy::{LengthPercentage, LengthPercentageAuto, NodeId, Rect, Size, Style, TaffyTree};

use crate::hlir::{FuncId, HLIRModule, HlirElement, Id, Op, StyleAttributes};

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
            Op::HlirElementEmit { index } => {
                // Document element - has metadata and computed styles
                let element = hlir_module.elements.get(*index).expect("element not found");
                let attributes_ref = match element {
                    HlirElement::Section { attributes, .. } => *attributes,
                    HlirElement::List { attributes, .. } => *attributes,
                    HlirElement::Text { attributes, .. } => *attributes,
                };
                self.create_node_from_metadata(*index, attributes_ref, hlir_module, parent_node);
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

        // Get attributes_ref from the element and look up computed styles
        let attributes_ref = match element {
            HlirElement::Section { attributes, .. } => *attributes,
            HlirElement::List { attributes, .. } => *attributes,
            HlirElement::Text { attributes, .. } => *attributes,
        };

        let attributes = match hlir_module.attributes.find_node(attributes_ref) {
            Some(node) => &node.computed,
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

        // Handle children (for Section, List, etc.)
        self.process_element_children(element, hlir_module, node_id);
    }

    fn process_element_children(
        &mut self,
        element: &HlirElement,
        hlir_module: &HLIRModule,
        parent_node: NodeId,
    ) {
        // HlirElement stores children as indices into hlir_module.elements
        let children = match element {
            HlirElement::Section { children, .. } => children,
            HlirElement::List { children, .. } => children,
            HlirElement::Text { .. } => return, // No children
        };

        for child_idx in children {
            self.create_node_from_element(*child_idx, hlir_module, parent_node);
        }
    }

    pub fn attr_to_style(attributes: &StyleAttributes) -> Style {
        // TODO hack, this needs to actually use the style attributes
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
