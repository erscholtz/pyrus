mod common;

use common::{document_elements, parse_errors};
use pyrus::{
    ast::{DocElemKind, ExprKind, Type},
    diagnostic::Diagnostic,
};

#[test]
fn test_parse_text_element() {
    let elements = document_elements("document { @text[Hello, World!] }");
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Text(text) => match &text.content.node {
            ExprKind::StringLiteral(value) => assert_eq!(value, "Hello, World!"),
            other => panic!("Expected StringLiteral text content, got {other:?}"),
        },
        other => panic!("Expected text element, got {other:?}"),
    }
}

#[test]
fn test_parse_text_element_with_attributes() {
    let elements = document_elements(r#"document { @text(class="hero", size=24)[Hello] }"#);
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Text(text) => {
            let attrs = text.attributes.as_ref().expect("Expected text attributes");
            assert!(
                matches!(attrs.get("class").map(|expr| &expr.node), Some(ExprKind::StringLiteral(value)) if value == "hero")
            );
            assert!(matches!(
                attrs.get("size").map(|expr| &expr.node),
                Some(ExprKind::Int(24))
            ));
        }
        other => panic!("Expected text element, got {other:?}"),
    }
}

#[test]
fn test_parse_image_element() {
    let elements = document_elements(r#"document { @image(width=320)["cover.png"] }"#);
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Image(image) => {
            assert_eq!(image.src, "cover.png");
            let attrs = image
                .attributes
                .as_ref()
                .expect("Expected image attributes");
            assert!(matches!(
                attrs.get("width").map(|expr| &expr.node),
                Some(ExprKind::Int(320))
            ));
        }
        other => panic!("Expected image element, got {other:?}"),
    }
}

#[test]
fn test_parse_table_element() {
    let elements = document_elements(
        "document { @table(class=\"report\")[| @text[Name] | @text[Score] ||---|---| | @text[Alice] | @text[99] |] }",
    );
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Table(table) => {
            assert_eq!(table.table.len(), 2);
            assert_eq!(table.table[0].len(), 2);
            assert_eq!(table.table[1].len(), 2);
            let attrs = table
                .attributes
                .as_ref()
                .expect("Expected table attributes");
            assert!(
                matches!(attrs.get("class").map(|expr| &expr.node), Some(ExprKind::StringLiteral(value)) if value == "report")
            );
            assert!(matches!(table.table[0][0].node, DocElemKind::Text(_)));
            assert!(matches!(table.table[1][1].node, DocElemKind::Text(_)));
        }
        other => panic!("Expected table element, got {other:?}"),
    }
}

#[test]
fn test_parse_table_rejects_mismatched_columns() {
    let errors = parse_errors(
        "document { @table[| @text[Name] | @text[Score] ||---|---| | @text[Alice] |] }",
    );
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message(), "Expected 2 columns, got 1");
}

#[test]
fn test_parse_list_element() {
    let elements =
        document_elements("document { @list(class=\"bullets\")[- @text[First] - @text[Second]] }");
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::List(list) => {
            assert_eq!(list.items.len(), 2);
            assert!(!list.numbered);
            let attrs = list.attributes.as_ref().expect("Expected list attributes");
            assert!(
                matches!(attrs.get("class").map(|expr| &expr.node), Some(ExprKind::StringLiteral(value)) if value == "bullets")
            );
            assert!(matches!(list.items[0].node, DocElemKind::Text(_)));
            assert!(matches!(list.items[1].node, DocElemKind::Text(_)));
        }
        other => panic!("Expected list element, got {other:?}"),
    }
}

#[test]
fn test_parse_function_call_no_args() {
    let elements = document_elements("document { @greet() }");
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Call(call) => {
            assert_eq!(call.name, "greet");
            assert!(call.args.is_empty());
            assert!(call.children.is_none());
        }
        other => panic!("Expected call element, got {other:?}"),
    }
}

#[test]
fn test_parse_function_call_with_args() {
    let elements = document_elements(r#"document { @print("hello", name, 3) }"#);
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Call(call) => {
            assert_eq!(call.name, "print");
            assert_eq!(call.args.len(), 3);
            assert!(matches!(call.args[0].ty, Type::String));
            assert_eq!(call.args[0].name, "hello");
            assert!(matches!(call.args[1].ty, Type::Var));
            assert_eq!(call.args[1].name, "name");
            assert!(matches!(call.args[2].ty, Type::Int));
            assert_eq!(call.args[2].name, "3");
        }
        other => panic!("Expected call element, got {other:?}"),
    }
}

#[test]
fn test_parse_function_call_with_children() {
    let elements = document_elements("document { @card(title)[@text[One], @text[Two]] }");
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Call(call) => {
            assert_eq!(call.name, "card");
            assert_eq!(call.args.len(), 1);
            assert!(matches!(call.args[0].ty, Type::Var));
            assert_eq!(call.args[0].name, "title");
            let children = call.children.as_ref().expect("Expected child elements");
            assert_eq!(children.len(), 2);
            assert!(matches!(children[0].node, DocElemKind::Text(_)));
            assert!(matches!(children[1].node, DocElemKind::Text(_)));
        }
        other => panic!("Expected call element, got {other:?}"),
    }
}

#[test]
fn test_parse_link_element() {
    let elements = document_elements(
        r#"document { @link(kind="external")["https://example.com", "Example"] }"#,
    );
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Link(link) => {
            assert_eq!(link.href, "https://example.com");
            assert_eq!(link.content, "Example");
            let attrs = link.attributes.as_ref().expect("Expected link attributes");
            assert!(
                matches!(attrs.get("kind").map(|expr| &expr.node), Some(ExprKind::StringLiteral(value)) if value == "external")
            );
        }
        other => panic!("Expected link element, got {other:?}"),
    }
}

#[test]
fn test_parse_section_element() {
    let elements = document_elements(r#"document { @section(id="main")[@text[Inner]] }"#);
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Section(section) => {
            assert_eq!(section.elements.len(), 1);
            let attrs = section
                .attributes
                .as_ref()
                .expect("Expected section attributes");
            assert!(
                matches!(attrs.get("id").map(|expr| &expr.node), Some(ExprKind::StringLiteral(value)) if value == "main")
            );
            assert!(matches!(section.elements[0].node, DocElemKind::Text(_)));
        }
        other => panic!("Expected section element, got {other:?}"),
    }
}

#[test]
fn test_parse_children_element() {
    let elements = document_elements("document { @children }");
    assert_eq!(elements.len(), 1);

    match &elements[0].node {
        DocElemKind::Children(children) => assert!(children.render_childen),
        other => panic!("Expected children element, got {other:?}"),
    }
}
