use std::{env, ffi::OsString, fs};

use pyrus::{lexer::Lexer, tokens::TokenKind};

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    let filename = args
        .get(1)
        .and_then(|arg| arg.to_str())
        .unwrap_or("temp.pyr");
    let source = fs::read_to_string(filename)
        .expect("source file should be readable UTF-8");
    let mut lexer = Lexer::new(filename.to_string(), source);

    loop {
        match lexer.pull() {
            Ok(token) => {
                let text = lexer.text(&token).unwrap_or("");
                println!(
                    "{:?} {}:{} {:?} {:?}",
                    token.kind, token.line, token.col, token.range, text
                );
                if token.kind == TokenKind::Eof {
                    break;
                }
            }
            Err(error) => {
                eprintln!("Lexing error: {error}");
                break;
            }
        }
    }
}
