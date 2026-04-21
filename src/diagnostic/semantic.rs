use super::{Diagnostic, Severity, SourceLocation};

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
    pub fn type_mismatch(
        expected: impl Into<String>,
        found: impl Into<String>,
        expression: Option<String>,
        location: SourceLocation,
    ) -> Self {
        Self::TypeMismatch {
            location,
            expected: expected.into(),
            found: found.into(),
            expression,
        }
    }

    pub fn undefined_variable(name: impl Into<String>, location: SourceLocation) -> Self {
        Self::UndefinedVariable {
            location,
            name: name.into(),
        }
    }

    pub fn invalid_binary_op(
        op: impl Into<String>,
        left_type: impl Into<String>,
        right_type: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::InvalidBinaryOp {
            location,
            op: op.into(),
            left_type: left_type.into(),
            right_type: right_type.into(),
        }
    }

    pub fn invalid_unary_op(
        op: impl Into<String>,
        operand_type: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::InvalidUnaryOp {
            location,
            op: op.into(),
            operand_type: operand_type.into(),
        }
    }

    pub fn argument_count_mismatch(
        function: impl Into<String>,
        expected: usize,
        found: usize,
        location: SourceLocation,
    ) -> Self {
        Self::ArgumentCountMismatch {
            location,
            function: function.into(),
            expected,
            found,
        }
    }

    pub fn argument_type_mismatch(
        function: impl Into<String>,
        arg_index: usize,
        expected: impl Into<String>,
        found: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::ArgumentTypeMismatch {
            location,
            function: function.into(),
            arg_index,
            expected: expected.into(),
            found: found.into(),
        }
    }

    pub fn duplicate_definition(
        name: impl Into<String>,
        previous_location: Option<SourceLocation>,
        location: SourceLocation,
    ) -> Self {
        Self::DuplicateDefinition {
            location,
            name: name.into(),
            previous_location,
        }
    }

    pub fn invalid_style_property(
        property: impl Into<String>,
        value: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::InvalidStyleProperty {
            location,
            property: property.into(),
            value: value.into(),
        }
    }

    pub fn missing_style_property(
        element: impl Into<String>,
        property: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::MissingStyleProperty {
            location,
            element: element.into(),
            property: property.into(),
        }
    }

    pub fn invalid_layout_constraint(
        constraint: impl Into<String>,
        reason: impl Into<String>,
        location: SourceLocation,
    ) -> Self {
        Self::InvalidLayoutConstraint {
            location,
            constraint: constraint.into(),
            reason: reason.into(),
        }
    }

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
            SemanticError::TypeMismatch { location, .. } => location.clone(),
            SemanticError::UndefinedVariable { location, .. } => location.clone(),
            SemanticError::InvalidBinaryOp { location, .. } => location.clone(),
            SemanticError::InvalidUnaryOp { location, .. } => location.clone(),
            SemanticError::ArgumentCountMismatch { location, .. } => location.clone(),
            SemanticError::ArgumentTypeMismatch { location, .. } => location.clone(),
            SemanticError::DuplicateDefinition { location, .. } => location.clone(),
            SemanticError::InvalidStyleProperty { location, .. } => location.clone(),
            SemanticError::MissingStyleProperty { location, .. } => location.clone(),
            SemanticError::InvalidLayoutConstraint { location, .. } => location.clone(),
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
                    .as_ref()
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
