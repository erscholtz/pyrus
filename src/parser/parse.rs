mod elem;
mod expr;
mod func;
mod root;
mod stmt;
mod style;

use crate::parser::{parser::Parser, parser_err::ParseError};

/// A trait for parsing a value from a token
pub trait Parse: Sized {
    fn parse(p: &mut Parser) -> Result<Self, ParseError>;
    fn try_parse(p: &mut Parser) -> Option<Self>;
}
