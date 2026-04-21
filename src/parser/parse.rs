mod elem;
mod expr;
mod func;
mod root;
mod stmt;
mod style;

use crate::{
    diagnostic::{DiagnosticManager, SyntaxError},
    parser::Parser,
};

/// A trait for parsing a value from a token
pub trait Parse: Sized {
    fn parse(p: &mut Parser) -> Result<Self, SyntaxError>;
}
