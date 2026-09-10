use crate::parser::tokens::TokenKind;

use super::{
    Diagnostic, FatalError, Note, SemanticError, Severity, SourceLocation,
    Span, SyntaxError, Warning,
};

#[derive(Debug, Clone)]
pub enum CompilerDiagnostic {
    Syntax(SyntaxError),
    Warning(Warning),
    Semantic(SemanticError),
    Fatal(FatalError),
    Note(Note),
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
            CompilerDiagnostic::Fatal(diagnostic) => diagnostic.message(),
            CompilerDiagnostic::Warning(diagnostic) => diagnostic.message(),
            CompilerDiagnostic::Note(diagnostic) => diagnostic.message(),
        }
    }

    fn location(&self) -> SourceLocation {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.location(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.location(),
            CompilerDiagnostic::Fatal(diagnostic) => diagnostic.location(),
            CompilerDiagnostic::Warning(diagnostic) => diagnostic.location(),
            CompilerDiagnostic::Note(diagnostic) => diagnostic.location(),
        }
    }

    fn severity(&self) -> Severity {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.severity(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.severity(),
            CompilerDiagnostic::Fatal(diagnostic) => diagnostic.severity(),
            CompilerDiagnostic::Warning(diagnostic) => diagnostic.severity(),
            CompilerDiagnostic::Note(diagnostic) => diagnostic.severity(),
        }
    }

    fn recoverable(&self) -> bool {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.recoverable(),
            CompilerDiagnostic::Semantic(diagnostic) => {
                diagnostic.recoverable()
            }
            CompilerDiagnostic::Fatal(_) => false,
            CompilerDiagnostic::Warning(_) => true,
            CompilerDiagnostic::Note(_) => true,
        }
    }

    fn span(&self) -> Option<&Span> {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.span(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.span(),
            CompilerDiagnostic::Fatal(_) => None,
            CompilerDiagnostic::Warning(_) => None,
            CompilerDiagnostic::Note(_) => None,
        }
    }

    fn help(&self) -> Option<&str> {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.help(),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.help(),
            CompilerDiagnostic::Fatal(_) => None,
            CompilerDiagnostic::Warning(_) => None,
            CompilerDiagnostic::Note(_) => None,
        }
    }
}

impl std::fmt::Display for CompilerDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerDiagnostic::Syntax(diagnostic) => diagnostic.fmt(f),
            CompilerDiagnostic::Semantic(diagnostic) => diagnostic.fmt(f),
            CompilerDiagnostic::Fatal(diagnostic) => diagnostic.fmt(f),
            CompilerDiagnostic::Warning(diagnostic) => diagnostic.fmt(f),
            CompilerDiagnostic::Note(diagnostic) => diagnostic.fmt(f),
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

    pub fn extend<D, I>(&mut self, diagnostics: I) -> &mut Self
    where
        D: Into<CompilerDiagnostic>,
        I: IntoIterator<Item = D>,
    {
        self.diagnostics
            .extend(diagnostics.into_iter().map(Into::into));
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
        self.diagnostics.iter().any(|diagnostic| {
            matches!(diagnostic.severity(), Severity::Error | Severity::Fatal)
        })
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
