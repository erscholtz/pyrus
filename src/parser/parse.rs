mod content;
mod document;
mod elem;
mod layout;
mod root;

use crate::diagnostic::CompilerDiagnostic;
use crate::parser::Parser;

/// Parses one syntax construct from the parser's current token.
pub trait Parse: Sized {
    fn parse(parser: &mut Parser) -> Result<Self, CompilerDiagnostic>;
}
