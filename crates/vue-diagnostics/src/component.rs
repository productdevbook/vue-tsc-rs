//! Component-level diagnostics.

use crate::{Diagnostic, DiagnosticCode, DiagnosticOptions};
use source_map::Span;
use vue_parser::Sfc;

/// Check an SFC for component-level issues.
pub fn check_sfc(sfc: &Sfc, _options: &DiagnosticOptions) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for multiple script setup blocks (already caught by parser)
    // Check for conflicting script types

    // Check script setup content
    if let Some(script_setup) = &sfc.script_setup {
        diagnostics.extend(check_script_setup(
            &script_setup.content,
            script_setup.content_span,
        ));
    }

    // Check for proper component structure
    if sfc.template.is_none() && sfc.script.is_none() && sfc.script_setup.is_none() {
        // Empty component - could be a hint
    }

    diagnostics
}

/// Check script setup content for issues.
fn check_script_setup(content: &str, span: Span) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for multiple defineProps
    let props_count = content.matches("defineProps").count();
    if props_count > 1 {
        diagnostics.push(Diagnostic::error(
            "defineProps can only be called once",
            span,
            DiagnosticCode::DuplicateMacro,
        ));
    }

    // Check for multiple defineEmits
    let emits_count = content.matches("defineEmits").count();
    if emits_count > 1 {
        diagnostics.push(Diagnostic::error(
            "defineEmits can only be called once",
            span,
            DiagnosticCode::DuplicateMacro,
        ));
    }

    // Check for multiple defineSlots
    let slots_count = content.matches("defineSlots").count();
    if slots_count > 1 {
        diagnostics.push(Diagnostic::error(
            "defineSlots can only be called once",
            span,
            DiagnosticCode::DuplicateMacro,
        ));
    }

    // Check for multiple defineExpose
    let expose_count = content.matches("defineExpose").count();
    if expose_count > 1 {
        diagnostics.push(Diagnostic::error(
            "defineExpose can only be called once",
            span,
            DiagnosticCode::DuplicateMacro,
        ));
    }

    // Check for defineOptions - can only be called once
    let options_count = content.matches("defineOptions").count();
    if options_count > 1 {
        diagnostics.push(Diagnostic::error(
            "defineOptions can only be called once",
            span,
            DiagnosticCode::DuplicateMacro,
        ));
    }

    diagnostics
}

/// Check if a component name follows conventions.
pub fn check_component_name(name: &str) -> Option<Diagnostic> {
    // Check for PascalCase
    if name.is_empty() {
        return Some(Diagnostic::warning(
            "Component name should not be empty",
            Span::empty(0),
            DiagnosticCode::InvalidComponentName,
        ));
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_uppercase() {
        return Some(Diagnostic::warning(
            format!("Component name '{}' should be in PascalCase", name),
            Span::empty(0),
            DiagnosticCode::InvalidComponentName,
        ));
    }

    // Check for reserved names
    if is_reserved_name(name) {
        return Some(Diagnostic::error(
            format!(
                "'{}' is a reserved name and cannot be used as a component name",
                name
            ),
            Span::empty(0),
            DiagnosticCode::InvalidComponentName,
        ));
    }

    None
}

/// Check if a name is reserved.
fn is_reserved_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "slot" | "component" | "template" | "script" | "style" | "html" | "body" | "head" | "base"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_check_component_name_valid() {
        assert!(check_component_name("MyComponent").is_none());
        assert!(check_component_name("Button").is_none());
        assert!(check_component_name("TheHeader").is_none());
    }

    #[test]
    fn test_check_component_name_invalid() {
        assert!(check_component_name("myComponent").is_some());
        assert!(check_component_name("").is_some());
    }

    #[test]
    fn test_check_component_name_reserved() {
        let diag = check_component_name("Slot").unwrap();
        assert_eq!(diag.severity, Severity::Error);
    }

    #[test]
    fn test_multiple_define_props() {
        let content = "defineProps<{}>(); defineProps<{}>();";
        let diagnostics = check_script_setup(content, Span::new(0, content.len() as u32));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::DuplicateMacro));
    }
}
