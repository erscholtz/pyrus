use std::collections::HashMap;

use crate::ast::{
    ArgType, Ast, BinaryExpr, CallElem, ChildrenElem, CodeElem, DocElem, DocElemKind, Expr,
    ExprKind, FuncDeclStmt, ImageElem, InterpolatedStringExpr, LinkElem, ListElem, ReturnStmt,
    SectionElem, Stmt, StmtKind, TableElem, TextElem,
};
use crate::diagnostic::{DiagnosticManager, SourceLocation};
pub use crate::hir::{
    PassManager,
    hir_passes::{func_pass::FuncPass, global_pass::GlobalPass, style_pass::StylePass},
    hir_types::{
        AttributeNode, AttributeTree, ElementId, ElementMetadata, FuncBlock, FuncDecl, FuncId,
        Global, GlobalId, HIRModule, HirElementDecl, HirElementOp, Literal, Op, StyleAttributes,
        Type, ValueId,
    },
    hir_util::{
        handle_args::parse_type,
        handle_expr::{assign_global, assign_local},
    },
};

pub fn lower(ast: &Ast, dm: &mut DiagnosticManager) -> Option<HIRModule> {
    let mut pass = HirPass { ast };
    Some(pass.lower(dm))
}

pub struct HirPass<'ast> {
    pub ast: &'ast Ast,
}

impl<'ast> HirPass<'ast> {
    fn lower(&mut self, dm: &mut DiagnosticManager) -> HIRModule {
        let mut hirmodule = HIRModule {
            file: self.ast.file.clone(),
            globals: HashMap::new(),
            functions: HashMap::new(),
            element_decls: HashMap::new(),
            attributes: AttributeTree::new(),
            css_rules: Vec::new(),
            elements: Vec::new(),
            element_metadata: Vec::new(),
        };

        let result = PassManager::default()
            .continue_on_error()
            // template
            .run::<GlobalPass>(&mut hirmodule, self.ast) // global variables
            .run::<FuncPass>(&mut hirmodule, self.ast) // function declarations
            // document
            // style
            .run::<StylePass>(&mut hirmodule, self.ast) // css styling
            // checks
            .finished();

        if let Err(errors) = result {
            dm.extend(errors);
        }

        hirmodule
    }

    fn lower_template_block(&mut self, hirmodule: &mut HIRModule) {
        let Some(template) = &self.ast.template else {
            return;
        };

        for statement in &template.statements {
            match &statement.node {
                StmtKind::DefaultSet(stmt) => {
                    let id = GlobalId(hirmodule.globals.len());
                    let global = assign_global(&format!("__{}", stmt.key), &stmt.value, false);
                    hirmodule.globals.insert(id, global);
                }
                StmtKind::ConstAssign(stmt) => {
                    let id = GlobalId(hirmodule.globals.len());
                    let global = assign_global(&stmt.name, &stmt.value, false);
                    hirmodule.globals.insert(id, global);
                }
                StmtKind::VarAssign(stmt) => {
                    let id = GlobalId(hirmodule.globals.len());
                    let global = assign_global(&stmt.name, &stmt.value, true);
                    hirmodule.globals.insert(id, global);
                }
                StmtKind::FuncDecl(func) => self.lower_element_decl(func, hirmodule),
                _ => {}
            }
        }
    }

    fn lower_element_decl(&mut self, func: &FuncDeclStmt, hirmodule: &mut HIRModule) {
        let element_id = ElementId(hirmodule.element_decls.len());

        let args = func
            .args
            .iter()
            .filter_map(|arg| parse_type(&arg.ty))
            .collect::<Vec<_>>();

        let body = self.lower_element_body(&func.body, hirmodule);

        hirmodule.element_decls.insert(
            element_id,
            HirElementDecl {
                name: func.name.clone(),
                args,
                body,
            },
        );
    }

    fn lower_document_block(&mut self, hirmodule: &mut HIRModule) {
        let mut ir_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        };

        if let Some(document) = &self.ast.document {
            for element in &document.elements {
                let index = self.lower_document_element(element, hirmodule, &mut ir_body, None);
                if !matches!(element.node, DocElemKind::Call(_)) && index != usize::MAX {
                    ir_body.ops.push(Op::HirElementEmit { index });
                }
            }
        }

        let func_id = FuncId(hirmodule.functions.len());
        hirmodule.functions.insert(
            func_id,
            FuncDecl {
                name: "__document".to_string(),
                args: Vec::new(),
                return_type: Some(Type::DocElement),
                body: ir_body,
            },
        );
    }

    pub fn lower_document_element(
        &mut self,
        element: &DocElem,
        hirmodule: &mut HIRModule,
        ir_body: &mut FuncBlock,
        parent_index: Option<usize>,
    ) -> usize {
        let location = element.location.clone();
        match &element.node {
            DocElemKind::Call(CallElem {
                name,
                args,
                children,
            }) => self.lower_call_element(
                name,
                args,
                children.as_ref(),
                hirmodule,
                ir_body,
                parent_index,
                location,
            ),
            DocElemKind::Text(TextElem {
                content,
                attributes,
            }) => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "text",
                    attributes.as_ref(),
                    parent_index,
                    location,
                );
                hirmodule.elements[index] = HirElementOp::Text {
                    content: content.to_string(),
                    attributes: attributes_ref,
                };
                index
            }
            DocElemKind::Section(SectionElem {
                elements,
                attributes,
            }) => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "section",
                    attributes.as_ref(),
                    parent_index,
                    location,
                );
                let children = elements
                    .iter()
                    .map(|child| {
                        self.lower_document_element(child, hirmodule, ir_body, Some(index))
                    })
                    .filter(|index| *index != usize::MAX)
                    .collect();
                hirmodule.elements[index] = HirElementOp::Section {
                    children,
                    attributes: attributes_ref,
                };
                index
            }
            DocElemKind::List(ListElem {
                items, attributes, ..
            }) => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "list",
                    attributes.as_ref(),
                    parent_index,
                    location,
                );
                let children = items
                    .iter()
                    .map(|child| {
                        self.lower_document_element(child, hirmodule, ir_body, Some(index))
                    })
                    .filter(|index| *index != usize::MAX)
                    .collect();
                hirmodule.elements[index] = HirElementOp::List {
                    children,
                    attributes: attributes_ref,
                };
                index
            }
            DocElemKind::Image(ImageElem { src, attributes }) => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "image",
                    attributes.as_ref(),
                    parent_index,
                    location,
                );
                hirmodule.elements[index] = HirElementOp::Image {
                    src: src.clone(),
                    attributes: attributes_ref,
                };
                index
            }
            DocElemKind::Table(TableElem { table, attributes }) => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "table",
                    attributes.as_ref(),
                    parent_index,
                    location,
                );
                let lowered = table
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|cell| {
                                self.lower_document_element(cell, hirmodule, ir_body, Some(index))
                            })
                            .filter(|index| *index != usize::MAX)
                            .collect::<Vec<_>>()
                    })
                    .collect();
                hirmodule.elements[index] = HirElementOp::Table {
                    table: lowered,
                    attributes: attributes_ref,
                };
                index
            }
            DocElemKind::Link(LinkElem {
                href,
                content,
                attributes,
            }) => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "text",
                    attributes.as_ref(),
                    parent_index,
                    location,
                );
                hirmodule.elements[index] = HirElementOp::Text {
                    content: format!("{content} ({href})"),
                    attributes: attributes_ref,
                };
                index
            }
            DocElemKind::Code(CodeElem {
                content,
                attributes,
            }) => {
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "text",
                    attributes.as_ref(),
                    parent_index,
                    location,
                );
                hirmodule.elements[index] = HirElementOp::Text {
                    content: content.clone(),
                    attributes: attributes_ref,
                };
                index
            }
            DocElemKind::Children(ChildrenElem { .. }) => {
                let mut attributes = HashMap::new();
                attributes.insert(
                    "class".to_string(),
                    Expr::new(
                        ExprKind::StringLiteral("children".to_string()),
                        location.clone(),
                    ),
                );
                let (index, attributes_ref) = self.reserve_element_slot(
                    hirmodule,
                    "section",
                    Some(&attributes),
                    parent_index,
                    location,
                );
                hirmodule.elements[index] = HirElementOp::Section {
                    children: Vec::new(),
                    attributes: attributes_ref,
                };
                index
            }
        }
    }

    fn lower_call_element(
        &mut self,
        name: &str,
        args: &[ArgType],
        children: Option<&Vec<DocElem>>,
        hirmodule: &mut HIRModule,
        ir_body: &mut FuncBlock,
        parent_index: Option<usize>,
        location: SourceLocation,
    ) -> usize {
        let Some(element_id) = find_element_decl(hirmodule, name) else {
            return usize::MAX;
        };

        let arg_value_ids = self.handle_args(args, ir_body);

        let mut wrapper_attrs = HashMap::new();
        wrapper_attrs.insert(
            "class".to_string(),
            Expr::new(ExprKind::StringLiteral(name.to_string()), location.clone()),
        );

        let (wrapper_index, wrapper_attr_ref) = self.reserve_element_slot(
            hirmodule,
            "section",
            Some(&wrapper_attrs),
            parent_index,
            location.clone(),
        );

        let mut wrapper_children = Vec::new();
        if let Some(children) = children.filter(|children| !children.is_empty()) {
            let mut children_attrs = HashMap::new();
            children_attrs.insert(
                "class".to_string(),
                Expr::new(
                    ExprKind::StringLiteral("children".to_string()),
                    location.clone(),
                ),
            );

            let (children_index, children_attr_ref) = self.reserve_element_slot(
                hirmodule,
                "section",
                Some(&children_attrs),
                Some(wrapper_index),
                location.clone(),
            );

            let lowered_children = children
                .iter()
                .map(|child| {
                    self.lower_document_element(child, hirmodule, ir_body, Some(children_index))
                })
                .filter(|index| *index != usize::MAX)
                .collect();

            hirmodule.elements[children_index] = HirElementOp::Section {
                children: lowered_children,
                attributes: children_attr_ref,
            };
            wrapper_children.push(children_index);
        }

        let result_id = ValueId(ir_body.ops.len());
        ir_body.ops.push(Op::ElementCall {
            result: result_id,
            element: element_id,
            args: arg_value_ids,
        });

        hirmodule.elements[wrapper_index] = HirElementOp::Section {
            children: wrapper_children,
            attributes: wrapper_attr_ref,
        };

        wrapper_index
    }

    fn handle_args(&mut self, arguments: &[ArgType], ir_body: &mut FuncBlock) -> Vec<ValueId> {
        let mut args = Vec::new();

        for arg in arguments {
            match arg.ty {
                crate::ast::Type::Var => {
                    // Identifier argument resolution belongs in validation/name resolution.
                }
                crate::ast::Type::Int => {
                    if let Ok(value) = arg.name.parse::<i64>() {
                        let id = ValueId(ir_body.ops.len());
                        ir_body.ops.push(Op::Const {
                            result: id,
                            name: format!("raw_arg_{}", ir_body.ops.len()),
                            literal: Literal::Int(value),
                            ty: Type::Int,
                        });
                        args.push(id);
                    }
                }
                crate::ast::Type::Float => {
                    if let Ok(value) = arg.name.parse::<f64>() {
                        let id = ValueId(ir_body.ops.len());
                        ir_body.ops.push(Op::Const {
                            result: id,
                            name: format!("raw_arg_{}", ir_body.ops.len()),
                            literal: Literal::Float(value),
                            ty: Type::Float,
                        });
                        args.push(id);
                    }
                }
                crate::ast::Type::String => {
                    let id = ValueId(ir_body.ops.len());
                    ir_body.ops.push(Op::Const {
                        result: id,
                        name: format!("raw_arg_{}", ir_body.ops.len()),
                        literal: Literal::String(arg.name.clone()),
                        ty: Type::String,
                    });
                    args.push(id);
                }
                crate::ast::Type::DocElem => {}
            }
        }

        args
    }

    fn lower_element_body(&mut self, body: &[Stmt], hirmodule: &mut HIRModule) -> FuncBlock {
        let mut ir_body = FuncBlock {
            ops: Vec::new(),
            returned_element_ref: None,
        };

        for stmt in body {
            match &stmt.node {
                StmtKind::ConstAssign(stmt) => {
                    let id = ValueId(ir_body.ops.len());
                    let op = assign_local(stmt.name.clone(), &stmt.value, id, false);
                    ir_body.ops.push(op);
                }
                StmtKind::VarAssign(stmt) => {
                    let id = ValueId(ir_body.ops.len());
                    let op = assign_local(stmt.name.clone(), &stmt.value, id, true);
                    ir_body.ops.push(op);
                }
                StmtKind::Return(ReturnStmt::DocElem(doc_element)) => {
                    let element_id =
                        self.lower_document_element(doc_element, hirmodule, &mut ir_body, None);
                    ir_body.ops.push(Op::Return {
                        doc_element_ref: element_id,
                    });
                    ir_body.returned_element_ref = Some(element_id);
                }
                StmtKind::Return(ReturnStmt::Expr(expr)) => {
                    let id = ValueId(ir_body.ops.len());
                    let op = assign_local("__return".to_string(), expr, id, false);
                    ir_body.ops.push(op);
                }
                StmtKind::Children(_) => {}
                _ => {}
            }
        }

        ir_body
    }

    fn extract_id_and_classes(
        &self,
        attributes: Option<&HashMap<String, Expr>>,
    ) -> (Option<String>, Vec<String>) {
        let Some(attributes) = attributes else {
            return (None, Vec::new());
        };

        let id = attributes.get("id").map(ToString::to_string);
        let classes = attributes
            .get("class")
            .map(|expr| {
                expr.to_string()
                    .split_whitespace()
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        (id, classes)
    }

    fn reserve_element_slot(
        &mut self,
        hirmodule: &mut HIRModule,
        element_type: &str,
        attributes: Option<&HashMap<String, Expr>>,
        parent_index: Option<usize>,
        location: SourceLocation,
    ) -> (usize, usize) {
        let (id, classes) = self.extract_id_and_classes(attributes);
        let attribute_node = AttributeNode::new_with_attributes(attributes);
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
        let placeholder = match element_type {
            "section" => HirElementOp::Section {
                children: Vec::new(),
                attributes: attributes_ref,
            },
            "list" => HirElementOp::List {
                children: Vec::new(),
                attributes: attributes_ref,
            },
            "text" => HirElementOp::Text {
                content: String::new(),
                attributes: attributes_ref,
            },
            "image" => HirElementOp::Image {
                src: String::new(),
                attributes: attributes_ref,
            },
            "table" => HirElementOp::Table {
                table: Vec::new(),
                attributes: attributes_ref,
            },
            _ => {
                // FIX return result
                // self.push_error(
                //     hirmodule,
                //     format!("Unknown element type: {element_type}"),
                //     location,
                // );
                HirElementOp::Section {
                    children: Vec::new(),
                    attributes: attributes_ref,
                }
            }
        };

        hirmodule.elements.push(placeholder);
        (element_index, attributes_ref)
    }
}

fn find_element_decl(hirmodule: &HIRModule, name: &str) -> Option<ElementId> {
    hirmodule
        .element_decls
        .iter()
        .find_map(|(id, decl)| (decl.name == name).then_some(*id))
}
