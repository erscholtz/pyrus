use super::{Diagnostic, Severity, SourceLocation, Span};

/// Semantic errors detected during HLIR validation and type checking
#[derive(Debug, Clone)]
pub enum SemanticError {
    /// Type mismatch in assignment or operation
    TypeMismatch {
        location: SourceLocation,
        expected: String,
        found: String,
        expression: Option<String>, // Optional: expression that caused the error
    },

    /// Use of undefined variable or function
    UndefinedVariable {
        location: SourceLocation,
        name: String,
    },

    /// Invalid binary operation for the given types
    InvalidBinaryOp {
        location: SourceLocation,
        op: String,
        left_type: String,
        right_type: String,
    },

    /// Invalid unary operation for the given type
    InvalidUnaryOp {
        location: SourceLocation,
        op: String,
        operand_type: String,
    },

    /// Function call with wrong number of arguments
    ArgumentCountMismatch {
        location: SourceLocation,
        function: String,
        expected: usize,
        found: usize,
    },

    /// Function call with wrong argument type
    ArgumentTypeMismatch {
        location: SourceLocation,
        function: String,
        arg_index: usize,
        expected: String,
        found: String,
    },

    /// Duplicate definition of variable or function
    DuplicateDefinition {
        location: SourceLocation,
        name: String,
        previous_location: Option<SourceLocation>,
    },

    /// Invalid style property or value
    InvalidStyleProperty {
        location: SourceLocation,
        property: String,
        value: String,
    },

    /// Missing required style property
    MissingStyleProperty {
        location: SourceLocation,
        element: String,
        property: String,
    },

    /// Invalid layout constraint
    InvalidLayoutConstraint {
        location: SourceLocation,
        constraint: String,
        reason: String,
    },
}

impl SemanticError {
    /// Get a short code for the error type (useful for documentation)
    pub fn code(&self) -> &'static str {
        match self {
            SemanticError::TypeMismatch { .. } => "E0001",
            SemanticError::UndefinedVariable { .. } => "E0002",
            SemanticError::InvalidBinaryOp { .. } => "E0003",
            SemanticError::InvalidUnaryOp { .. } => "E0004",
            SemanticError::ArgumentCountMismatch { .. } => "E0005",
            SemanticError::ArgumentTypeMismatch { .. } => "E0006",
            SemanticError::DuplicateDefinition { .. } => "E0007",
            SemanticError::InvalidStyleProperty { .. } => "E0008",
            SemanticError::MissingStyleProperty { .. } => "E0009",
            SemanticError::InvalidLayoutConstraint { .. } => "E0010",
        }
    }
}

impl Diagnostic for SemanticError {
    fn message(&self) -> &str {
        match self {
            SemanticError::TypeMismatch { .. } => "type mismatch",
            SemanticError::UndefinedVariable { .. } => "undefined variable or function",
            SemanticError::InvalidBinaryOp { .. } => "invalid binary operation",
            SemanticError::InvalidUnaryOp { .. } => "invalid unary operation",
            SemanticError::ArgumentCountMismatch { .. } => "argument count mismatch",
            SemanticError::ArgumentTypeMismatch { .. } => "argument type mismatch",
            SemanticError::DuplicateDefinition { .. } => "duplicate definition",
            SemanticError::InvalidStyleProperty { .. } => "invalid style property or value",
            SemanticError::MissingStyleProperty { .. } => "missing required style property",
            SemanticError::InvalidLayoutConstraint { .. } => "invalid layout constraint",
        }
    }

    fn location(&self) -> SourceLocation {
        match self {
            SemanticError::TypeMismatch { location, .. } => *location,
            SemanticError::UndefinedVariable { location, .. } => *location,
            SemanticError::InvalidBinaryOp { location, .. } => *location,
            SemanticError::InvalidUnaryOp { location, .. } => *location,
            SemanticError::ArgumentCountMismatch { location, .. } => *location,
            SemanticError::ArgumentTypeMismatch { location, .. } => *location,
            SemanticError::DuplicateDefinition { location, .. } => *location,
            SemanticError::InvalidStyleProperty { location, .. } => *location,
            SemanticError::MissingStyleProperty { location, .. } => *location,
            SemanticError::InvalidLayoutConstraint { location, .. } => *location,
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn help(&self) -> Option<&str> {
        // Help messages are built dynamically in the Display impl
        None
    }
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.code();
        let message = self.message();
        let location = self.location();

        write!(
            f,
            "[{}] {} at {}: {}",
            code,
            message,
            location,
            self.detailed_message()
        )
    }
}

impl SemanticError {
    fn detailed_message(&self) -> String {
        match self {
            SemanticError::TypeMismatch {
                expected,
                found,
                expression,
                ..
            } => {
                let expr_str = expression
                    .as_ref()
                    .map(|e| format!(" in expression `{}`", e))
                    .unwrap_or_default();
                format!(
                    "expected type `{}`, found type `{}`{}",
                    expected, found, expr_str
                )
            }
            SemanticError::UndefinedVariable { name, .. } => {
                format!("`{}` is not defined in this scope", name)
            }
            SemanticError::InvalidBinaryOp {
                op,
                left_type,
                right_type,
                ..
            } => {
                format!(
                    "cannot apply operator `{}` to types `{}` and `{}`",
                    op, left_type, right_type
                )
            }
            SemanticError::InvalidUnaryOp {
                op, operand_type, ..
            } => {
                format!("cannot apply operator `{}` to type `{}`", op, operand_type)
            }
            SemanticError::ArgumentCountMismatch {
                function,
                expected,
                found,
                ..
            } => {
                format!(
                    "function `{}` expects {} argument(s), found {}",
                    function, expected, found
                )
            }
            SemanticError::ArgumentTypeMismatch {
                function,
                arg_index,
                expected,
                found,
                ..
            } => {
                format!(
                    "argument {} of function `{}` expects type `{}`, found `{}`",
                    arg_index + 1,
                    function,
                    expected,
                    found
                )
            }
            SemanticError::DuplicateDefinition {
                name,
                previous_location,
                ..
            } => {
                let prev_str = previous_location
                    .map(|loc| format!(" (previously defined at {})", loc))
                    .unwrap_or_default();
                format!("`{}` is already defined{}", name, prev_str)
            }
            SemanticError::InvalidStyleProperty {
                property, value, ..
            } => {
                format!(
                    "invalid value `{}` for style property `{}`",
                    value, property
                )
            }
            SemanticError::MissingStyleProperty {
                element, property, ..
            } => {
                format!(
                    "element `{}` is missing required style property `{}`",
                    element, property
                )
            }
            SemanticError::InvalidLayoutConstraint {
                constraint, reason, ..
            } => {
                format!("invalid layout constraint `{}`: {}", constraint, reason)
            }
        }
    }
}

impl std::error::Error for SemanticError {}
