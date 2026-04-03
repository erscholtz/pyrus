use crate::error::{Diagnostic, Severity, SourceLocation, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct HirError {
    message: String,
    severity: Severity,
    location: SourceLocation,
    span: Span,
}

impl HirError {
    pub fn new(message: String, severity: Severity, location: SourceLocation, span: Span) -> Self {
        Self {
            message,
            severity,
            location,
            span,
        }
    }
}

impl Diagnostic for HirError {
    fn message(&self) -> &str {
        &self.message
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn location(&self) -> SourceLocation {
        self.location.clone()
    }

    fn span(&self) -> Option<&Span> {
        Some(&self.span)
    }
}
