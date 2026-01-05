//! Template diagnostics.

use crate::{Diagnostic, DiagnosticCode, DiagnosticOptions};
use vue_template_compiler::{
    ElementNode, ForNode, IfNode, TemplateAst, TemplateNode,
};

/// Check a template AST for issues.
pub fn check_template(ast: &TemplateAst, options: &DiagnosticOptions) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for child in &ast.children {
        check_node(child, options, &mut diagnostics);
    }

    diagnostics
}

/// Check a template node for issues.
fn check_node(node: &TemplateNode, options: &DiagnosticOptions, diagnostics: &mut Vec<Diagnostic>) {
    match node {
        TemplateNode::Element(el) => check_element(el, options, diagnostics),
        TemplateNode::For(f) => check_for(f, options, diagnostics),
        TemplateNode::If(i) => check_if(i, options, diagnostics),
        TemplateNode::Template(t) => {
            for child in &t.children {
                check_node(child, options, diagnostics);
            }
        }
        TemplateNode::SlotOutlet(s) => {
            for child in &s.fallback {
                check_node(child, options, diagnostics);
            }
        }
        _ => {}
    }
}

/// Check an element for issues.
fn check_element(el: &ElementNode, options: &DiagnosticOptions, diagnostics: &mut Vec<Diagnostic>) {
    // Check for unknown components
    if options.check_unknown_components && el.is_component {
        if !is_known_component(&el.tag, options) {
            diagnostics.push(Diagnostic::warning(
                format!("Unknown component: <{}>", el.tag),
                el.tag_span,
                DiagnosticCode::UnknownComponent,
            ));
        }
    }

    // Check for unknown directives
    if options.check_unknown_directives {
        for dir in &el.directives {
            if !is_builtin_directive(&dir.name) && !is_known_directive(&dir.name, options) {
                diagnostics.push(Diagnostic::warning(
                    format!("Unknown directive: v-{}", dir.name),
                    dir.span,
                    DiagnosticCode::UnknownDirective,
                ));
            }
        }
    }

    // Check v-model on invalid elements
    if let Some(model_dir) = el.directives.iter().find(|d| d.name == "model") {
        if !can_use_v_model(&el.tag) {
            diagnostics.push(Diagnostic::error(
                format!("v-model is not valid on <{}> elements", el.tag),
                model_dir.span,
                DiagnosticCode::InvalidVModel,
            ));
        }
    }

    // Check children recursively
    for child in &el.children {
        check_node(child, options, diagnostics);
    }

    // Check slots
    for (_name, slot) in &el.slots {
        for child in &slot.children {
            check_node(child, options, diagnostics);
        }
    }
}

/// Check a v-for node for issues.
fn check_for(f: &ForNode, options: &DiagnosticOptions, diagnostics: &mut Vec<Diagnostic>) {
    // Check for missing key attribute
    if options.check_v_for_keys && f.key_attr.is_none() {
        diagnostics.push(Diagnostic::warning(
            "v-for is missing a :key attribute",
            f.span,
            DiagnosticCode::MissingKey,
        ));
    }

    // Check children
    for child in &f.children {
        check_node(child, options, diagnostics);
    }
}

/// Check an if node for issues.
fn check_if(i: &IfNode, options: &DiagnosticOptions, diagnostics: &mut Vec<Diagnostic>) {
    for branch in &i.branches {
        for child in &branch.children {
            check_node(child, options, diagnostics);
        }
    }
}

/// Check if a component is known.
fn is_known_component(name: &str, options: &DiagnosticOptions) -> bool {
    // Built-in Vue components
    if is_builtin_component(name) {
        return true;
    }

    // User-specified known components
    options
        .known_components
        .iter()
        .any(|c| c.eq_ignore_ascii_case(name))
}

/// Check if a directive is known.
fn is_known_directive(name: &str, options: &DiagnosticOptions) -> bool {
    options
        .known_directives
        .iter()
        .any(|d| d.eq_ignore_ascii_case(name))
}

/// Check if a directive is a built-in Vue directive.
fn is_builtin_directive(name: &str) -> bool {
    matches!(
        name,
        "if" | "else"
            | "else-if"
            | "for"
            | "show"
            | "bind"
            | "on"
            | "model"
            | "slot"
            | "pre"
            | "cloak"
            | "once"
            | "memo"
            | "html"
            | "text"
    )
}

/// Check if a component is a built-in Vue component.
fn is_builtin_component(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "transition"
            | "transition-group"
            | "transitiongroup"
            | "keep-alive"
            | "keepalive"
            | "suspense"
            | "teleport"
            | "slot"
            | "component"
    )
}

/// Check if an element can use v-model.
fn can_use_v_model(tag: &str) -> bool {
    let tag_lower = tag.to_lowercase();

    // HTML form elements
    if matches!(
        tag_lower.as_str(),
        "input" | "select" | "textarea"
    ) {
        return true;
    }

    // Components (assumed to support v-model)
    if tag.chars().next().map_or(false, |c| c.is_uppercase()) || tag.contains('-') {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use vue_template_compiler::parse_template;

    #[test]
    fn test_check_valid_template() {
        let ast = parse_template("<div>Hello</div>").unwrap();
        let diagnostics = check_template(&ast, &DiagnosticOptions::default());
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_check_missing_key() {
        let ast = parse_template(r#"<div v-for="item in items">{{ item }}</div>"#).unwrap();
        let options = DiagnosticOptions {
            check_v_for_keys: true,
            ..Default::default()
        };
        let diagnostics = check_template(&ast, &options);
        assert!(diagnostics.iter().any(|d| d.code == DiagnosticCode::MissingKey));
    }

    #[test]
    fn test_check_v_model_on_div() {
        let ast = parse_template(r#"<div v-model="value">Content</div>"#).unwrap();
        let diagnostics = check_template(&ast, &DiagnosticOptions::default());
        assert!(diagnostics.iter().any(|d| d.code == DiagnosticCode::InvalidVModel));
    }

    #[test]
    fn test_check_v_model_on_input() {
        let ast = parse_template(r#"<input v-model="value" />"#).unwrap();
        let diagnostics = check_template(&ast, &DiagnosticOptions::default());
        assert!(diagnostics.iter().all(|d| d.code != DiagnosticCode::InvalidVModel));
    }
}
