use std::collections::HashMap;
use std::str::FromStr;

use crate::ast::{DocElement, Expression, StyleRule};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Color,
    DocElement,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(String),
}

// IDs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub usize);

use std::fmt;

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Id {
    Func(FuncId),
    Global(GlobalId),
    Value(ValueId),
}

#[derive(Debug, Clone)]
pub enum Op {
    Const {
        result: Id,
        literal: Literal,
        ty: Type,
    },
    Var {
        result: Id,
        name: String,
        ty: Type,
    },
    Binary {
        result: Id,
        op: BinOp,
        lhs: Id,
        rhs: Id,
    },
    Call {
        result: Option<Id>,
        func: Id,
        args: Vec<Id>,
    },
    Return {
        doc_element_ref: usize,
    },
    HlirElementEmit {
        index: usize,
    },
    StringConcat {
        result: Id,
        parts: Vec<Id>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

// how globals + Variables are handled

#[derive(Debug, Clone)]
pub struct Global {
    pub id: Id,
    pub name: String,
    pub ty: Type,
    pub init: Literal,
    pub mutable: bool,
}

pub struct Local {
    // NOTE: potential use once template section has some more use
    pub id: Id,
    pub ty: Type,
}

// how functions are handled

#[derive(Debug, Clone)]
pub struct Func {
    pub id: Id,
    pub name: String,
    pub args: Vec<Type>,
    pub return_type: Option<Type>,
    pub body: FuncBlock,
}

#[derive(Debug, Clone)]
pub struct FuncBlock {
    pub ops: Vec<Op>,
    pub returned_element_ref: Option<usize>,
}

// how template, document and style sections are handled

#[derive(Debug, Clone)]
pub struct Block {
    pub ops: Vec<Op>,
    pub element_refs: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ElementMetadata {
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub element_type: String,
    pub parent: Option<usize>, // Index into elements vector
    pub attributes_ref: usize, // Index into AttributeTree
}

#[derive(Debug, Clone)]
pub struct HIRModule {
    pub globals: HashMap<Id, Global>, // TODO eventually remove IDs from actual struct and just refer to them (I think)
    pub functions: HashMap<Id, Func>,
    pub attributes: AttributeTree,
    pub css_rules: Vec<StyleRule>, // Parsed CSS rules (unapplied)
    pub elements: Vec<HirElement>,
    pub element_metadata: Vec<ElementMetadata>, // Parallel to elements, for CSS matching
}

#[derive(Debug, Clone)]
pub enum HirElement {
    Section {
        children: Vec<usize>,
        attributes: usize,
    },
    List {
        children: Vec<usize>,
        attributes: usize,
    },
    Text {
        content: String,
        attributes: usize,
    },
    // TODO code, images, etc
}

#[derive(Debug, Clone)]
pub struct AttributeTree {
    pub root: AttributeNode,
    pub size: usize,
}

impl AttributeTree {
    // TODO, will need to rethink this ID stuff at some point but working for now
    pub fn new() -> Self {
        Self {
            root: AttributeNode::new(),
            size: 1,
        }
    }

    pub fn add_attribute(&mut self, attributes: AttributeNode) -> usize {
        let id = self.size;
        self.size += 1;
        self.root.add_child(attributes, id)
    }

    pub fn find_node(&self, id: usize) -> Option<&AttributeNode> {
        self.root.find_node_recursive(id)
    }

    pub fn find_node_mut(&mut self, id: usize) -> Option<&mut AttributeNode> {
        self.root.find_node_mut_recursive(id)
    }
}

#[derive(Debug, Clone)]
pub struct AttributeNode {
    pub parent: Option<usize>, // Pointer to parent AttributeNode
    pub id: usize,
    pub inline: StyleAttributes, // Inline styles from element attributes
    pub computed: StyleAttributes, // Final computed styles after CSS resolution
    pub children: HashMap<usize, AttributeNode>,
}

impl AttributeNode {
    pub fn new() -> Self {
        Self {
            parent: None,
            id: 1,
            inline: StyleAttributes::default(),
            computed: StyleAttributes::default(),
            children: HashMap::new(),
        }
    }

    pub fn new_with_attributes(attributes: &HashMap<String, Expression>, parent_id: usize) -> Self {
        Self {
            parent: Some(parent_id),
            id: parent_id + 1,
            inline: StyleAttributes::new_with_attributes(attributes),
            computed: StyleAttributes::default(),
            children: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, child: AttributeNode, parent_id: usize) -> usize {
        let id = parent_id + 1;
        self.children.insert(id, child);
        id
    }

    fn find_node_recursive(&self, target_id: usize) -> Option<&AttributeNode> {
        if self.id == target_id {
            return Some(self);
        }
        for child in self.children.values() {
            if let Some(found) = child.find_node_recursive(target_id) {
                return Some(found);
            }
        }
        None
    }

    fn find_node_mut_recursive(&mut self, target_id: usize) -> Option<&mut AttributeNode> {
        if self.id == target_id {
            return Some(self);
        }
        for child in self.children.values_mut() {
            if let Some(found) = child.find_node_mut_recursive(target_id) {
                return Some(found);
            }
        }
        None
    }

    pub fn is_inherited_property(property: &str) -> bool {
        matches!(
            property,
            "color"
                | "font-family"
                | "font-size"
                | "font-weight"
                | "font-style"
                | "line-height"
                | "text-align"
                | "visibility"
        )
    }

    pub fn get_effective_value(&self, property: &str, tree: &AttributeTree) -> Option<String> {
        if let Some(val) = self.computed.get(property) {
            return Some(val);
        }

        if Self::is_inherited_property(property) {
            if let Some(parent_id) = self.parent {
                if let Some(parent_node) = tree.find_node(parent_id) {
                    return parent_node.get_effective_value(property, tree);
                }
            }
        }

        None
    }
}

// ----------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Align {
    Left,
    Center,
    Right,
}

impl FromStr for Align {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Align::Left),
            "center" => Ok(Align::Center),
            "right" => Ok(Align::Right),
            _ => Err(format!("Invalid alignment value: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageBreak {
    Before,
    After,
    None,
}

impl FromStr for PageBreak {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "before" => Ok(PageBreak::Before),
            "after" => Ok(PageBreak::After),
            "none" => Ok(PageBreak::None),
            _ => Err(format!("Invalid page break value: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StyleAttributes {
    pub id: Option<String>,
    pub class: Vec<String>,
    pub style: HashMap<String, String>,

    pub margin: Option<f32>,
    pub padding: Option<f32>,
    pub align: Option<Align>,

    pub hidden: bool,
    pub condition: Option<bool>, // corresponds to `if=...`

    pub page_break: PageBreak,

    pub role: Option<String>,
}

impl Default for StyleAttributes {
    fn default() -> Self {
        Self {
            id: None,
            class: Vec::new(),
            style: HashMap::new(),

            margin: None,
            padding: None,
            align: None,

            hidden: false,
            condition: None,

            page_break: PageBreak::None,

            role: None,
        }
    }
}

impl StyleAttributes {
    pub fn get(&self, property: &str) -> Option<String> {
        if let Some(val) = self.style.get(property) {
            return Some(val.clone());
        }

        match property {
            "id" => self.id.clone(),
            "margin" => self.margin.map(|v| v.to_string()),
            "padding" => self.padding.map(|v| v.to_string()),
            "align" => self.align.as_ref().map(|v| format!("{:?}", v)),
            "hidden" => Some(self.hidden.to_string()),
            "page_break" => Some(format!("{:?}", self.page_break).to_lowercase()),
            "role" => self.role.clone(),
            _ => None,
        }
    }

    pub fn set(&mut self, property: &str, value: String) {
        match property {
            "id" => self.id = Some(value),
            "margin" => {
                if let Some(v) = Self::parse_css_length(&value) {
                    self.margin = Some(v);
                }
            }
            "padding" => {
                if let Some(v) = Self::parse_css_length(&value) {
                    self.padding = Some(v);
                }
            }
            "align" => {
                if let Ok(v) = value.parse::<Align>() {
                    self.align = Some(v);
                }
            }
            "hidden" => {
                self.hidden = value.parse().unwrap_or(false);
            }
            "page_break" => {
                self.page_break = value.parse().unwrap_or(PageBreak::None);
            }
            "role" => self.role = Some(value),
            _ => {
                // Unknown property goes into the style map
                self.style.insert(property.to_string(), value);
            }
        }
    }

    pub fn merge(&mut self, other: &StyleAttributes) {
        // Merge style map
        for (key, val) in &other.style {
            if !self.style.contains_key(key) {
                self.style.insert(key.clone(), val.clone());
            }
        }

        // Merge known properties (only if not already set)
        if self.id.is_none() && other.id.is_some() {
            self.id = other.id.clone();
        }
        if self.class.is_empty() && !other.class.is_empty() {
            self.class = other.class.clone();
        }
        if self.margin.is_none() {
            self.margin = other.margin;
        }
        if self.padding.is_none() {
            self.padding = other.padding;
        }
        if self.align.is_none() {
            self.align = other.align.clone();
        }
        if self.role.is_none() {
            self.role = other.role.clone();
        }
        // boolean flags: use OR semantics
        self.hidden = self.hidden || other.hidden;
        // page_break: 'Before' and 'After' override 'None'
        if self.page_break == PageBreak::None {
            self.page_break = other.page_break.clone();
        }
    }

    pub fn apply_inherited(&mut self, parent: &StyleAttributes) {
        if self.style.get("font-family").is_none() {
            if let Some(val) = parent.style.get("font-family") {
                self.style.insert("font-family".to_string(), val.clone());
            }
        }
        if self.style.get("font-size").is_none() {
            if let Some(val) = parent.style.get("font-size") {
                self.style.insert("font-size".to_string(), val.clone());
            }
        }
        if self.style.get("font-weight").is_none() {
            if let Some(val) = parent.style.get("font-weight") {
                self.style.insert("font-weight".to_string(), val.clone());
            }
        }
        if self.style.get("color").is_none() {
            if let Some(val) = parent.style.get("color") {
                self.style.insert("color".to_string(), val.clone());
            }
        }
        if self.style.get("line-height").is_none() {
            if let Some(val) = parent.style.get("line-height") {
                self.style.insert("line-height".to_string(), val.clone());
            }
        }
        if self.align.is_none() {
            self.align = parent.align.clone();
        }
    }
}
impl StyleAttributes {
    pub fn new_with_attributes(attributes: &HashMap<String, Expression>) -> Self {
        let mut result = Self::default();

        if let Some(expr) = attributes.get("id") {
            result.id = Some(expr.to_string());
        }

        if let Some(expr) = attributes.get("class") {
            result.class = expr
                .to_string()
                .split_whitespace()
                .map(String::from)
                .collect();
        }

        if let Some(expr) = attributes.get("style") {
            result.style = Self::parse_style(&expr.to_string());
        }

        if let Some(expr) = attributes.get("margin") {
            result.margin = expr.to_string().parse().ok();
        }

        if let Some(expr) = attributes.get("padding") {
            result.padding = expr.to_string().parse().ok();
        }

        if let Some(expr) = attributes.get("align") {
            result.align = expr.to_string().parse().ok();
        }

        if let Some(expr) = attributes.get("hidden") {
            result.hidden = expr.to_string().parse().unwrap_or(false);
        }

        if let Some(expr) = attributes.get("condition") {
            result.condition = expr.to_string().parse().ok();
        }

        if let Some(expr) = attributes.get("page_break") {
            result.page_break = expr.to_string().parse().unwrap_or(PageBreak::None);
        }

        if let Some(expr) = attributes.get("role") {
            result.role = Some(expr.to_string());
        }

        result
    }

    fn parse_style(input: &str) -> HashMap<String, String> {
        input
            .split(';')
            .filter_map(|decl| {
                let (key, value) = decl.split_once(':')?;
                Some((key.trim().to_string(), value.trim().to_string()))
            })
            .collect()
    }

    fn parse_css_length(value: &str) -> Option<f32> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }

        let num_end = value
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(value.len());

        let num_str = &value[..num_end];
        let unit_str = &value[num_end..].trim().to_lowercase();

        let num: f32 = num_str.parse().ok()?;

        match unit_str.as_str() {
            "pt" => Some(num),
            "px" => Some(num * 0.75),    // 1px = 0.75pt
            "mm" => Some(num * 2.83465), // 1mm = 2.83465pt
            "cm" => Some(num * 28.3465), // 1cm = 28.3465pt
            "in" => Some(num * 72.0),    // 1in = 72pt
            "" => Some(num),             // No unit, assume points
            _ => {
                // Unknown unit, try to parse as number anyway
                num_str.parse().ok()
            }
        }
    }
}

// ----------------------------------------------------------------------------------
