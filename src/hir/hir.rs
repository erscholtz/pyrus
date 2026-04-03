use std::collections::HashMap;

use crate::ast::{Ast, Expression, Statement, StatementKind};
use crate::error::{Diagnostic, Severity, SourceLocation, Span};
use crate::hir::hir_util::hir_error::HirError;
pub use crate::hir::hir_util::style_resolver::resolve_styles;
pub use crate::hir::ir_types::{
    AttributeNode, AttributeTree, ElementMetadata, FuncBlock, FuncDecl, FuncId, GlobalId,
    HIRModule, HirElementOp, Id, Literal, Op, StyleAttributes, Type,
};
use crate::util::Spanned;

pub fn lower(ast: &Ast) -> HIRModule {
    let mut pass = HIRPass {
        ast: ast.clone(),
        symbol_table: Vec::new(),
    };
    pass.lower()
}

pub struct HIRPass {
    // Fields and methods for the Hir struct
    pub ast: Ast,
    pub symbol_table: Vec<HashMap<String, Id>>, // Scope stack
}

impl HIRPass {
    // Methods for the Hir struct
    fn lower(&mut self) -> HIRModule {
        let mut hirmodule = HIRModule {
            file: self.ast.file.clone(),
            globals: HashMap::new(),
            functions: HashMap::new(),
            attributes: AttributeTree::new(),
            css_rules: Vec::new(),
            elements: Vec::new(),
            element_metadata: Vec::new(),
            errors: Vec::new(),
        };

        self.symbol_table.push(HashMap::new()); // add new scope (global)

        self.lower_template_block(&mut hirmodule);
        self.lower_document_block(&mut hirmodule);
        // Store CSS rules from AST
        if let Some(style) = &self.ast.style {
            hirmodule.css_rules = style.statements.clone();
        }

        self.symbol_table.pop(); // remove scope (global)

        hirmodule
    }

    fn lower_template_block(&mut self, hirmodule: &mut HIRModule) {
        // all global, default and function declarations
        // handle defaults and globals inside this function call since they are small

        let Some(template) = &self.ast.template else {
            return;
        };

        let statements = template.statements.clone();
        for statement in &statements {
            match &statement.node {
                StatementKind::DefaultSet { key, value } => {
                    let global_id = Id::Global(GlobalId(hirmodule.globals.len()));
                    let global_name = "__".to_string() + &key.clone();
                    let global = self.assign_global(&global_name, &value, global_id, false);
                    hirmodule.globals.insert(global_id, global);
                    self.add_symbol(key.clone(), global_id);
                }
                StatementKind::ConstAssign { name, value } => {
                    let global_id = Id::Global(GlobalId(hirmodule.globals.len()));
                    let global = self.assign_global(&name, &value, global_id, false);
                    hirmodule.globals.insert(global_id, global);
                    self.add_symbol(name.clone(), global_id);
                }
                StatementKind::VarAssign { name, value } => {
                    let global_id = Id::Global(GlobalId(hirmodule.globals.len()));
                    let global = self.assign_global(&name, &value, global_id, true);
                    hirmodule.globals.insert(global_id, global);
                    self.add_symbol(name.clone(), global_id);
                }
                StatementKind::FunctionDecl {
                    name,
                    args,
                    body,
                    return_type,
                } => {
                    let func_id = FuncId(hirmodule.functions.len());
                    let hir_body = self.lower_function_block(body, hirmodule);
                    self.add_symbol(name.clone(), Id::Func(func_id)); // adds function name to symbol table
                    let mut arg_list = Vec::new();
                    for arg in args {
                        match arg.ty.as_str() {
                            "Int" => arg_list.push(Type::Int),
                            "Float" => arg_list.push(Type::Float),
                            "String" => arg_list.push(Type::String),
                            _ => panic!("type not known"),
                        }
                    }

                    let return_type = match return_type {
                        Some(t) => match t.as_str() {
                            "Int" => Some(Type::Int),
                            "Float" => Some(Type::Float),
                            "String" => Some(Type::String),
                            "DocElement" => Some(Type::DocElement),
                            _ => panic!("type not known"),
                        },
                        None => None,
                    };

                    hirmodule.functions.insert(
                        Id::Func(func_id),
                        FuncDecl {
                            id: Id::Func(func_id),
                            name: name.clone(),
                            args: arg_list,
                            return_type: return_type,
                            body: hir_body,
                        },
                    );
                }
                StatementKind::ElementDecl { name, args, body } => {
                    let element_id = hirmodule.elements.len();
                    let location = statement.location.clone();

                    hirmodule.element_metadata.push(ElementMetadata {
                        id: None,
                        classes: Vec::new(),
                        element_type: name.clone(),
                        parent: None,
                        attributes_ref: 0,
                        location: location,
                    });
                }
                _ => {}
            }
        }
    }

    fn lower_document_block(&mut self, hirmodule: &mut HIRModule) {
        let mut ir_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        };

        self.symbol_table.push(HashMap::new()); // add new scope (document)

        if let Some(document) = &self.ast.document {
            let elements = document.elements.clone();
            for element in &elements {
                let index = self.lower_document_element(element, hirmodule, &mut ir_body, None);

                // Only emit HirElementEmit for actual elements, not for function calls
                // Calls handle element emission separately via Op::Call
                if !matches!(element.node, crate::ast::DocElementKind::Call { .. }) {
                    ir_body.ops.push(Op::HirElementEmit { index });
                }
            }
        }
        let func_id = FuncId(TryInto::<usize>::try_into(hirmodule.functions.len()).unwrap());
        hirmodule.functions.insert(
            Id::Func(func_id),
            FuncDecl {
                id: Id::Func(func_id),
                name: "__document".to_string(),
                args: Vec::new(),
                return_type: Some(Type::DocElement), // For right now only DocElements are supported TODO add in other types support later
                body: ir_body,
            },
        );

        self.symbol_table.pop(); // remove scope (document)
    }

    pub fn lower_document_element(
        &mut self,
        element: &crate::ast::DocElement,
        hirmodule: &mut HIRModule,
        ir_body: &mut FuncBlock,
        parent_index: Option<usize>,
    ) -> usize {
        let location = element.location.clone();
        match &element.node {
            crate::ast::DocElementKind::Call {
                name,
                args,
                children,
            } => {
                let func_id = match self.find_symbol(name.as_str()) {
                    Some(id) => Some(id),
                    None => None,
                };
                if func_id.is_none() {
                    hirmodule.errors.push(HirError::new(
                        format!("Function not found: {}", name),
                        Severity::Error,
                        location.clone(),
                        Span {
                            start: location.line,
                            end: location.column,
                            file: location.file.clone(),
                        },
                    ));
                    return 0;
                }

                let arg_value_ids = self.handle_args(&args, ir_body);
                ir_body.ops.push(Op::Call {
                    func: func_id.unwrap(),
                    result: None,
                    args: arg_value_ids,
                });
                // Call ops don't need to return an index - they handle element emission separately
                // The returned_element_ref in the function body is used instead
                0 // TODO magic number
            }
            crate::ast::DocElementKind::Text {
                content,
                attributes,
            } => {
                let (id, classes) = self.extract_id_and_classes(&attributes);
                let element_type = "text".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(&attributes, hirmodule.attributes.size);
                let attributes_ref = hirmodule.attributes.add_attribute(attribute_node);
                let index = hirmodule.elements.len();
                hirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                    location,
                });
                hirmodule.elements.push(HirElementOp::Text {
                    content: content.to_string(),
                    attributes: attributes_ref,
                });

                index
            }
            crate::ast::DocElementKind::Section {
                elements: section_elements,
                attributes,
            } => {
                let (id, classes) = self.extract_id_and_classes(&attributes);
                let element_type = "section".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(&attributes, hirmodule.attributes.size);
                let attributes_ref = hirmodule.attributes.add_attribute(attribute_node);
                // Reserve index before processing children so children get correct parent
                let index = hirmodule.elements.len();
                hirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                    location,
                });
                // Push placeholder first to reserve the slot
                hirmodule.elements.push(HirElementOp::Section {
                    children: Vec::new(), // Will be updated
                    attributes: attributes_ref,
                });
                let mut children = Vec::new();
                for child in section_elements {
                    children.push(self.lower_document_element(
                        child,
                        hirmodule,
                        ir_body,
                        Some(index),
                    ));
                }
                // Update with actual children
                hirmodule.elements[index] = HirElementOp::Section {
                    children,
                    attributes: attributes_ref,
                };

                index
            }
            crate::ast::DocElementKind::List {
                items,
                attributes,
                numbered,
            } => {
                let (id, classes) = self.extract_id_and_classes(&attributes);
                let element_type = "list".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(&attributes, hirmodule.attributes.size);
                let attributes_ref = hirmodule.attributes.add_attribute(attribute_node);
                // Reserve index before processing children so children get correct parent
                let index = hirmodule.elements.len();
                hirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    location,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                });
                // Push placeholder first to reserve the slot
                hirmodule.elements.push(HirElementOp::List {
                    children: Vec::new(), // Will be updated
                    attributes: attributes_ref,
                });
                let mut children = Vec::new();
                for child in items {
                    children.push(self.lower_document_element(
                        child,
                        hirmodule,
                        ir_body,
                        Some(index),
                    ));
                }
                // Update with actual children
                hirmodule.elements[index] = HirElementOp::List {
                    children,
                    attributes: attributes_ref,
                };

                index
            }
            // TODO: Handle Image, Code, Link, Table similarly
            crate::ast::DocElementKind::Image { src, attributes } => {
                let (id, classes) = self.extract_id_and_classes(&attributes);
                let element_type = "list".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(&attributes, hirmodule.attributes.size);
                let attributes_ref = hirmodule.attributes.add_attribute(attribute_node);
                // Reserve index before processing children so children get correct parent
                let index = hirmodule.elements.len();
                hirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                    location,
                });
                hirmodule.elements.push(HirElementOp::Image {
                    src: src.to_string(),
                    attributes: attributes_ref,
                });

                index
            }
            crate::ast::DocElementKind::Table { table, attributes } => {
                let (id, classes) = self.extract_id_and_classes(&attributes);
                let element_type = "list".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(&attributes, hirmodule.attributes.size);
                let attributes_ref = hirmodule.attributes.add_attribute(attribute_node);
                // Reserve index before processing children so children get correct parent
                let index = hirmodule.elements.len();
                hirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                    location,
                });
                let table = table
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|cell| {
                                self.lower_document_element(&cell, hirmodule, ir_body, Some(index))
                            })
                            .collect()
                    })
                    .collect();
                hirmodule.elements.push(HirElementOp::Table {
                    table,
                    attributes: attributes_ref,
                });

                index
            }
            _ => {
                let span = Span::point(0, "unknown");
                let err = HirError::new(
                    format!(
                        "Unsupported document element: {:?}  (HIR document lowering)",
                        element.node
                    ),
                    Severity::Error,
                    location,
                    span,
                );
                hirmodule.errors.push(err);
                // return invalid index
                usize::MAX
            }
        }
    }

    /// Extract id and classes from element attributes
    fn extract_id_and_classes(
        &self,
        attributes: &HashMap<String, Expression>,
    ) -> (Option<String>, Vec<String>) {
        let id = attributes.get("id").map(|e| e.to_string());

        let classes = attributes
            .get("class")
            .map(|e| e.to_string().split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        (id, classes)
    }

    pub fn add_symbol(&mut self, name: String, id: Id) {
        for scope in self.symbol_table.iter_mut().rev() {
            if let Some(_symbol) = scope.get(&name) {
                // TODO check if the the id types match (Func/value/global), if there is a function defined with the same name as a variable then it should be ok or vice versa
                panic!("Symbol {} already exists", name);
            }
        }
        let len = self.symbol_table.len();
        let scope = self.symbol_table.get_mut(len - 1).unwrap(); // most recent scope
        scope.insert(name.clone(), id); // add to known symbols
    }

    fn find_symbol(&mut self, name: &str) -> Option<Id> {
        for scope in self.symbol_table.iter_mut().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(*symbol);
            }
        }
        None
    }
}
