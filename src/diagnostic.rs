pub mod diagnostic;
pub mod diagnostic_manager;

pub use diagnostic::Diagnostic;
pub use diagnostic::Severity;
pub use diagnostic::SourceLocation;
pub use diagnostic::Span;
pub use diagnostic_manager::DiagnosticManager;
