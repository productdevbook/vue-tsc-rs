//! Vue-specific diagnostics.
//!
//! This crate provides Vue-specific linting and diagnostics including:
//! - Template validation
//! - Component naming conventions
//! - Props validation
//! - Event validation
//! - Slot validation

pub mod component;
pub mod template;

use source_map::Span;
use vue_parser::Sfc;
use vue_template_compiler::TemplateAst;

/// A diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The diagnostic message.
    pub message: String,
    /// The span where the diagnostic applies.
    pub span: Span,
    /// The severity level.
    pub severity: Severity,
    /// The diagnostic code.
    pub code: DiagnosticCode,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(message: impl Into<String>, span: Span, code: DiagnosticCode) -> Self {
        Self {
            message: message.into(),
            span,
            severity: Severity::Error,
            code,
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(message: impl Into<String>, span: Span, code: DiagnosticCode) -> Self {
        Self {
            message: message.into(),
            span,
            severity: Severity::Warning,
            code,
        }
    }

    /// Create a new hint diagnostic.
    pub fn hint(message: impl Into<String>, span: Span, code: DiagnosticCode) -> Self {
        Self {
            message: message.into(),
            span,
            severity: Severity::Hint,
            code,
        }
    }
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// An error that should block type checking.
    Error,
    /// A warning that indicates a potential issue.
    Warning,
    /// A hint for improvement.
    Hint,
}

impl Severity {
    /// Get the severity as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Hint => "hint",
        }
    }
}

/// Diagnostic codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    // Template diagnostics
    /// Unknown component.
    UnknownComponent,
    /// Unknown directive.
    UnknownDirective,
    /// Invalid v-for syntax.
    InvalidVFor,
    /// Invalid v-model syntax.
    InvalidVModel,
    /// Missing required prop.
    MissingProp,
    /// Invalid prop type.
    InvalidPropType,
    /// Unknown event.
    UnknownEvent,
    /// Invalid slot usage.
    InvalidSlot,
    /// Duplicate key in v-for.
    DuplicateKey,
    /// Missing key in v-for.
    MissingKey,

    // Component diagnostics
    /// Invalid component name.
    InvalidComponentName,
    /// Missing required option.
    MissingOption,
    /// Invalid props definition.
    InvalidPropsDefinition,
    /// Invalid emits definition.
    InvalidEmitsDefinition,

    // Script diagnostics
    /// Invalid macro usage.
    InvalidMacroUsage,
    /// Duplicate macro.
    DuplicateMacro,

    // Style diagnostics
    /// Unused CSS selector.
    UnusedSelector,
    /// Invalid deep selector.
    InvalidDeepSelector,
}

impl DiagnosticCode {
    /// Get the code as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnknownComponent => "unknown-component",
            Self::UnknownDirective => "unknown-directive",
            Self::InvalidVFor => "invalid-v-for",
            Self::InvalidVModel => "invalid-v-model",
            Self::MissingProp => "missing-prop",
            Self::InvalidPropType => "invalid-prop-type",
            Self::UnknownEvent => "unknown-event",
            Self::InvalidSlot => "invalid-slot",
            Self::DuplicateKey => "duplicate-key",
            Self::MissingKey => "missing-key",
            Self::InvalidComponentName => "invalid-component-name",
            Self::MissingOption => "missing-option",
            Self::InvalidPropsDefinition => "invalid-props-definition",
            Self::InvalidEmitsDefinition => "invalid-emits-definition",
            Self::InvalidMacroUsage => "invalid-macro-usage",
            Self::DuplicateMacro => "duplicate-macro",
            Self::UnusedSelector => "unused-selector",
            Self::InvalidDeepSelector => "invalid-deep-selector",
        }
    }
}

/// Options for diagnostics.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticOptions {
    /// Check for unknown components.
    pub check_unknown_components: bool,
    /// Check for unknown directives.
    pub check_unknown_directives: bool,
    /// Check for missing keys in v-for.
    pub check_v_for_keys: bool,
    /// Known component names.
    pub known_components: Vec<String>,
    /// Known directive names.
    pub known_directives: Vec<String>,
}

/// Run diagnostics on an SFC.
pub fn diagnose_sfc(sfc: &Sfc, options: &DiagnosticOptions) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Component-level diagnostics
    diagnostics.extend(component::check_sfc(sfc, options));

    // Template diagnostics
    if let Some(template) = &sfc.template {
        if let Ok(ast) = vue_template_compiler::parse_template(&template.content) {
            diagnostics.extend(template::check_template(&ast, options));
        }
    }

    diagnostics
}

/// Run diagnostics on a template AST.
pub fn diagnose_template(ast: &TemplateAst, options: &DiagnosticOptions) -> Vec<Diagnostic> {
    template::check_template(ast, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vue_parser::parse_sfc;

    #[test]
    fn test_diagnose_empty_sfc() {
        let sfc = parse_sfc("").unwrap();
        let diagnostics = diagnose_sfc(&sfc, &DiagnosticOptions::default());
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_diagnose_valid_sfc() {
        let source = r#"<script setup>
const msg = 'Hello'
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;
        let sfc = parse_sfc(source).unwrap();
        let diagnostics = diagnose_sfc(&sfc, &DiagnosticOptions::default());
        // Should have no errors for a valid SFC
        assert!(diagnostics.iter().all(|d| d.severity != Severity::Error));
    }
}
