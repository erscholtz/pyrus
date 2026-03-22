#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    error_message: String,
    line: usize,
    column: usize,
}

impl ParseError {
    pub fn new(error_message: String, line: usize, column: usize) -> Self {
        Self {
            error_message,
            line,
            column,
        }
    }
}
