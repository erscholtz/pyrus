use std::collections::HashMap;

use crate::ast::{
    ArgType, BinaryExpr, CallElem, ChildrenElem, CodeElem, DocElem, DocElemKind, Expr, ExprKind,
    ImageElem, InterpolatedStringExpr, LinkElem, ListElem, SectionElem, SeparatorElem,
    StructDefaultExpr, TableElem, TextElem, UnaryExpr,
};
use crate::diagnostic::{SemanticError, SourceLocation};
use crate::hir::{
    HIRModule,
    hir_types::{AttributeNode, ElementMetadata, FuncBlock, FuncDecl, HirElementOp, Op, ValueId},
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
                content_expr: Some(content.clone()),
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
        DocElemKind::Separator(SeparatorElem { attributes }) => {
            let (index, attributes_ref) = reserve_element_slot(
                hirmodule,
                "separator",
                attributes.as_ref(),
                parent_index,
                location,
            );
            hirmodule.elements[index] = HirElementOp::Separator {
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
                content_expr: None,
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
                content_expr: None,
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
    let function = find_function_decl(hirmodule, name).cloned();
    let function_found = function.is_some();

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
    if let Some(function) = function {
        if let Some(returned_element_ref) = function.body.returned_element_ref {
            let substitutions = build_arg_substitutions(&function, args);
            match clone_element_tree_for_call(
                returned_element_ref,
                hirmodule,
                ir_body,
                Some(wrapper_index),
                &substitutions,
                children,
            ) {
                Ok(cloned_element_ref) => wrapper_children.push(cloned_element_ref),
                Err(err) => errors.extend(err),
            }
        }
    }

    if !function_found {
        // Unknown calls still preserve their children so later validation can report the call
        // without losing surrounding document content.
        if let Some(children) = children.filter(|children| !children.is_empty()) {
            let lowered_children =
                lower_document_children(children, hirmodule, ir_body, Some(wrapper_index))?;
            wrapper_children.extend(lowered_children);
        }
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
            content_expr: None,
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
        "separator" => HirElementOp::Separator {
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

fn find_function_decl<'hir>(hirmodule: &'hir HIRModule, name: &str) -> Option<&'hir FuncDecl> {
    hirmodule.functions.values().find(|decl| decl.name == name)
}

fn build_arg_substitutions(function: &FuncDecl, args: &[ArgType]) -> HashMap<String, String> {
    function
        .arg_names
        .iter()
        .zip(args.iter())
        .map(|(name, arg)| (name.clone(), arg.name.clone()))
        .collect()
}

fn clone_element_tree_for_call(
    element_index: usize,
    hirmodule: &mut HIRModule,
    ir_body: &mut FuncBlock,
    parent_index: Option<usize>,
    substitutions: &HashMap<String, String>,
    call_children: Option<&Vec<DocElem>>,
) -> Result<usize, Vec<SemanticError>> {
    let element = hirmodule.elements[element_index].clone();
    let metadata = hirmodule.element_metadata[element_index].clone();

    if is_children_placeholder(&element, &metadata) {
        return clone_children_placeholder(
            hirmodule,
            ir_body,
            parent_index,
            metadata.attributes_ref,
            metadata.location,
            call_children,
        );
    }

    let attributes_ref = clone_attributes(metadata.attributes_ref, hirmodule);
    let new_index = reserve_cloned_element_slot(&metadata, attributes_ref, parent_index, hirmodule);

    let cloned_element = match element {
        HirElementOp::Section { children, .. } => {
            let cloned_children = clone_child_elements(
                &children,
                hirmodule,
                ir_body,
                Some(new_index),
                substitutions,
                call_children,
            )?;
            HirElementOp::Section {
                children: cloned_children,
                attributes: attributes_ref,
            }
        }
        HirElementOp::List { children, .. } => {
            let cloned_children = clone_child_elements(
                &children,
                hirmodule,
                ir_body,
                Some(new_index),
                substitutions,
                call_children,
            )?;
            HirElementOp::List {
                children: cloned_children,
                attributes: attributes_ref,
            }
        }
        HirElementOp::Text {
            content,
            content_expr,
            ..
        } => {
            let content = content_expr
                .as_ref()
                .map(|expr| render_expr_with_substitutions(&expr.node, substitutions))
                .unwrap_or(content);
            HirElementOp::Text {
                content,
                content_expr,
                attributes: attributes_ref,
            }
        }
        HirElementOp::Image { src, .. } => HirElementOp::Image {
            src,
            attributes: attributes_ref,
        },
        HirElementOp::Table { table, .. } => {
            let mut cloned_rows = Vec::new();
            for row in table {
                cloned_rows.push(clone_child_elements(
                    &row,
                    hirmodule,
                    ir_body,
                    Some(new_index),
                    substitutions,
                    call_children,
                )?);
            }
            HirElementOp::Table {
                table: cloned_rows,
                attributes: attributes_ref,
            }
        }
        HirElementOp::Separator { .. } => HirElementOp::Separator {
            attributes: attributes_ref,
        },
    };

    hirmodule.elements[new_index] = cloned_element;
    Ok(new_index)
}

fn clone_child_elements(
    children: &[usize],
    hirmodule: &mut HIRModule,
    ir_body: &mut FuncBlock,
    parent_index: Option<usize>,
    substitutions: &HashMap<String, String>,
    call_children: Option<&Vec<DocElem>>,
) -> Result<Vec<usize>, Vec<SemanticError>> {
    let mut cloned = Vec::new();
    let mut errors = Vec::new();

    for child in children {
        match clone_element_tree_for_call(
            *child,
            hirmodule,
            ir_body,
            parent_index,
            substitutions,
            call_children,
        ) {
            Ok(index) => cloned.push(index),
            Err(err) => errors.extend(err),
        }
    }

    if errors.is_empty() {
        Ok(cloned)
    } else {
        Err(errors)
    }
}

fn is_children_placeholder(element: &HirElementOp, metadata: &ElementMetadata) -> bool {
    matches!(element, HirElementOp::Section { children, .. } if children.is_empty())
        && metadata.element_type == "section"
        && metadata.classes.iter().any(|class| class == "children")
}

fn clone_children_placeholder(
    hirmodule: &mut HIRModule,
    ir_body: &mut FuncBlock,
    parent_index: Option<usize>,
    old_attributes_ref: usize,
    location: SourceLocation,
    call_children: Option<&Vec<DocElem>>,
) -> Result<usize, Vec<SemanticError>> {
    let attributes_ref = clone_attributes(old_attributes_ref, hirmodule);

    hirmodule.element_metadata.push(ElementMetadata {
        id: None,
        classes: vec!["children".to_string()],
        element_type: "section".to_string(),
        parent: parent_index,
        attributes_ref,
        location,
    });

    let new_index = hirmodule.elements.len();
    hirmodule.elements.push(HirElementOp::Section {
        children: Vec::new(),
        attributes: attributes_ref,
    });

    let children = if let Some(call_children) = call_children {
        lower_document_children(call_children, hirmodule, ir_body, Some(new_index))?
    } else {
        Vec::new()
    };

    hirmodule.elements[new_index] = HirElementOp::Section {
        children,
        attributes: attributes_ref,
    };
    Ok(new_index)
}

fn clone_attributes(attributes_ref: usize, hirmodule: &mut HIRModule) -> usize {
    let inline = hirmodule
        .attributes
        .find_node(attributes_ref)
        .map(|node| node.inline.clone())
        .unwrap_or_default();

    hirmodule.attributes.add_attribute(AttributeNode {
        parent: None,
        id: 0,
        inline,
        computed: Default::default(),
        children: HashMap::new(),
    })
}

fn reserve_cloned_element_slot(
    metadata: &ElementMetadata,
    attributes_ref: usize,
    parent_index: Option<usize>,
    hirmodule: &mut HIRModule,
) -> usize {
    hirmodule.element_metadata.push(ElementMetadata {
        id: metadata.id.clone(),
        classes: metadata.classes.clone(),
        element_type: metadata.element_type.clone(),
        parent: parent_index,
        attributes_ref,
        location: metadata.location.clone(),
    });

    let element_index = hirmodule.elements.len();
    hirmodule.elements.push(HirElementOp::Section {
        children: Vec::new(),
        attributes: attributes_ref,
    });
    element_index
}

fn render_expr_with_substitutions(
    expr: &ExprKind,
    substitutions: &HashMap<String, String>,
) -> String {
    match expr {
        ExprKind::StringLiteral(value) => value.clone(),
        ExprKind::InterpolatedString(InterpolatedStringExpr { parts }) => parts
            .iter()
            .map(|part| render_expr_with_substitutions(part, substitutions))
            .collect(),
        ExprKind::Identifier(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.clone()),
        ExprKind::Int(value) => value.to_string(),
        ExprKind::Float(value) => value.to_string(),
        ExprKind::StructDefault(StructDefaultExpr { name }) => format!("default({name})"),
        ExprKind::Binary(BinaryExpr { left, op, right }) => {
            let op = match op {
                crate::ast::BinOp::Add => "+",
                crate::ast::BinOp::Subtract => "-",
                crate::ast::BinOp::Multiply => "*",
                crate::ast::BinOp::Divide => "/",
                crate::ast::BinOp::Equals => "=",
                crate::ast::BinOp::Mod => "%",
            };
            format!(
                "{} {} {}",
                render_expr_with_substitutions(left, substitutions),
                op,
                render_expr_with_substitutions(right, substitutions)
            )
        }
        ExprKind::Unary(UnaryExpr { op, expr }) => {
            let op = match op {
                crate::ast::UnaryOp::Negate => "-",
                crate::ast::UnaryOp::Not => "!",
            };
            format!(
                "{op}{}",
                render_expr_with_substitutions(expr, substitutions)
            )
        }
    }
}
