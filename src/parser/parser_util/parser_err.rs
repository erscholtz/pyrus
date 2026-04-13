use crate::diagnostic::SourceLocation;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    error_message: String,
    source: SourceLocation,
}

impl ParseError {
    pub fn new(error_message: String, source: SourceLocation) -> Self {
        Self {
            error_message,
            source,
        }
    }

    pub fn message(&self) -> &str {
        &self.error_message
    }

    pub fn line(&self) -> usize {
        self.source.line
    }

    pub fn column(&self) -> usize {
        self.source.column
    }

    pub fn file(&self) -> &str {
        &self.source.file
    }
}

// Implement the Diagnostic trait for unified error handling
impl crate::diagnostic::Diagnostic for ParseError {
    fn message(&self) -> &str {
        &self.error_message
    }

    fn location(&self) -> crate::diagnostic::SourceLocation {
        self.source.clone()
    }

    fn severity(&self) -> crate::diagnostic::Severity {
        crate::diagnostic::Severity::Error
    }

    fn recoverable(&self) -> bool {
        // Parser errors can be recoverable in many cases (e.g., skip to next statement)
        true
    }
}
