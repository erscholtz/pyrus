use std::fmt;

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// error in the code (e.g. a syntax error)
    Error,
    /// bad form in the code (e.g. a unused variable)
    Warning,
    /// no error/warning but something that could be improved (e.g. spelling)
    Note,
    /// error the compiler cannot recover from
    Fatal,
}

/// A location span in the source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub file: String,
}

impl Span {
    pub fn new(start: usize, end: usize, file: impl Into<String>) -> Self {
        Self {
            start,
            end,
            file: file.into(),
        }
    }

    /// Create a simple point span (for single-location errors)
    pub fn point(pos: usize, file: impl Into<String>) -> Self {
        Self {
            start: pos,
            end: pos,
            file: file.into(),
        }
    }
}

/// A simpler location for line/column-based errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file: String,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize, file: impl Into<String>) -> Self {
        Self {
            line,
            column,
            file: file.into(),
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: SourceLocation,
    pub span: Option<Span>,
    pub help: Option<String>,
}

impl Diagnostic {
    /// Whether compilation can continue after this error
    pub fn recoverable(&self) -> bool {
        !matches!(self.severity, Severity::Fatal)
    }

    /// Display implementation for any Diagnostic
    pub fn format(&self) -> String {
        let severity_str = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Fatal => "fatal error",
        };

        let location = &self.location;
        let mut result =
            format!("{} at {}: {}", severity_str, location, self.message);

        if let Some(help) = &self.help {
            result.push_str(&format!("\n  help: {}", help));
        }

        result
    }
}
