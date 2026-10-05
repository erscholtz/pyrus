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
    let start_time = std::time::Instant::now();
    let args: Vec<OsString> = env::args_os().collect();
    let filename = args
        .get(1)
        .and_then(|arg| arg.to_str())
        .unwrap_or("temp.pyr");
    let source = fs::read_to_string(filename)
        .expect("source file should be readable UTF-8");
    let read_time = std::time::Instant::now() - start_time;
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
    let parse_time = std::time::Instant::now() - start_time - read_time;
    let hir = match lower(&ast) {
        Ok(hir) => hir,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };
    let lower_time =
        std::time::Instant::now() - start_time - read_time - parse_time;
    let layout = match layout(&hir) {
        Ok(layout) => layout,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };
    let layout_time = std::time::Instant::now()
        - start_time
        - read_time
        - parse_time
        - lower_time;

    Renderer::render_pdf(&layout, "test.pdf");
    let render_time = std::time::Instant::now()
        - start_time
        - read_time
        - parse_time
        - lower_time
        - layout_time;

    println!(
        "{r:width$}{0:#?} \n{p:width$}{1:#?} \n{l:width$}{2:#?} \n{t:width$}{3:#?} \n{rd:width$}{4:#?}",
        read_time,
        parse_time,
        lower_time,
        layout_time,
        render_time,
        r = "read:",
        p = "parse:",
        l = "lower:",
        t = "layout:",
        rd = "render:",
        width = 16
    )
}
