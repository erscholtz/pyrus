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

    pub fn message(&self) -> &str {
        &self.error_message
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

// Implement the Diagnostic trait for unified error handling
impl crate::error::Diagnostic for ParseError {
    fn message(&self) -> &str {
        &self.error_message
    }

    fn location(&self) -> crate::error::SourceLocation {
        crate::error::SourceLocation::new(self.line, self.column)
    }

    fn severity(&self) -> crate::error::Severity {
        crate::error::Severity::Error
    }

    fn recoverable(&self) -> bool {
        // Parser errors can be recoverable in many cases (e.g., skip to next statement)
        true
    }
}
