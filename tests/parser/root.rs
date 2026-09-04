use pyrus::{
    ast::{Ast, DocumentConfig, Item},
    lexer::Lexer,
    parser::{Parse, Parser},
};

#[test]
fn parses_top_level_items_in_source_order() {
    let source = r#"
document {}

elem marker {}

layout marker {}

@marker {}
"#;
    let file = "top-level.pyr".to_string();
    let lexer = Lexer::new(file.clone(), source.to_string());
    let mut parser =
        Parser::new(file, lexer).expect("parser should read its first token");

    let ast = Ast::parse(&mut parser).expect("top-level items should parse");

    assert_eq!(ast.items.len(), 4);
    assert!(matches!(&ast.items[0].node, Item::Document(_)));
    assert!(matches!(&ast.items[1].node, Item::ElemDecl(_)));
    assert!(matches!(&ast.items[2].node, Item::LayoutDecl(_)));
    assert!(matches!(&ast.items[3].node, Item::ElemInvoke(_)));
}

#[test]
fn parses_document_config() {
    let source = r#"
document {
    type: A4
    margin: 4
}
"#;
    let file = "document-config.pyr".to_string();
    let lexer = Lexer::new(file.clone(), source.to_string());
    let mut parser =
        Parser::new(file, lexer).expect("parser should read its first token");

    let ast = Ast::parse(&mut parser).expect("document config should parse");

    assert_eq!(ast.items.len(), 1);
    assert!(matches!(&ast.items[0].node, Item::Document(_)));

    let Item::Document(document) = &ast.items[0].node else {
        panic!("expected document configuration");
    };

    assert_eq!(document.entries.len(), 2);
    assert_eq!(document.entries[0].name.text, "type");
    assert_eq!(document.entries[0].value.node, "A4");
    assert_eq!(document.entries[1].name.text, "margin");
    assert_eq!(document.entries[1].value.node, "4");
}
