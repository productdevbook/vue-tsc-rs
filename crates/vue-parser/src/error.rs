//! Error types for Vue SFC parsing.

use source_map::Span;
use std::fmt;

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// An error that occurred during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The error message.
    pub message: String,
    /// The span where the error occurred.
    pub span: Span,
    /// The error code.
    pub code: ErrorCode,
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(message: impl Into<String>, span: Span, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            span,
            code,
        }
    }

    /// Create an unexpected token error.
    pub fn unexpected_token(expected: &str, found: &str, span: Span) -> Self {
        Self::new(
            format!("Expected {}, found {}", expected, found),
            span,
            ErrorCode::UnexpectedToken,
        )
    }

    /// Create an unclosed tag error.
    pub fn unclosed_tag(tag: &str, span: Span) -> Self {
        Self::new(
            format!("Unclosed tag: <{}>", tag),
            span,
            ErrorCode::UnclosedTag,
        )
    }

    /// Create an invalid attribute error.
    pub fn invalid_attribute(attr: &str, span: Span) -> Self {
        Self::new(
            format!("Invalid attribute: {}", attr),
            span,
            ErrorCode::InvalidAttribute,
        )
    }

    /// Create a duplicate block error.
    pub fn duplicate_block(block: &str, span: Span) -> Self {
        Self::new(
            format!("Duplicate {} block", block),
            span,
            ErrorCode::DuplicateBlock,
        )
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

/// Error codes for categorizing parse errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Unexpected token encountered.
    UnexpectedToken,
    /// Unclosed tag.
    UnclosedTag,
    /// Invalid attribute.
    InvalidAttribute,
    /// Duplicate block (e.g., two <template> blocks).
    DuplicateBlock,
    /// Invalid block content.
    InvalidContent,
    /// Syntax error.
    SyntaxError,
}

impl ErrorCode {
    /// Get the error code as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::UnexpectedToken => "unexpected-token",
            ErrorCode::UnclosedTag => "unclosed-tag",
            ErrorCode::InvalidAttribute => "invalid-attribute",
            ErrorCode::DuplicateBlock => "duplicate-block",
            ErrorCode::InvalidContent => "invalid-content",
            ErrorCode::SyntaxError => "syntax-error",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
