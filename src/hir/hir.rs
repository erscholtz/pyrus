use std::collections::HashMap;

use crate::ast::{Ast, Expression, Statement, StatementKind};
use crate::diagnostic::{Diagnostic, Severity, SourceLocation, Span};
use crate::hir::hir_util::hir_error::HirError;
pub use crate::hir::hir_util::style_resolver::resolve_styles;
pub use crate::hir::ir_types::{
    AttributeNode, AttributeTree, ElementId, ElementMetadata, FuncBlock, FuncDecl, FuncId,
    GlobalId, HIRModule, HirElementDecl, HirElementOp, Id, Literal, Op, StyleAttributes, Type,
    ValueId,
};

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
            element_decls: HashMap::new(),
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
                        arg_list.push(self.parse_type(&arg.ty).unwrap());
                    }
                    let return_type = self.parse_type(&return_type.as_deref().unwrap_or(""));

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
                    let element_decl_id = ElementId(hirmodule.element_decls.len());
                    let id = Id::Element(element_decl_id);
                    let location = statement.location.clone();
                    self.add_symbol(name.clone(), id);
                    let mut arg_list = Vec::new();
                    for arg in args {
                        arg_list.push(self.parse_type(&arg.ty).unwrap());
                    }
                    let hir_body = self.lower_element_body(body, hirmodule);
                    let element_decl = HirElementDecl {
                        id,
                        name: name.clone(),
                        args: arg_list,
                        body: hir_body,
                    };
                    hirmodule.element_decls.insert(id, element_decl);

                    hirmodule.element_metadata.push(ElementMetadata {
                        id: Some(name.clone()),
                        classes: Vec::new(),
                        element_type: name.clone(),
                        parent: None,
                        attributes_ref: 0,
                        location,
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
                let symbol = self.find_symbol(name.as_str());
                if symbol.is_none() {
                    hirmodule.errors.push(HirError::new(
                        format!("Function or element not found: {}", name),
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

                let symbol_id = symbol.unwrap();

                // Check if this is an element declaration or a function
                match symbol_id {
                    Id::Element(_) => {
                        // Build wrapper attributes with element name as id and class
                        let mut wrapper_attrs = std::collections::HashMap::new();
                        wrapper_attrs.insert(
                            "id".to_string(),
                            Expression::new(
                                crate::ast::ExpressionKind::StringLiteral(name.clone()),
                                location.clone(),
                            ),
                        );
                        wrapper_attrs.insert(
                            "class".to_string(),
                            Expression::new(
                                crate::ast::ExpressionKind::StringLiteral(name.clone()),
                                location.clone(),
                            ),
                        );

                        let (wrapper_index, wrapper_attr_ref) = self.reserve_element_slot(
                            hirmodule,
                            "section",
                            &wrapper_attrs,
                            parent_index,
                            location.clone(),
                        );

                        let mut wrapper_children = Vec::new();

                        if !children.is_empty() {
                            // Build children section attributes
                            let mut children_attrs = std::collections::HashMap::new();
                            children_attrs.insert(
                                "class".to_string(),
                                Expression::new(
                                    crate::ast::ExpressionKind::StringLiteral(
                                        "children".to_string(),
                                    ),
                                    location.clone(),
                                ),
                            );

                            let (children_section_index, children_attr_ref) = self
                                .reserve_element_slot(
                                    hirmodule,
                                    "section",
                                    &children_attrs,
                                    Some(wrapper_index),
                                    location.clone(),
                                );

                            // Lower the children into the children section
                            let mut children_indices = Vec::new();
                            for child in children {
                                children_indices.push(self.lower_document_element(
                                    child,
                                    hirmodule,
                                    ir_body,
                                    Some(children_section_index),
                                ));
                            }

                            // Update children section with actual children
                            hirmodule.elements[children_section_index] = HirElementOp::Section {
                                children: children_indices,
                                attributes: children_attr_ref,
                            };

                            wrapper_children.push(children_section_index);
                        }

                        // Handle arguments
                        let arg_value_ids = self.handle_args(&args, ir_body);

                        // Emit ElementCall op
                        let result_id = Id::Value(ValueId(ir_body.ops.len()));
                        ir_body.ops.push(Op::ElementCall {
                            result: result_id,
                            element: symbol_id,
                            args: arg_value_ids,
                        });

                        // Update wrapper section with children section (if any)
                        hirmodule.elements[wrapper_index] = HirElementOp::Section {
                            children: wrapper_children,
                            attributes: wrapper_attr_ref,
                        };

                        wrapper_index
                    }
                    _ => {
                        // Function call: use FuncCall op (original behavior)
                        let arg_value_ids = self.handle_args(&args, ir_body);
                        ir_body.ops.push(Op::FuncCall {
                            func: symbol_id,
                            result: None,
                            args: arg_value_ids,
                        });
                        // Function calls don't return an index - they handle element emission separately
                        0 // TODO magic number
                    }
                }
            }
            crate::ast::DocElementKind::Text {
                content,
                attributes,
            } => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "text",
                    &attributes,
                    parent_index,
                    location,
                );
                hirmodule.elements[index] = HirElementOp::Text {
                    content: content.to_string(),
                    attributes: attributes_ref,
                };
                index
            }
            crate::ast::DocElementKind::Section {
                elements: section_elements,
                attributes,
            } => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "section",
                    &attributes,
                    parent_index,
                    location,
                );
                let mut children = Vec::new();
                for child in section_elements {
                    children.push(self.lower_document_element(
                        child,
                        hirmodule,
                        ir_body,
                        Some(index),
                    ));
                }
                hirmodule.elements[index] = HirElementOp::Section {
                    children,
                    attributes: attributes_ref,
                };
                index
            }
            crate::ast::DocElementKind::List {
                items,
                attributes,
                numbered: _,
            } => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "list",
                    &attributes,
                    parent_index,
                    location,
                );
                let mut children = Vec::new();
                for child in items {
                    children.push(self.lower_document_element(
                        child,
                        hirmodule,
                        ir_body,
                        Some(index),
                    ));
                }
                hirmodule.elements[index] = HirElementOp::List {
                    children,
                    attributes: attributes_ref,
                };
                index
            }
            // TODO: Handle Image, Code, Link, Table similarly
            crate::ast::DocElementKind::Image { src, attributes } => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "image",
                    &attributes,
                    parent_index,
                    location,
                );
                hirmodule.elements[index] = HirElementOp::Image {
                    src: src.to_string(),
                    attributes: attributes_ref,
                };
                index
            }
            crate::ast::DocElementKind::Table { table, attributes } => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "table",
                    &attributes,
                    parent_index,
                    location,
                );
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
                hirmodule.elements[index] = HirElementOp::Table {
                    table,
                    attributes: attributes_ref,
                };
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

    fn reserve_element_slot(
        &mut self,
        hirmodule: &mut HIRModule,
        element_type: &str,
        attributes: &HashMap<String, Expression>,
        parent_index: Option<usize>,
        location: SourceLocation,
    ) -> (usize, usize) {
        // (element_index, attributes_ref)
        let (id, classes) = self.extract_id_and_classes(&attributes);
        let attribute_node =
            AttributeNode::new_with_attributes(&attributes, hirmodule.attributes.size);
        let attributes_ref = hirmodule.attributes.add_attribute(attribute_node);
        hirmodule.element_metadata.push(ElementMetadata {
            id,
            classes,
            element_type: element_type.to_string(),
            parent: parent_index,
            attributes_ref,
            location: location.clone(),
        });

        let element_index = hirmodule.elements.len();

        match element_type {
            "section" => {
                hirmodule.elements.push(HirElementOp::Section {
                    children: Vec::new(),
                    attributes: attributes_ref,
                });
            }
            "list" => {
                hirmodule.elements.push(HirElementOp::List {
                    children: Vec::new(),
                    attributes: attributes_ref,
                });
            }
            "text" => {
                hirmodule.elements.push(HirElementOp::Text {
                    content: "".to_string(),
                    attributes: attributes_ref,
                });
            }
            "image" => {
                hirmodule.elements.push(HirElementOp::Image {
                    src: "".to_string(),
                    attributes: attributes_ref,
                });
            }
            "table" => {
                hirmodule.elements.push(HirElementOp::Table {
                    table: Vec::new(),
                    attributes: attributes_ref,
                });
            }
            _ => {
                hirmodule.errors.push(HirError {
                    // TODO: add span wonder why its not here or maybe return result instead and bubble up error
                    message: format!("Unknown element type: {}", element_type),
                    severity: Severity::Error,
                    location: location.clone(),
                    span: Span {
                        start: 0,
                        end: 0,
                        file: "".to_string(),
                    },
                });
            }
        }

        (element_index, attributes_ref)
    }

    fn parse_type(&mut self, type_str: &str) -> Option<Type> {
        match type_str {
            "Int" => Some(Type::Int),
            "Float" => Some(Type::Float),
            "String" => Some(Type::String),
            "DocElement" => Some(Type::DocElement),
            _ => None,
        }
    }
}
