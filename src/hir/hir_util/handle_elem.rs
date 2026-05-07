use std::collections::HashMap;

use crate::ast::{
    ArgType, CallElem, ChildrenElem, CodeElem, DocElem, DocElemKind, Expr, ExprKind, ImageElem,
    LinkElem, ListElem, SectionElem, TableElem, TextElem,
};
use crate::diagnostic::{SemanticError, SourceLocation};
use crate::hir::{
    HIRModule,
    hir_types::{AttributeNode, ElementId, ElementMetadata, FuncBlock, HirElementOp, Op, ValueId},
    hir_util::handle_args::handle_args,
};

pub fn lower_document_element(
    element: &DocElem,
    hirmodule: &mut HIRModule,
    ir_body: &mut FuncBlock,
    parent_index: Option<usize>,
) -> Result<usize, Vec<SemanticError>> {
    let location = element.location.clone();
    match &element.node {
        DocElemKind::Call(CallElem {
            name,
            args,
            children,
        }) => lower_call_element(
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
            let (index, attributes_ref) = reserve_element_slot(
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
            Ok(index)
        }
        DocElemKind::Section(SectionElem {
            elements,
            attributes,
        }) => {
            let (index, attributes_ref) = reserve_element_slot(
                hirmodule,
                "section",
                attributes.as_ref(),
                parent_index,
                location,
            );
            let children = lower_document_children(elements, hirmodule, ir_body, Some(index))?;
            hirmodule.elements[index] = HirElementOp::Section {
                children,
                attributes: attributes_ref,
            };
            Ok(index)
        }
        DocElemKind::List(ListElem {
            items, attributes, ..
        }) => {
            let (index, attributes_ref) = reserve_element_slot(
                hirmodule,
                "list",
                attributes.as_ref(),
                parent_index,
                location,
            );
            let children = lower_document_children(items, hirmodule, ir_body, Some(index))?;
            hirmodule.elements[index] = HirElementOp::List {
                children,
                attributes: attributes_ref,
            };
            Ok(index)
        }
        DocElemKind::Image(ImageElem { src, attributes }) => {
            let (index, attributes_ref) = reserve_element_slot(
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
            Ok(index)
        }
        DocElemKind::Table(TableElem { table, attributes }) => {
            let (index, attributes_ref) = reserve_element_slot(
                hirmodule,
                "table",
                attributes.as_ref(),
                parent_index,
                location,
            );
            let lowered = lower_table_cells(table, hirmodule, ir_body, Some(index))?;
            hirmodule.elements[index] = HirElementOp::Table {
                table: lowered,
                attributes: attributes_ref,
            };
            Ok(index)
        }
        DocElemKind::Link(LinkElem {
            href,
            content,
            attributes,
        }) => {
            let (index, attributes_ref) = reserve_element_slot(
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
            Ok(index)
        }
        DocElemKind::Code(CodeElem {
            content,
            attributes,
        }) => {
            let (index, attributes_ref) = reserve_element_slot(
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
            Ok(index)
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
            let (index, attributes_ref) = reserve_element_slot(
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
            Ok(index)
        }
    }
}

fn lower_document_children(
    elements: &[DocElem],
    hirmodule: &mut HIRModule,
    ir_body: &mut FuncBlock,
    parent_index: Option<usize>,
) -> Result<Vec<usize>, Vec<SemanticError>> {
    let mut children = Vec::new();
    let mut errors = Vec::new();

    for child in elements {
        match lower_document_element(child, hirmodule, ir_body, parent_index) {
            Ok(index) => children.push(index),
            Err(err) => errors.extend(err),
        }
    }

    if errors.is_empty() {
        Ok(children)
    } else {
        Err(errors)
    }
}

fn lower_table_cells(
    table: &[Vec<DocElem>],
    hirmodule: &mut HIRModule,
    ir_body: &mut FuncBlock,
    parent_index: Option<usize>,
) -> Result<Vec<Vec<usize>>, Vec<SemanticError>> {
    let mut lowered = Vec::new();
    let mut errors = Vec::new();

    for row in table {
        match lower_document_children(row, hirmodule, ir_body, parent_index) {
            Ok(cells) => lowered.push(cells),
            Err(err) => errors.extend(err),
        }
    }

    if errors.is_empty() {
        Ok(lowered)
    } else {
        Err(errors)
    }
}

fn lower_call_element(
    name: &str,
    args: &[ArgType],
    children: Option<&Vec<DocElem>>,
    hirmodule: &mut HIRModule,
    ir_body: &mut FuncBlock,
    parent_index: Option<usize>,
    location: SourceLocation,
) -> Result<usize, Vec<SemanticError>> {
    let mut errors = Vec::new();

    let arg_value_ids = match handle_args(args, ir_body) {
        Ok(ids) => ids,
        Err(err) => {
            errors.extend(err);
            return Err(errors);
        }
    };
    let returned_element_ref = find_function_returned_element(hirmodule, name);

    let mut wrapper_attrs = HashMap::new();
    wrapper_attrs.insert(
        "class".to_string(),
        Expr::new(ExprKind::StringLiteral(name.to_string()), location.clone()),
    );

    let (wrapper_index, wrapper_attr_ref) = reserve_element_slot(
        hirmodule,
        "section",
        Some(&wrapper_attrs),
        parent_index,
        location.clone(),
    );

    let mut wrapper_children = Vec::new();
    if let Some(returned_element_ref) = returned_element_ref {
        wrapper_children.push(returned_element_ref);
    }

    if let Some(children) = children.filter(|children| !children.is_empty()) {
        let mut children_attrs = HashMap::new();
        children_attrs.insert(
            "class".to_string(),
            Expr::new(
                ExprKind::StringLiteral("children".to_string()),
                location.clone(),
            ),
        );

        let (children_index, children_attr_ref) = reserve_element_slot(
            hirmodule,
            "section",
            Some(&children_attrs),
            Some(wrapper_index),
            location.clone(),
        );

        let lowered_children =
            lower_document_children(children, hirmodule, ir_body, Some(children_index))?;

        hirmodule.elements[children_index] = HirElementOp::Section {
            children: lowered_children,
            attributes: children_attr_ref,
        };
        wrapper_children.push(children_index);
    }

    let result_id = ValueId(ir_body.ops.len());
    ir_body.ops.push(Op::ElementCall {
        name: name.to_string(),
        result: result_id,
        element: None,
        args: arg_value_ids,
    });

    hirmodule.elements[wrapper_index] = HirElementOp::Section {
        children: wrapper_children,
        attributes: wrapper_attr_ref,
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(wrapper_index)
}

fn extract_id_and_classes(
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
    hirmodule: &mut HIRModule,
    element_type: &str,
    attributes: Option<&HashMap<String, Expr>>,
    parent_index: Option<usize>,
    location: SourceLocation,
) -> (usize, usize) {
    let (id, classes) = extract_id_and_classes(attributes);
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

fn find_element_decl(hirmodule: &HIRModule, name: &str) -> Option<ElementId> {
    hirmodule
        .element_decls
        .iter()
        .find_map(|(id, decl)| (decl.name == name).then_some(*id))
}

fn find_function_returned_element(hirmodule: &HIRModule, name: &str) -> Option<usize> {
    hirmodule
        .functions
        .values()
        .find(|decl| decl.name == name)
        .and_then(|decl| decl.body.returned_element_ref)
}
