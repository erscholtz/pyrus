use std::env;
use std::ffi::OsString;
use std::fs;
use std::time::Instant;

use pyrus::{
    ast::Ast,
    backend,
    diagnostic::DiagnosticManager,
    hir::{self, hir_debug::HirDisplayExt},
    layout::setup_layout,
    lexer::{self, Lexer},
    parser::Parser,
};

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
        args[1].to_str().unwrap_or("resume.ink")
    } else {
        "resume.ink"
    };
    let data =
        fs::read_to_string(filename).expect("Should be able to read test file");

    let mut dm = DiagnosticManager::default();
    let mut lexer = Lexer::new(filename.to_string(), data);
    let tokens = match lexer.lex_all() {
        Ok(tokens) => tokens,
        Err(errors) => {
            for error in errors {
                println!("Lexing error: {}", error);
            }
            return;
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = parser.parse::<Ast>().unwrap();
    parser.gather_errors(&mut dm);

    let hir_module =
        hir::lower(&ast).expect("Should be able to lower AST to HIR");

    let layout = setup_layout(&hir_module);

    // Compute document flow layout (simple vertical stacking)
    let computed_layouts = layout.compute_document_flow(&hir_module);

    // Render to PDF using backend
    let backend = backend::Backend::new(backend::Renderer::Pdf);
    if let Err(e) = backend.render(hir_module, &layout, &computed_layouts) {
        eprintln!("Failed to render PDF: {}", e);
    } else {
        println!("\nPDF rendered successfully to generated/output.pdf");
    }

    let now = Instant::now();
    let time = now - last;
    println!("\nTime taken: {:?}", time);
}
