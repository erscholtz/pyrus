use super::{Diagnostic, Severity, SourceLocation};

/// Non-fatal diagnostics that highlight likely mistakes or questionable code.
#[derive(Debug, Clone)]
pub enum Warning {
    /// Free-form warning when a more specific variant is not needed.
    General {
        location: SourceLocation,
        message: String,
        help: Option<String>,
    },

    /// A declared variable is never used.
    UnusedVariable {
        location: SourceLocation,
        name: String,
    },

    /// A declared function is never called.
    UnusedFunction {
        location: SourceLocation,
        name: String,
    },

    /// Syntax is still accepted but should be replaced.
    DeprecatedSyntax {
        location: SourceLocation,
        feature: String,
        replacement: Option<String>,
    },

    /// Control flow makes subsequent code impossible to reach.
    UnreachableCode {
        location: SourceLocation,
        reason: Option<String>,
    },

    /// The same style property is specified redundantly.
    RedundantStyleProperty {
        location: SourceLocation,
        property: String,
    },
}

impl Warning {
    pub fn general(location: SourceLocation, message: impl Into<String>) -> Self {
        Self::General {
            location,
            message: message.into(),
            help: None,
        }
    }

    pub fn with_help(
        location: SourceLocation,
        message: impl Into<String>,
        help: impl Into<String>,
    ) -> Self {
        Self::General {
            location,
            message: message.into(),
            help: Some(help.into()),
        }
    }

    pub fn unused_variable(name: impl Into<String>, location: SourceLocation) -> Self {
        Self::UnusedVariable {
            location,
            name: name.into(),
        }
    }

    pub fn unused_function(name: impl Into<String>, location: SourceLocation) -> Self {
        Self::UnusedFunction {
            location,
            name: name.into(),
        }
    }

    pub fn deprecated_syntax(
        feature: impl Into<String>,
        replacement: Option<String>,
        location: SourceLocation,
    ) -> Self {
        Self::DeprecatedSyntax {
            location,
            feature: feature.into(),
            replacement,
        }
    }

    pub fn unreachable_code(reason: Option<String>, location: SourceLocation) -> Self {
        Self::UnreachableCode { location, reason }
    }

    pub fn redundant_style_property(
        property: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::RedundantStyleProperty {
            location,
            property: property.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Warning::General { .. } => "W0001",
            Warning::UnusedVariable { .. } => "W0002",
            Warning::UnusedFunction { .. } => "W0003",
            Warning::DeprecatedSyntax { .. } => "W0004",
            Warning::UnreachableCode { .. } => "W0005",
            Warning::RedundantStyleProperty { .. } => "W0006",
        }
    }

    fn detailed_message(&self) -> String {
        match self {
            Warning::General { message, .. } => message.clone(),
            Warning::UnusedVariable { name, .. } => {
                format!("`{name}` is never used")
            }
            Warning::UnusedFunction { name, .. } => {
                format!("function `{name}` is never used")
            }
            Warning::DeprecatedSyntax {
                feature,
                replacement,
                ..
            } => {
                match replacement {
                    Some(replacement) => {
                        format!("`{feature}` is deprecated; use `{replacement}` instead")
                    }
                    None => format!("`{feature}` is deprecated"),
                }
            }
            Warning::UnreachableCode { reason, .. } => match reason {
                Some(reason) => format!("code is unreachable: {reason}"),
                None => "code is unreachable".to_string(),
            },
            Warning::RedundantStyleProperty { property, .. } => {
                format!("style property `{property}` is set redundantly")
            }
        }
    }
}

impl Diagnostic for Warning {
    fn message(&self) -> &str {
        match self {
            Warning::General { message, .. } => message,
            Warning::UnusedVariable { .. } => "unused variable",
            Warning::UnusedFunction { .. } => "unused function",
            Warning::DeprecatedSyntax { .. } => "deprecated syntax",
            Warning::UnreachableCode { .. } => "unreachable code",
            Warning::RedundantStyleProperty { .. } => "redundant style property",
        }
    }

    fn location(&self) -> SourceLocation {
        match self {
            Warning::General { location, .. } => location.clone(),
            Warning::UnusedVariable { location, .. } => location.clone(),
            Warning::UnusedFunction { location, .. } => location.clone(),
            Warning::DeprecatedSyntax { location, .. } => location.clone(),
            Warning::UnreachableCode { location, .. } => location.clone(),
            Warning::RedundantStyleProperty { location, .. } => location.clone(),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn help(&self) -> Option<&str> {
        match self {
            Warning::General { help, .. } => help.as_deref(),
            _ => None,
        }
    }
}

impl std::fmt::Display for Warning {
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

impl std::error::Error for Warning {}
