use std::fmt;

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Core trait that all compiler diagnostics implement
pub trait Diagnostic: std::fmt::Debug {
    /// The main error message
    fn message(&self) -> &str;

    /// Source location (line:column format)
    fn location(&self) -> SourceLocation;

    /// Severity level
    fn severity(&self) -> Severity;

    /// Whether compilation can continue after this error
    fn recoverable(&self) -> bool {
        !matches!(self.severity(), Severity::Fatal)
    }

    /// Optional: full span information if available
    fn span(&self) -> Option<&Span> {
        None
    }

    /// Optional: help message with suggestions
    fn help(&self) -> Option<&str> {
        None
    }
}

/// Display implementation for any Diagnostic
pub fn format_diagnostic<D: Diagnostic>(diag: &D) -> String {
    let severity_str = match diag.severity() {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
        Severity::Fatal => "fatal error",
    };

    let location = diag.location();
    let mut result = format!("{} at {}: {}", severity_str, location, diag.message());

    if let Some(help) = diag.help() {
        result.push_str(&format!("\n  help: {}", help));
    }

    result
}
