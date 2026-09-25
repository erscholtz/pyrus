use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use pyrus::ast::Ast;
use pyrus::hir::lower;
use pyrus::layout::layout;
use pyrus::parser::Parser;
use pyrus::render::Renderer;

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    let filename = args
        .get(1)
        .and_then(|arg| arg.to_str())
        .unwrap_or("temp.pyr");
    let source = fs::read_to_string(filename)
        .expect("source file should be readable UTF-8");
    let mut parser = match Parser::new(filename.to_string(), source) {
        Ok(parser) => parser,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };
    let ast = match parser.parse::<Ast>() {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };

    let hir = match lower(&ast) {
        Ok(hir) => hir,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };

    let layout = match layout(&hir) {
        Ok(layout) => layout,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };

    Renderer::render_pdf(&layout.page, "test.pdf");
}
