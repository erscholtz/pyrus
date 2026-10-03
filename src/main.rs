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
    let start = std::time::Instant::now();
    let args: Vec<OsString> = env::args_os().collect();
    let filename = args
        .get(1)
        .and_then(|arg| arg.to_str())
        .unwrap_or("temp.pyr");
    let source = fs::read_to_string(filename)
        .expect("source file should be readable UTF-8");
    let read = std::time::Instant::now() - start;
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
    let parse = std::time::Instant::now() - start - read;
    let hir = match lower(&ast) {
        Ok(hir) => hir,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };
    let lower = std::time::Instant::now() - start - read - parse;
    let layout = match layout(&hir) {
        Ok(layout) => layout,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };
    let layout = std::time::Instant::now() - start - read - parse - lower;

    println!(
        "{r:width$}{0:#?} \n{p:width$}{1:#?} \n{l:width$}{2:#?} \n{t:width$}{3:#?} \n{rd:width$}??",
        read,
        parse,
        lower,
        layout,
        r = "read:",
        p = "parse:",
        l = "lower:",
        t = "layout:",
        rd = "render:",
        width = 16
    )
    // Renderer::render_pdf(&layout.page, "test.pdf");
}
