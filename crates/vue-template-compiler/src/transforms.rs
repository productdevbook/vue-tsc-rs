//! Template transformations.
//!
//! This module provides transformations for Vue template AST nodes,
//! such as normalizing directives and optimizing static content.

use crate::ast::*;
use smol_str::SmolStr;

/// Transform context for tracking state during transformation.
pub struct TransformContext {
    /// Current scope variables.
    pub scope_vars: Vec<ScopeVar>,
    /// Component imports detected.
    pub components: Vec<SmolStr>,
    /// Directive imports detected.
    pub directives: Vec<SmolStr>,
    /// Whether we're inside a v-for.
    pub in_v_for: bool,
    /// Whether we're inside a v-slot.
    pub in_v_slot: bool,
}

impl Default for TransformContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformContext {
    /// Create a new transform context.
    pub fn new() -> Self {
        Self {
            scope_vars: Vec::new(),
            components: Vec::new(),
            directives: Vec::new(),
            in_v_for: false,
            in_v_slot: false,
        }
    }

    /// Add a scope variable.
    pub fn add_scope_var(&mut self, name: SmolStr, source: SmolStr, span: source_map::Span) {
        self.scope_vars.push(ScopeVar { name, source, span });
    }

    /// Check if a variable is in scope.
    pub fn has_scope_var(&self, name: &str) -> bool {
        self.scope_vars.iter().any(|v| v.name == name)
    }

    /// Enter a new scope.
    pub fn enter_scope(&mut self) -> usize {
        self.scope_vars.len()
    }

    /// Exit a scope, removing variables added since marker.
    pub fn exit_scope(&mut self, marker: usize) {
        self.scope_vars.truncate(marker);
    }
}

/// Transform a template AST.
pub fn transform(ast: &mut TemplateAst, ctx: &mut TransformContext) {
    for child in &mut ast.children {
        transform_node(child, ctx);
    }
    ast.scope_vars = ctx.scope_vars.clone();
}

/// Transform a single node.
fn transform_node(node: &mut TemplateNode, ctx: &mut TransformContext) {
    match node {
        TemplateNode::Element(el) => transform_element(el, ctx),
        TemplateNode::For(f) => transform_for(f, ctx),
        TemplateNode::If(i) => transform_if(i, ctx),
        TemplateNode::Template(t) => transform_template(t, ctx),
        TemplateNode::SlotOutlet(s) => transform_slot_outlet(s, ctx),
        _ => {}
    }
}

/// Transform an element node.
fn transform_element(el: &mut ElementNode, ctx: &mut TransformContext) {
    // Track component usage
    if el.is_component {
        let component_name = el.tag.clone();
        if !ctx.components.contains(&component_name) {
            ctx.components.push(component_name);
        }
    }

    // Track directive usage
    for dir in &el.directives {
        if !is_builtin_directive(&dir.name) {
            if !ctx.directives.contains(&dir.name) {
                ctx.directives.push(dir.name.clone());
            }
        }
    }

    // Process children
    for child in &mut el.children {
        transform_node(child, ctx);
    }

    // Process slots
    for (_name, slot) in &mut el.slots {
        let scope_marker = ctx.enter_scope();
        ctx.in_v_slot = true;

        // Add slot props to scope
        if let Some(props) = &slot.props {
            // Parse destructured props
            for var in extract_binding_names(&props.pattern) {
                ctx.add_scope_var(var.into(), "slot-props".into(), props.span);
            }
        }

        for child in &mut slot.children {
            transform_node(child, ctx);
        }

        ctx.in_v_slot = false;
        ctx.exit_scope(scope_marker);
    }
}

/// Transform a for node.
fn transform_for(f: &mut ForNode, ctx: &mut TransformContext) {
    let scope_marker = ctx.enter_scope();
    ctx.in_v_for = true;

    // Add loop variables to scope
    for var in extract_binding_names(&f.value.pattern) {
        ctx.add_scope_var(var.into(), "v-for".into(), f.value.span);
    }
    if let Some(key) = &f.key {
        for var in extract_binding_names(&key.pattern) {
            ctx.add_scope_var(var.into(), "v-for".into(), key.span);
        }
    }
    if let Some(index) = &f.index {
        for var in extract_binding_names(&index.pattern) {
            ctx.add_scope_var(var.into(), "v-for".into(), index.span);
        }
    }

    for child in &mut f.children {
        transform_node(child, ctx);
    }

    ctx.in_v_for = false;
    ctx.exit_scope(scope_marker);
}

/// Transform an if node.
fn transform_if(i: &mut IfNode, ctx: &mut TransformContext) {
    for branch in &mut i.branches {
        for child in &mut branch.children {
            transform_node(child, ctx);
        }
    }
}

/// Transform a template element node.
fn transform_template(t: &mut TemplateElementNode, ctx: &mut TransformContext) {
    // Check for v-slot
    let v_slot = t.directives.iter().find(|d| d.name == "slot");
    if let Some(dir) = v_slot {
        let scope_marker = ctx.enter_scope();
        ctx.in_v_slot = true;

        // Add slot props to scope
        if let Some(value) = &dir.value {
            for var in extract_binding_names(&value.content) {
                ctx.add_scope_var(var.into(), "slot-props".into(), value.span);
            }
        }

        for child in &mut t.children {
            transform_node(child, ctx);
        }

        ctx.in_v_slot = false;
        ctx.exit_scope(scope_marker);
    } else {
        for child in &mut t.children {
            transform_node(child, ctx);
        }
    }
}

/// Transform a slot outlet node.
fn transform_slot_outlet(s: &mut SlotOutletNode, ctx: &mut TransformContext) {
    for child in &mut s.fallback {
        transform_node(child, ctx);
    }
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

/// Extract binding names from a pattern (simple extraction).
fn extract_binding_names(pattern: &str) -> Vec<&str> {
    let pattern = pattern.trim();

    // Handle object destructuring: { a, b: c, d = 1 }
    if pattern.starts_with('{') && pattern.ends_with('}') {
        let inner = &pattern[1..pattern.len() - 1];
        return inner
            .split(',')
            .filter_map(|part| {
                let part = part.trim();
                // Handle renaming: key: value
                if let Some(colon_pos) = part.find(':') {
                    let value_part = part[colon_pos + 1..].trim();
                    // Handle default: value = default
                    if let Some(eq_pos) = value_part.find('=') {
                        Some(value_part[..eq_pos].trim())
                    } else {
                        Some(value_part)
                    }
                } else if let Some(eq_pos) = part.find('=') {
                    // Handle default: name = default
                    Some(part[..eq_pos].trim())
                } else {
                    Some(part)
                }
            })
            .filter(|s| !s.is_empty() && !s.starts_with("..."))
            .collect();
    }

    // Handle array destructuring: [a, b, c]
    if pattern.starts_with('[') && pattern.ends_with(']') {
        let inner = &pattern[1..pattern.len() - 1];
        return inner
            .split(',')
            .filter_map(|part| {
                let part = part.trim();
                if part.is_empty() || part.starts_with("...") {
                    None
                } else if let Some(eq_pos) = part.find('=') {
                    Some(part[..eq_pos].trim())
                } else {
                    Some(part)
                }
            })
            .collect();
    }

    // Simple identifier
    if pattern
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    {
        vec![pattern]
    } else {
        Vec::new()
    }
}

/// Camelize a string (kebab-case to camelCase).
pub fn camelize(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert to PascalCase.
pub fn pascalize(s: &str) -> String {
    let camel = camelize(s);
    if let Some(first) = camel.chars().next() {
        format!("{}{}", first.to_ascii_uppercase(), &camel[first.len_utf8()..])
    } else {
        camel
    }
}

/// Convert to kebab-case.
pub fn hyphenate(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_binding_names_simple() {
        assert_eq!(extract_binding_names("item"), vec!["item"]);
    }

    #[test]
    fn test_extract_binding_names_destructure() {
        assert_eq!(
            extract_binding_names("{ a, b, c }"),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn test_extract_binding_names_rename() {
        assert_eq!(extract_binding_names("{ key: value }"), vec!["value"]);
    }

    #[test]
    fn test_extract_binding_names_array() {
        assert_eq!(extract_binding_names("[a, b, c]"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_camelize() {
        assert_eq!(camelize("foo-bar"), "fooBar");
        assert_eq!(camelize("foo-bar-baz"), "fooBarBaz");
        assert_eq!(camelize("foo"), "foo");
    }

    #[test]
    fn test_pascalize() {
        assert_eq!(pascalize("foo-bar"), "FooBar");
        assert_eq!(pascalize("my-component"), "MyComponent");
    }

    #[test]
    fn test_hyphenate() {
        assert_eq!(hyphenate("FooBar"), "foo-bar");
        assert_eq!(hyphenate("MyComponent"), "my-component");
    }
}
