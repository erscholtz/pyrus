pub mod diagnostic;
pub mod diagnostic_manager;
// pub mod fatal;
// pub mod note;
pub mod semantic;
pub mod syntax;
// pub mod warning;

pub use diagnostic::{Diagnostic, Severity, SourceLocation, Span, format_diagnostic};
pub use diagnostic_manager::{CompilerDiagnostic, DiagnosticManager};
// pub use fatal::FatalError;
// pub use note::Note;
pub use semantic::SemanticError;
pub use syntax::SyntaxError;
// pub use warning::Warning;
