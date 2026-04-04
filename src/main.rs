use std::env;
use std::ffi::OsString;
use std::fs;
use std::time::Instant;

use pyrus::backend;
use pyrus::hir;
use pyrus::hir::HirDisplayExt;
use pyrus::hir::resolve_styles;
use pyrus::layout::setup_layout;
use pyrus::lexer;
use pyrus::parser;

fn main() {
    let last = Instant::now();
    let args: Vec<OsString> = env::args_os().collect();

    println!("All args: {:?}", args);

    if args.len() > 1 {
        let first_arg = &args[1];
        println!("First argument: {:?}", first_arg);
    } else {
        println!("No arguments provided!");
    }

    let filename = if args.len() > 1 {
        args[1].to_str().unwrap_or("ideas.ink")
    } else {
        "ideas.ink"
    };
    let data = fs::read_to_string(filename).expect("Should be able to read test file");

    let tokens = match lexer::lex(&data, filename) {
        Ok(tokens) => tokens,
        Err(errors) => {
            for error in errors {
                println!("Lexing error: {}", error.message);
            }
            return;
        }
    };
    println!("{:?}", &tokens);

    let ast = parser::parse(tokens).expect("Should be able to parse tokens to AST");
    println!("{:#?}", ast);

    let mut hlir_module = hir::lower(&ast);
    println!("{}", hlir_module.hir_display());
    // println!("HLIR before style resolution:");
    // println!("  Elements: {}", hlir_module.elements.len());
    // println!("  CSS Rules: {}", hlir_module.css_rules.len());
    // println!("  Element Metadata: {}", hlir_module.element_metadata.len());

    // Run CSS style resolution
    resolve_styles(&mut hlir_module);

    // println!("\n=== Computed Styles ===");
    // for (idx, metadata) in hlir_module.element_metadata.iter().enumerate() {
    //     if let Some(node) = hlir_module.attributes.find_node(metadata.attributes_ref) {
    //         println!(
    //             "\nElement {} (type: {:?}, id: {:?}, classes: {:?}):",
    //             idx, metadata.element_type, metadata.id, metadata.classes
    //         );
    //         println!(
    //             "  Inline: margin={:?}, padding={:?}, align={:?}",
    //             node.inline.margin, node.inline.padding, node.inline.align
    //         );
    //         println!(
    //             "  Computed: margin={:?}, padding={:?}, align={:?}, hidden={}",
    //             node.computed.margin,
    //             node.computed.padding,
    //             node.computed.align,
    //             node.computed.hidden
    //         );
    //         println!("  Style map: {:?}", node.computed.style);
    //     }
    // }

    let layout = setup_layout(&hlir_module);

    // Compute document flow layout (simple vertical stacking)
    let computed_layouts = layout.compute_document_flow(&hlir_module);

    // Print computed layouts for each element
    println!("\n=== Computed Layouts ===");
    for computed in &computed_layouts {
        if let Some(metadata) = hlir_module.element_metadata.get(computed.element_index) {
            println!(
                "Element {} (type: {:?}, id: {:?}): x={:.1}, y={:.1}, w={:.1}, h={:.1}",
                computed.element_index,
                metadata.element_type,
                metadata.id,
                computed.x,
                computed.y,
                computed.width,
                computed.height
            );
        }
    }

    // Render to PDF using backend
    let backend = backend::Backend::new(backend::Renderer::Pdf);
    if let Err(e) = backend.render(hlir_module, &layout, &computed_layouts) {
        eprintln!("Failed to render PDF: {}", e);
    } else {
        println!("\nPDF rendered successfully to generated/output.pdf");
    }

    let now = Instant::now();
    let time = now - last;
    println!("\nTime taken: {:?}", time);
}
