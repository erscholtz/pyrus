use super::{Diagnostic, Severity, SourceLocation};

/// Supplemental context attached to a primary diagnostic.
#[derive(Debug, Clone)]
pub enum Note {
    /// Free-form note message.
    General {
        location: SourceLocation,
        message: String,
    },

    /// Highlights a related source location.
    RelatedLocation {
        location: SourceLocation,
        message: String,
    },

    /// Suggests a concrete follow-up.
    Suggestion {
        location: SourceLocation,
        message: String,
        suggestion: String,
    },
}

impl Note {
    pub fn general(location: SourceLocation, message: impl Into<String>) -> Self {
        Self::General {
            location,
            message: message.into(),
        }
    }

    pub fn related_location(location: SourceLocation, message: impl Into<String>) -> Self {
        Self::RelatedLocation {
            location,
            message: message.into(),
        }
    }

    pub fn suggestion(
        location: SourceLocation,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::Suggestion {
            location,
            message: message.into(),
            suggestion: suggestion.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Note::General { .. } => "N0001",
            Note::RelatedLocation { .. } => "N0002",
            Note::Suggestion { .. } => "N0003",
        }
    }

    fn detailed_message(&self) -> String {
        match self {
            Note::General { message, .. } => message.clone(),
            Note::RelatedLocation { message, .. } => message.clone(),
            Note::Suggestion {
                message,
                suggestion,
                ..
            } => {
                format!("{message} Suggestion: {suggestion}")
            }
        }
    }
}

impl Diagnostic for Note {
    fn message(&self) -> &str {
        match self {
            Note::General { message, .. } => message,
            Note::RelatedLocation { message, .. } => message,
            Note::Suggestion { message, .. } => message,
        }
    }

    fn location(&self) -> SourceLocation {
        match self {
            Note::General { location, .. } => location.clone(),
            Note::RelatedLocation { location, .. } => location.clone(),
            Note::Suggestion { location, .. } => location.clone(),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Note
    }

    fn help(&self) -> Option<&str> {
        match self {
            Note::Suggestion { suggestion, .. } => Some(suggestion),
            _ => None,
        }
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] note at {}: {}",
            self.code(),
            self.location(),
            self.detailed_message()
        )
    }
}

impl std::error::Error for Note {}
