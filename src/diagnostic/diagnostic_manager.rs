use crate::parser::tokens::TokenKind;

use super::Diagnostic;
use super::FatalError;
use super::Note;
use super::SemanticError;
use super::Severity;
use super::SyntaxError;
use super::Warning;

#[derive(Debug, Clone, Default)]
pub struct DiagnosticManager {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn extend(&mut self, diagnostics: Vec<Diagnostic>) -> &mut Self {
        self.diagnostics.extend(diagnostics.into_iter());
        self
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| {
            matches!(diagnostic.severity, Severity::Error | Severity::Fatal)
        })
    }

    pub fn has_fatal(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Fatal)
    }
}
