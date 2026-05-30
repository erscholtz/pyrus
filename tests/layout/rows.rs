use pyrus::ast::Ast;
use pyrus::hir::{
    hir_types::{HIRModule, HirElementOp},
    lower,
};
use pyrus::layout::setup_layout;
use pyrus::lexer::{TokenStream, lex};
use pyrus::parser::Parser;

fn parse(tokens: TokenStream) -> Result<Ast, Vec<pyrus::diagnostic::SyntaxError>> {
    Parser::new(tokens).parse::<Ast>()
}

fn lower_ast(ast: &Ast) -> HIRModule {
    lower(ast).unwrap_or_else(|errors| panic!("Lowering failed: {errors:?}"))
}

#[test]
fn test_document_flow_row_uses_gap_and_nowrap_side_metadata() {
    let source = r#"
document {
    @section(class="row")[
        @text(class="title")[Project Title]
        @text(class="side")[September 2025 - Present]
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 20pt;
    }

    .side {
        width: 130pt;
        white-space: nowrap;
        text-align: right;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_uses_gap_and_nowrap_side_metadata",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let side_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"side".to_string())
        })
        .expect("Side metadata should have layout");

    assert!(side_layout.nowrap);
    assert!(side_layout.x > title_layout.x + title_layout.width);
    assert!((side_layout.x - (title_layout.x + title_layout.width)) >= 19.9);
    assert!((side_layout.box_x + side_layout.box_width - 595.0).abs() < 0.001);
    assert!(side_layout.x > side_layout.box_x);
    assert!(side_layout.x + side_layout.width <= side_layout.box_x + side_layout.box_width);
}

#[test]
fn test_document_flow_row_lays_out_wrapped_link_component_on_right() {
    let source = r#"
template {
    func side_link(url: String) {
        return @link(class="badge")["${url}", "GitHub"]
    }
}

document {
    @section(class="row")[
        @text(class="title")[Project Title]
        @side_link("github.com/example/project")
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 10pt;
    }

    .badge {
        white-space: nowrap;
        padding-top: 1pt;
        padding-right: 3pt;
        padding-bottom: 1pt;
        padding-left: 3pt;
        border: "0.55pt solid #1f4e79";
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_lays_out_wrapped_link_component_on_right",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let badge_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"badge".to_string())
        })
        .expect("Wrapped badge link should have layout");

    assert!(badge_layout.nowrap);
    assert!(badge_layout.box_width > badge_layout.width);
    assert!(badge_layout.x > title_layout.x + title_layout.width);
    assert!(title_layout.x + title_layout.width <= badge_layout.box_x - 9.9);
}

#[test]
fn test_document_flow_row_lays_out_grouped_project_links_on_right() {
    let source = r#"
template {
    func project_link(url: String, label: String) {
        return @link(class="project_link")["${url}", "${label}"]
    }

    func project_links(paper_url: String, github_url: String, demo_url: String) {
        return @section(class="project_links")[
            @project_link("${paper_url}", "[paper")
            @text(class="project_link_sep")[|]
            @project_link("${github_url}", "github")
            @text(class="project_link_sep")[|]
            @project_link("${demo_url}", "demo]")
        ]
    }
}

document {
    @section(class="row")[
        @text(class="title")[Project Title]
        @project_links(
            "example.com/paper",
            "github.com/example/project",
            "example.com/demo"
        )
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 10pt;
    }

    .project_links {
        width: 130pt;
        display: flex;
        flex-direction: row;
        justify-content: flex-end;
        column-gap: 3pt;
        white-space: nowrap;
    }

    .project_link,
    .project_link_sep {
        white-space: nowrap;
        font-size: 8pt;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_lays_out_grouped_project_links_on_right",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let link_layout = |content: &str| {
        computed
            .iter()
            .find(|layout| {
                matches!(
                    &hlir.elements[layout.element_index],
                    HirElementOp::Link { content: link_content, .. } if link_content == content
                )
            })
            .expect("Project link should have layout")
    };

    let paper_layout = link_layout("[paper");
    let github_layout = link_layout("github");
    let demo_layout = link_layout("demo]");

    assert!(paper_layout.nowrap);
    assert!(github_layout.nowrap);
    assert!(demo_layout.nowrap);
    assert!(paper_layout.x > title_layout.x + title_layout.width);
    assert!(github_layout.x > paper_layout.x + paper_layout.width);
    assert!(demo_layout.x > github_layout.x + github_layout.width);
    assert!((demo_layout.box_x + demo_layout.box_width - 595.0).abs() < 0.001);
}

#[test]
fn test_document_flow_row_wraps_left_before_nowrap_side_metadata() {
    let source = r#"
document {
    @section(class="row")[
        @text(class="title")[This is a deliberately long project title that needs to wrap before it reaches the right aligned metadata and it keeps going with more descriptive project words to force another line]
        @text(class="side")[GitHub]
    ]
}
style {
    body {
        font-size: 10pt;
    }

    .row {
        display: flex;
        flex-direction: row;
        column-gap: 10pt;
    }

    .side {
        width: 70pt;
        white-space: nowrap;
        text-align: right;
    }
}
"#;
    let tokens = lex(
        source,
        "test_document_flow_row_wraps_left_before_nowrap_side_metadata",
    )
    .expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    let hlir = lower_ast(&ast);
    let layout = setup_layout(&hlir);
    let computed = layout.compute_document_flow(&hlir);

    let title_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"title".to_string())
        })
        .expect("Title should have layout");
    let side_layout = computed
        .iter()
        .find(|layout| {
            hlir.element_metadata[layout.element_index]
                .classes
                .contains(&"side".to_string())
        })
        .expect("Side metadata should have layout");

    assert!(title_layout.height > 12.0);
    assert!(side_layout.nowrap);
    assert!((side_layout.box_x + side_layout.box_width - 595.0).abs() < 0.001);
    assert!(title_layout.x + title_layout.width <= side_layout.box_x - 9.9);
}
