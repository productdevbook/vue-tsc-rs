//! Error types for Vue template compilation.

use source_map::Span;
use std::fmt;

/// Result type for compilation operations.
pub type CompileResult<T> = Result<T, CompileError>;

/// An error that occurred during template compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    /// The error message.
    pub message: String,
    /// The span where the error occurred.
    pub span: Span,
    /// The error code.
    pub code: CompileErrorCode,
}

impl CompileError {
    /// Create a new compile error.
    pub fn new(message: impl Into<String>, span: Span, code: CompileErrorCode) -> Self {
        Self {
            message: message.into(),
            span,
            code,
        }
    }

    /// Create an invalid directive error.
    pub fn invalid_directive(directive: &str, span: Span) -> Self {
        Self::new(
            format!("Invalid directive: {}", directive),
            span,
            CompileErrorCode::InvalidDirective,
        )
    }

    /// Create an invalid expression error.
    pub fn invalid_expression(expr: &str, span: Span) -> Self {
        Self::new(
            format!("Invalid expression: {}", expr),
            span,
            CompileErrorCode::InvalidExpression,
        )
    }

    /// Create an unexpected token error.
    pub fn unexpected_token(expected: &str, found: &str, span: Span) -> Self {
        Self::new(
            format!("Expected {}, found {}", expected, found),
            span,
            CompileErrorCode::UnexpectedToken,
        )
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CompileError {}

/// Error codes for template compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompileErrorCode {
    /// Invalid directive usage.
    InvalidDirective,
    /// Invalid expression syntax.
    InvalidExpression,
    /// Unexpected token.
    UnexpectedToken,
    /// Unclosed element.
    UnclosedElement,
    /// Missing required attribute.
    MissingAttribute,
    /// Invalid slot usage.
    InvalidSlot,
    /// Invalid v-for syntax.
    InvalidVFor,
    /// Invalid v-model syntax.
    InvalidVModel,
    /// Component resolution error.
    ComponentResolution,
}

impl CompileErrorCode {
    /// Get the error code as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidDirective => "invalid-directive",
            Self::InvalidExpression => "invalid-expression",
            Self::UnexpectedToken => "unexpected-token",
            Self::UnclosedElement => "unclosed-element",
            Self::MissingAttribute => "missing-attribute",
            Self::InvalidSlot => "invalid-slot",
            Self::InvalidVFor => "invalid-v-for",
            Self::InvalidVModel => "invalid-v-model",
            Self::ComponentResolution => "component-resolution",
        }
    }
}

impl fmt::Display for CompileErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
