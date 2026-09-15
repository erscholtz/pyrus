use std::{env, ffi::OsString, fs};

use pyrus::{ast::Ast, parser::Parser};

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
    println!("{:#?}", ast);
}
