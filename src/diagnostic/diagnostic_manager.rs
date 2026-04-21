use crate::lexer::TokenKind;

use super::{Diagnostic, SemanticError, Severity, SourceLocation, Span, SyntaxError};

#[derive(Debug, Clone)]
pub enum CompilerDiagnostic {
    Syntax(SyntaxError),
    // Warning(Warning),
    Semantic(SemanticError),
    // Fatal(FatalError),
    // Note(Note),
}

impl From<SyntaxError> for CompilerDiagnostic {
    fn from(value: SyntaxError) -> Self {
        Self::Syntax(value)
    }
}

impl From<SemanticError> for CompilerDiagnostic {
    fn from(value: SemanticError) -> Self {
        Self::Semantic(value)
    }
}

impl Diagnostic for CompilerDiagnostic {
    fn message(&self) -> &str {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.message(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.message(),
        }
    }

    fn location(&self) -> SourceLocation {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.location(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.location(),
        }
    }

    fn severity(&self) -> Severity {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.severity(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.severity(),
        }
    }

    fn recoverable(&self) -> bool {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.recoverable(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.recoverable(),
        }
    }

    fn span(&self) -> Option<&Span> {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.span(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.span(),
        }
    }

    fn help(&self) -> Option<&str> {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.help(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.help(),
        }
    }
}

impl std::fmt::Display for CompilerDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.fmt(f),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.fmt(f),
        }
    }
}

impl std::error::Error for CompilerDiagnostic {}

#[derive(Debug, Clone, Default)]
pub struct DiagnosticManager {
    diagnostics: Vec<CompilerDiagnostic>,
}

impl DiagnosticManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<D>(&mut self, diagnostic: D) -> &mut Self
    where
        D: Into<CompilerDiagnostic>,
    {
        self.diagnostics.push(diagnostic.into());
        self
    }

    pub fn diagnostics(&self) -> &[CompilerDiagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<CompilerDiagnostic> {
        self.diagnostics
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic.severity(), Severity::Error | Severity::Fatal))
    }

    pub fn has_fatal(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == Severity::Fatal)
    }

    pub fn syntax_error(
        &mut self,
        expected: Vec<TokenKind>,
        found: TokenKind,
        location: SourceLocation,
    ) -> &mut Self {
        self.push(SyntaxError::unexpected_token(expected, found, location))
    }

    pub fn semantic(&mut self, diagnostic: SemanticError) -> &mut Self {
        self.push(diagnostic)
    }
}
