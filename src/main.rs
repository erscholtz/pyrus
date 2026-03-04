use std::env;
use std::ffi::OsString;
use std::fs;
use std::time::Instant;

use pyrus::hlir;
use pyrus::hlir::resolve_styles;
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

    let data = fs::read_to_string("temp.ink").expect("Should be able to read test file");

    let tokens = lexer::lex(&data);
    println!("{:?}", &tokens);

    let ast = parser::parse(tokens);
    println!("{:#?}", ast);

    let mut hlir_module = hlir::lower(&ast);
    println!("{:#?}", hlir_module);
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

    let mut layout = setup_layout(&hlir_module);

    // A4 page size in points (1 inch = 72 points)
    let page_width = 595.0;
    let page_height = 842.0;
    layout.compute_layout(page_width, page_height);

    // Print computed layouts for each element
    println!("\n=== Computed Layouts ===");
    for (idx, metadata) in hlir_module.element_metadata.iter().enumerate() {
        if let Some(computed) = layout.get_element_layout(idx) {
            println!(
                "Element {} (type: {:?}, id: {:?}): x={:.1}, y={:.1}, w={:.1}, h={:.1}",
                idx,
                metadata.element_type,
                metadata.id,
                computed.x,
                computed.y,
                computed.width,
                computed.height
            );
        }
    }

    let now = Instant::now();
    let time = now - last;
    println!("\nTime taken: {:?}", time);
}
