use std::collections::HashMap;

use crate::ast::{Ast, DocElement, Expression, Statement};
use crate::hlir::ir_types::{
    AttributeNode, AttributeTree, ElementMetadata, Func, FuncBlock, FuncId, GlobalId, HLIRModule,
    HlirElement, Id, Op, Type,
};

pub fn lower(ast: &Ast) -> HLIRModule {
    let mut pass = HLIRPass {
        ast: ast.clone(),
        symbol_table: Vec::new(),
    };
    pass.lower()
}

pub struct HLIRPass {
    // Fields and methods for the Hir struct
    ast: Ast,
    pub symbol_table: Vec<HashMap<String, Id>>, // Scope stack
}

impl HLIRPass {
    // Methods for the Hlir struct

    fn lower(&mut self) -> HLIRModule {
        let mut hlirmodule = HLIRModule {
            globals: HashMap::new(),
            functions: HashMap::new(),
            attributes: AttributeTree::new(),
            css_rules: Vec::new(),
            elements: Vec::new(),
            element_metadata: Vec::new(),
        };

        self.symbol_table.push(HashMap::new()); // add new scope (global)

        self.lower_template_block(&mut hlirmodule);

        // Store CSS rules from AST
        if let Some(style) = &self.ast.style {
            hlirmodule.css_rules = style.statements.clone();
        }

        self.lower_document_block(&mut hlirmodule);

        self.symbol_table.pop(); // remove scope (global)

        hlirmodule
    }

    fn lower_template_block(&mut self, hlirmodule: &mut HLIRModule) {
        // all global, default and function declarations
        // handle defaults and globals inside this function call since they are small
        let _scope_index = self.symbol_table.len() - 1;

        if let Some(template) = &self.ast.template {
            let statements = template.statements.clone();
            for statement in &statements {
                match statement {
                    Statement::DefaultSet { key, value } => {
                        let global_id =
                            GlobalId(TryInto::<usize>::try_into(hlirmodule.globals.len()).unwrap());
                        let global = self.assign_global(
                            "__".to_string() + &key.clone(),
                            value.clone(),
                            Id::Global(global_id),
                        ); // TODO see if I can get rid of clone
                        hlirmodule.globals.insert(Id::Global(global_id), global);
                        self.add_symbol(key.clone(), Id::Global(global_id));
                    }
                    Statement::ConstAssign { name, value } => {
                        let global_id =
                            GlobalId(TryInto::<usize>::try_into(hlirmodule.globals.len()).unwrap());
                        let global =
                            self.assign_global(name.clone(), value.clone(), Id::Global(global_id)); // TODO see if I can get rid of clone
                        hlirmodule.globals.insert(Id::Global(global_id), global);
                        self.add_symbol(name.clone(), Id::Global(global_id));
                    }
                    Statement::VarAssign { name, value } => {
                        let global_id =
                            GlobalId(TryInto::<usize>::try_into(hlirmodule.globals.len()).unwrap());
                        let global =
                            self.assign_global(name.clone(), value.clone(), Id::Global(global_id)); // TODO see if I can get rid of clone
                        hlirmodule.globals.insert(Id::Global(global_id), global);
                        self.add_symbol(name.clone(), Id::Global(global_id));
                    }
                    Statement::FunctionDecl { name, args, body } => {
                        let func_id =
                            FuncId(TryInto::<usize>::try_into(hlirmodule.functions.len()).unwrap());
                        let hlir_body = self.lower_function_block(body, hlirmodule);
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

                        hlirmodule.functions.insert(
                            Id::Func(func_id),
                            Func {
                                id: Id::Func(func_id),
                                name: name.clone(),
                                args: arg_list,
                                return_type: Some(Type::DocElement), // TODO check return type before setting (right now only DocElement)
                                body: hlir_body,
                            },
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    fn lower_document_block(&mut self, hlirmodule: &mut HLIRModule) {
        let mut ir_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: Some(0), // TODO this return type, magic number and I have a feeling that its wrong
        };

        self.symbol_table.push(HashMap::new()); // add new scope (document)

        if let Some(document) = &self.ast.document {
            let elements = document.elements.clone();
            for element in &elements {
                let index = self.lower_document_element(element, hlirmodule, &mut ir_body, None);

                // Only emit HlirElementEmit for actual elements, not for function calls
                // Calls handle element emission separately via Op::Call
                if !matches!(element, crate::ast::DocElement::Call { .. }) {
                    ir_body.ops.push(Op::HlirElementEmit { index });
                }
            }
        }
        let func_id = FuncId(TryInto::<usize>::try_into(hlirmodule.functions.len()).unwrap());
        hlirmodule.functions.insert(
            Id::Func(func_id),
            Func {
                id: Id::Func(func_id),
                name: "__document".to_string(),
                args: Vec::new(),
                return_type: Some(Type::DocElement), // For right now only DocElements are supported TODO add in other types support later
                body: ir_body,
            },
        );

        self.symbol_table.pop(); // remove scope (document)
    }

    fn lower_document_element(
        &mut self,
        element: &crate::ast::DocElement,
        hlirmodule: &mut HLIRModule,
        ir_body: &mut FuncBlock,
        parent_index: Option<usize>,
    ) -> usize {
        match element {
            crate::ast::DocElement::Call { name, args } => {
                let func_id = match self.find_symbol(name.as_str()) {
                    Some(id) => Some(id),
                    None => panic!("Function not found: {}", name),
                };

                let arg_value_ids = self.handle_args(args, ir_body);
                ir_body.ops.push(Op::Call {
                    func: func_id.unwrap(),
                    result: None,
                    args: arg_value_ids,
                });
                // Call ops don't need to return an index - they handle element emission separately
                // The returned_element_ref in the function body is used instead
                0
            }
            crate::ast::DocElement::Text {
                content,
                attributes,
            } => {
                let (id, classes) = self.extract_id_and_classes(attributes);
                let element_type = "text".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(attributes, hlirmodule.attributes.size);
                let attributes_ref = hlirmodule.attributes.add_attribute(attribute_node);
                hlirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                });

                hlirmodule.elements.push(HlirElement::Text {
                    content: content.to_string(),
                    attributes: attributes_ref,
                });

                hlirmodule.elements.len() - 1
            }
            crate::ast::DocElement::Section {
                elements: section_elements,
                attributes,
            } => {
                let (id, classes) = self.extract_id_and_classes(attributes);
                let element_type = "section".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(attributes, hlirmodule.attributes.size);
                let attributes_ref = hlirmodule.attributes.add_attribute(attribute_node);

                // Reserve index before processing children so children get correct parent
                let index = hlirmodule.elements.len();
                hlirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                });
                // Push placeholder first to reserve the slot
                hlirmodule.elements.push(HlirElement::Section {
                    children: Vec::new(), // Will be updated
                    attributes: attributes_ref,
                });

                let mut children = Vec::new();
                for child in section_elements {
                    children.push(self.lower_document_element(
                        child,
                        hlirmodule,
                        ir_body,
                        Some(index),
                    ));
                }

                // Update with actual children
                hlirmodule.elements[index] = HlirElement::Section {
                    children,
                    attributes: attributes_ref,
                };

                index
            }
            crate::ast::DocElement::List { items, attributes } => {
                let (id, classes) = self.extract_id_and_classes(attributes);
                let element_type = "list".to_string();
                let attribute_node =
                    AttributeNode::new_with_attributes(attributes, hlirmodule.attributes.size);
                let attributes_ref = hlirmodule.attributes.add_attribute(attribute_node);

                // Reserve index before processing children so children get correct parent
                let index = hlirmodule.elements.len();
                hlirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type,
                    parent: parent_index,
                    attributes_ref,
                });
                // Push placeholder first to reserve the slot
                hlirmodule.elements.push(HlirElement::List {
                    children: Vec::new(), // Will be updated
                    attributes: attributes_ref,
                });

                let mut children = Vec::new();
                for child in items {
                    children.push(self.lower_document_element(
                        child,
                        hlirmodule,
                        ir_body,
                        Some(index),
                    ));
                }

                // Update with actual children
                hlirmodule.elements[index] = HlirElement::List {
                    children,
                    attributes: attributes_ref,
                };

                index
            }
            // TODO: Handle Image, Code, Link, Table similarly
            _ => {
                panic!(
                    "Unsupported document element: {:?}  (HLIR document lowering)",
                    element
                );
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

    pub fn convert_doc_element_to_hlir(
        &self,
        element: &crate::ast::DocElement,
        hlirmodule: &mut HLIRModule,
    ) -> HlirElement {
        match element {
            crate::ast::DocElement::Text {
                content,
                attributes,
            } => {
                let (id, classes) = self.extract_id_and_classes(attributes);
                let attribute_node =
                    AttributeNode::new_with_attributes(attributes, hlirmodule.attributes.size);
                let attributes_ref = hlirmodule.attributes.add_attribute(attribute_node);
                // Push metadata for this element
                hlirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type: "text".to_string(),
                    parent: None,
                    attributes_ref,
                });
                HlirElement::Text {
                    content: content.clone(),
                    attributes: attributes_ref,
                }
            }
            crate::ast::DocElement::Section {
                elements,
                attributes,
            } => {
                let (id, classes) = self.extract_id_and_classes(attributes);
                let attribute_node =
                    AttributeNode::new_with_attributes(attributes, hlirmodule.attributes.size);
                let attributes_ref = hlirmodule.attributes.add_attribute(attribute_node);
                // Get the parent index for children (current section's index)
                let parent_index = hlirmodule.elements.len();
                // Recursively convert all children
                let children: Vec<usize> = elements
                    .iter()
                    .map(|child| {
                        // Update parent in child's metadata after we know the parent's index
                        let child_hlir = self.convert_doc_element_to_hlir(child, hlirmodule);
                        let child_index = hlirmodule.elements.len();
                        hlirmodule.elements.push(child_hlir);
                        // Update the child's metadata to set the parent
                        if let Some(metadata) = hlirmodule.element_metadata.get_mut(child_index) {
                            metadata.parent = Some(parent_index);
                        }
                        child_index
                    })
                    .collect();
                // Push metadata for this section element
                hlirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type: "section".to_string(),
                    parent: None,
                    attributes_ref,
                });
                HlirElement::Section {
                    children,
                    attributes: attributes_ref,
                }
            }
            crate::ast::DocElement::List { items, attributes } => {
                let (id, classes) = self.extract_id_and_classes(attributes);
                let attribute_node =
                    AttributeNode::new_with_attributes(attributes, hlirmodule.attributes.size);
                let attributes_ref = hlirmodule.attributes.add_attribute(attribute_node);
                // Get the parent index for children (current list's index)
                let parent_index = hlirmodule.elements.len();
                // Recursively convert all list items
                let children: Vec<usize> = items
                    .iter()
                    .map(|item| {
                        let child_hlir = self.convert_doc_element_to_hlir(item, hlirmodule);
                        let child_index = hlirmodule.elements.len();
                        hlirmodule.elements.push(child_hlir);
                        // Update the child's metadata to set the parent
                        if let Some(metadata) = hlirmodule.element_metadata.get_mut(child_index) {
                            metadata.parent = Some(parent_index);
                        }
                        child_index
                    })
                    .collect();
                // Push metadata for this list element
                hlirmodule.element_metadata.push(ElementMetadata {
                    id,
                    classes,
                    element_type: "list".to_string(),
                    parent: None,
                    attributes_ref,
                });
                HlirElement::List {
                    children,
                    attributes: attributes_ref,
                }
            }
            _ => {
                // For other element types, create a placeholder text element
                // This is a temporary bandaid for the MLIR migration
                // TODO fix
                hlirmodule.element_metadata.push(ElementMetadata {
                    id: None,
                    classes: Vec::new(),
                    element_type: "text".to_string(),
                    parent: None,
                    attributes_ref: 1,
                });
                HlirElement::Text {
                    content: String::new(),
                    attributes: 1, // Root attribute node
                }
            }
        }
    }
}
