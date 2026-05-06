use super::{Diagnostic, Severity, SourceLocation};

/// Fatal diagnostics stop compilation immediately.
#[derive(Debug, Clone)]
pub enum FatalError {
    /// Free-form fatal error.
    General {
        location: SourceLocation,
        message: String,
    },

    /// The compiler reached an unexpected internal condition.
    InternalCompilerError {
        location: SourceLocation,
        phase: String,
        details: String,
    },

    /// Reading or writing compiler resources failed.
    IoError {
        location: SourceLocation,
        path: String,
        reason: String,
    },

    /// Compiler bookkeeping became inconsistent.
    InvalidCompilerState {
        location: SourceLocation,
        state: String,
    },
}

impl FatalError {
    pub fn general(location: SourceLocation, message: impl Into<String>) -> Self {
        Self::General {
            location,
            message: message.into(),
        }
    }

    pub fn internal_compiler_error(
        phase: impl Into<String>,
        details: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::InternalCompilerError {
            location,
            phase: phase.into(),
            details: details.into(),
        }
    }

    pub fn io_error(
        path: impl Into<String>,
        reason: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::IoError {
            location,
            path: path.into(),
            reason: reason.into(),
        }
    }

    pub fn invalid_compiler_state(
        state: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::InvalidCompilerState {
            location,
            state: state.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            FatalError::General { .. } => "F0001",
            FatalError::InternalCompilerError { .. } => "F0002",
            FatalError::IoError { .. } => "F0003",
            FatalError::InvalidCompilerState { .. } => "F0004",
        }
    }

    fn detailed_message(&self) -> String {
        match self {
            FatalError::General { message, .. } => message.clone(),
            FatalError::InternalCompilerError { phase, details, .. } => {
                format!("internal compiler error in {phase}: {details}")
            }
            FatalError::IoError { path, reason, .. } => {
                format!("I/O error while accessing `{path}`: {reason}")
            }
            FatalError::InvalidCompilerState { state, .. } => {
                format!("compiler entered an invalid state: {state}")
            }
        }
    }
}

impl Diagnostic for FatalError {
    fn message(&self) -> &str {
        match self {
            FatalError::General { message, .. } => message,
            FatalError::InternalCompilerError { .. } => "internal compiler error",
            FatalError::IoError { .. } => "I/O failure",
            FatalError::InvalidCompilerState { .. } => "invalid compiler state",
        }
    }

    fn location(&self) -> SourceLocation {
        match self {
            FatalError::General { location, .. } => location.clone(),
            FatalError::InternalCompilerError { location, .. } => location.clone(),
            FatalError::IoError { location, .. } => location.clone(),
            FatalError::InvalidCompilerState { location, .. } => location.clone(),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Fatal
    }

    fn recoverable(&self) -> bool {
        false
    }
}

impl std::fmt::Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} at {}: {}",
            self.code(),
            self.message(),
            self.location(),
            self.detailed_message()
        )
    }
}

impl std::error::Error for FatalError {}
