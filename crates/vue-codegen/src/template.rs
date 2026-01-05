//! Template code generation.
//!
//! This module generates TypeScript code from Vue template AST
//! that enables type checking of template expressions.

use crate::context::{CodegenContext, VarSource};
use crate::helpers::{is_html_tag, is_svg_tag};
use source_map::CodeBuilder;
use vue_template_compiler::{
    Attribute, ElementNode, EventListener, Expression, ForNode, IfBranch, IfNode,
    InterpolationNode, Prop, SlotOutletNode, TemplateAst, TemplateNode,
};

/// Generate type checking code for a template.
pub fn generate_template(builder: &mut CodeBuilder, ast: &TemplateAst, ctx: &mut CodegenContext) {
    builder.push_str("\n// Template type checking\n");
    builder.push_str("function __VLS_template() {\n");

    // Add template context
    builder.push_str("  const __VLS_ctx = {} as __VLS_TemplateContext & {\n");
    builder.push_str("    $props: typeof __VLS_props;\n");
    builder.push_str("    $emit: typeof __VLS_emit;\n");
    builder.push_str("  };\n\n");

    // Generate code for children
    for child in &ast.children {
        generate_node(builder, child, ctx, 1);
    }

    builder.push_str("}\n");
}

/// Generate code for a template node.
fn generate_node(builder: &mut CodeBuilder, node: &TemplateNode, ctx: &mut CodegenContext, indent: usize) {
    match node {
        TemplateNode::Element(el) => generate_element(builder, el, ctx, indent),
        TemplateNode::Interpolation(interp) => generate_interpolation(builder, interp, ctx, indent),
        TemplateNode::If(if_node) => generate_if(builder, if_node, ctx, indent),
        TemplateNode::For(for_node) => generate_for(builder, for_node, ctx, indent),
        TemplateNode::SlotOutlet(slot) => generate_slot_outlet(builder, slot, ctx, indent),
        TemplateNode::Template(tmpl) => {
            for child in &tmpl.children {
                generate_node(builder, child, ctx, indent);
            }
        }
        TemplateNode::Text(_) | TemplateNode::Comment(_) => {
            // Text and comments don't need type checking
        }
    }
}

/// Generate code for an element.
fn generate_element(
    builder: &mut CodeBuilder,
    el: &ElementNode,
    ctx: &mut CodegenContext,
    indent: usize,
) {
    let ind = "  ".repeat(indent);
    let tag = &el.tag;

    // Determine if this is a component or HTML element
    let is_component = el.is_component;

    if is_component {
        // Component
        ctx.use_component(tag.clone());

        builder.push_str(&ind);
        builder.push_str("{\n");

        // Resolve component
        builder.push_str(&ind);
        builder.push_str("  const __VLS_");
        builder.push_str(&ctx.unique_id("component"));
        builder.push_str(" = __VLS_resolveComponent('");
        builder.push_str(tag);
        builder.push_str("');\n");

        // Check props
        generate_props_check(builder, &el.props, ctx, indent + 1);

        // Check events
        generate_events_check(builder, &el.events, ctx, indent + 1);

        // Check slots
        for (_name, slot) in &el.slots {
            let scope_marker = ctx.enter_scope();

            // Add slot props to scope
            if let Some(props) = &slot.props {
                for name in extract_binding_names(&props.pattern) {
                    ctx.add_var(name, VarSource::SlotProps);
                }
            }

            for child in &slot.children {
                generate_node(builder, child, ctx, indent + 1);
            }

            ctx.exit_scope(scope_marker);
        }

        builder.push_str(&ind);
        builder.push_str("}\n");
    } else {
        // HTML/SVG element
        let is_svg = is_svg_tag(tag);
        let is_html = is_html_tag(tag);

        if is_html || is_svg {
            builder.push_str(&ind);
            builder.push_str("{\n");

            // Check attributes
            for attr in &el.attrs {
                generate_attr_check(builder, attr, tag, ctx, indent + 1);
            }

            // Check props (dynamic attributes)
            generate_props_check(builder, &el.props, ctx, indent + 1);

            // Check events
            generate_events_check(builder, &el.events, ctx, indent + 1);

            builder.push_str(&ind);
            builder.push_str("}\n");
        }
    }

    // Generate children
    for child in &el.children {
        generate_node(builder, child, ctx, indent);
    }
}

/// Generate code for props type checking.
fn generate_props_check(
    builder: &mut CodeBuilder,
    props: &[Prop],
    ctx: &mut CodegenContext,
    indent: usize,
) {
    let ind = "  ".repeat(indent);

    for prop in props {
        builder.push_str(&ind);
        builder.push_str("// prop: ");
        builder.push_str(&prop.name);
        builder.push_str("\n");

        builder.push_str(&ind);
        builder.push_str("(");
        generate_expression(builder, &prop.value, ctx);
        builder.push_str(");\n");
    }
}

/// Generate code for events type checking.
fn generate_events_check(
    builder: &mut CodeBuilder,
    events: &[EventListener],
    ctx: &mut CodegenContext,
    indent: usize,
) {
    let ind = "  ".repeat(indent);

    for event in events {
        builder.push_str(&ind);
        builder.push_str("// event: ");
        builder.push_str(&event.name);
        builder.push_str("\n");

        builder.push_str(&ind);
        builder.push_str("(");
        generate_expression(builder, &event.handler, ctx);
        builder.push_str(");\n");
    }
}

/// Generate code for attribute type checking.
fn generate_attr_check(
    builder: &mut CodeBuilder,
    attr: &Attribute,
    tag: &str,
    ctx: &mut CodegenContext,
    indent: usize,
) {
    // Static attributes don't need runtime type checking
    // but we can validate them against known HTML attributes
    let _ = (builder, attr, tag, ctx, indent);
}

/// Generate code for an interpolation.
fn generate_interpolation(
    builder: &mut CodeBuilder,
    interp: &InterpolationNode,
    ctx: &mut CodegenContext,
    indent: usize,
) {
    let ind = "  ".repeat(indent);

    builder.push_str(&ind);
    builder.push_str("// interpolation: {{ ");
    builder.push_str(&interp.expression.content);
    builder.push_str(" }}\n");

    builder.push_str(&ind);
    builder.push_str("(");
    generate_expression(builder, &interp.expression, ctx);
    builder.push_str(");\n");
}

/// Generate code for a conditional (v-if).
fn generate_if(
    builder: &mut CodeBuilder,
    if_node: &IfNode,
    ctx: &mut CodegenContext,
    indent: usize,
) {
    let ind = "  ".repeat(indent);

    for (i, branch) in if_node.branches.iter().enumerate() {
        generate_if_branch(builder, branch, ctx, indent, i == 0);
    }

    builder.push_str(&ind);
    builder.push_str("}\n");
}

/// Generate code for an if branch.
fn generate_if_branch(
    builder: &mut CodeBuilder,
    branch: &IfBranch,
    ctx: &mut CodegenContext,
    indent: usize,
    is_first: bool,
) {
    let ind = "  ".repeat(indent);

    if is_first {
        builder.push_str(&ind);
        builder.push_str("if (");
        if let Some(condition) = &branch.condition {
            generate_expression(builder, condition, ctx);
        }
        builder.push_str(") {\n");
    } else if branch.condition.is_some() {
        builder.push_str(&ind);
        builder.push_str("} else if (");
        if let Some(condition) = &branch.condition {
            generate_expression(builder, condition, ctx);
        }
        builder.push_str(") {\n");
    } else {
        builder.push_str(&ind);
        builder.push_str("} else {\n");
    }

    for child in &branch.children {
        generate_node(builder, child, ctx, indent + 1);
    }
}

/// Generate code for a loop (v-for).
fn generate_for(
    builder: &mut CodeBuilder,
    for_node: &ForNode,
    ctx: &mut CodegenContext,
    indent: usize,
) {
    let ind = "  ".repeat(indent);
    let scope_marker = ctx.enter_scope();

    builder.push_str(&ind);
    builder.push_str("for (const [");

    // Add loop variables to scope
    let value_name = &for_node.value.pattern;
    ctx.add_var(value_name.as_str(), VarSource::VFor);
    builder.push_str(value_name);

    if let Some(key) = &for_node.key {
        ctx.add_var(key.pattern.as_str(), VarSource::VFor);
        builder.push_str(", ");
        builder.push_str(&key.pattern);
    }

    if let Some(index) = &for_node.index {
        ctx.add_var(index.pattern.as_str(), VarSource::VFor);
        builder.push_str(", ");
        builder.push_str(&index.pattern);
    }

    builder.push_str("] of __VLS_getVForSourceType(");
    generate_expression(builder, &for_node.source, ctx);
    builder.push_str(")) {\n");

    for child in &for_node.children {
        generate_node(builder, child, ctx, indent + 1);
    }

    builder.push_str(&ind);
    builder.push_str("}\n");

    ctx.exit_scope(scope_marker);
}

/// Generate code for a slot outlet.
fn generate_slot_outlet(
    builder: &mut CodeBuilder,
    slot: &SlotOutletNode,
    ctx: &mut CodegenContext,
    indent: usize,
) {
    let ind = "  ".repeat(indent);

    builder.push_str(&ind);
    builder.push_str("// slot outlet\n");

    builder.push_str(&ind);
    builder.push_str("__VLS_ctx.$slots[");
    generate_expression(builder, &slot.name, ctx);
    builder.push_str("]?.({\n");

    // Slot props
    for prop in &slot.props {
        builder.push_str(&ind);
        builder.push_str("  ");
        builder.push_str(&prop.name);
        builder.push_str(": ");
        generate_expression(builder, &prop.value, ctx);
        builder.push_str(",\n");
    }

    builder.push_str(&ind);
    builder.push_str("});\n");

    // Fallback content
    if !slot.fallback.is_empty() {
        for child in &slot.fallback {
            generate_node(builder, child, ctx, indent);
        }
    }
}

/// Generate code for an expression.
fn generate_expression(builder: &mut CodeBuilder, expr: &Expression, ctx: &mut CodegenContext) {
    let content = &expr.content;

    // Wrap identifiers with context access
    // This is a simplified version - a full implementation would parse the expression
    let wrapped = wrap_expression_identifiers(content, ctx);

    builder.push_mapped(&wrapped, expr.span.start);
}

/// Wrap identifiers in an expression with context access.
fn wrap_expression_identifiers(expr: &str, ctx: &CodegenContext) -> String {
    let expr = expr.trim();

    // Very simple identifier detection
    // A proper implementation would use AST parsing
    if is_simple_identifier(expr) {
        if is_js_builtin(expr) || ctx.has_var(expr) {
            expr.to_string()
        } else {
            format!("__VLS_ctx.{}", expr)
        }
    } else {
        // For complex expressions, return as-is
        // A full implementation would walk the AST
        expr.to_string()
    }
}

/// Check if a string is a simple identifier.
fn is_simple_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Check if an identifier is a JavaScript builtin.
fn is_js_builtin(name: &str) -> bool {
    matches!(
        name,
        "true"
            | "false"
            | "null"
            | "undefined"
            | "NaN"
            | "Infinity"
            | "this"
            | "console"
            | "window"
            | "document"
            | "Math"
            | "JSON"
            | "Date"
            | "Array"
            | "Object"
            | "String"
            | "Number"
            | "Boolean"
            | "Symbol"
            | "Map"
            | "Set"
            | "WeakMap"
            | "WeakSet"
            | "Promise"
            | "Proxy"
            | "Reflect"
            | "Error"
            | "TypeError"
            | "RangeError"
            | "parseInt"
            | "parseFloat"
            | "isNaN"
            | "isFinite"
            | "encodeURI"
            | "decodeURI"
            | "encodeURIComponent"
            | "decodeURIComponent"
    )
}

/// Extract binding names from a pattern.
fn extract_binding_names(pattern: &str) -> Vec<&str> {
    let pattern = pattern.trim();

    // Handle object destructuring: { a, b: c }
    if pattern.starts_with('{') && pattern.ends_with('}') {
        let inner = &pattern[1..pattern.len() - 1];
        return inner
            .split(',')
            .filter_map(|part| {
                let part = part.trim();
                if let Some(colon_pos) = part.find(':') {
                    Some(part[colon_pos + 1..].trim())
                } else if let Some(eq_pos) = part.find('=') {
                    Some(part[..eq_pos].trim())
                } else {
                    Some(part)
                }
            })
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Handle array destructuring: [a, b]
    if pattern.starts_with('[') && pattern.ends_with(']') {
        let inner = &pattern[1..pattern.len() - 1];
        return inner
            .split(',')
            .map(|part| part.trim())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Simple identifier
    vec![pattern]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_simple_identifier() {
        assert!(is_simple_identifier("foo"));
        assert!(is_simple_identifier("_bar"));
        assert!(is_simple_identifier("$baz"));
        assert!(is_simple_identifier("foo123"));
        assert!(!is_simple_identifier("foo.bar"));
        assert!(!is_simple_identifier("foo()"));
        assert!(!is_simple_identifier("123foo"));
    }

    #[test]
    fn test_is_js_builtin() {
        assert!(is_js_builtin("true"));
        assert!(is_js_builtin("console"));
        assert!(is_js_builtin("Math"));
        assert!(!is_js_builtin("myVar"));
    }

    #[test]
    fn test_extract_binding_names() {
        assert_eq!(extract_binding_names("item"), vec!["item"]);
        assert_eq!(extract_binding_names("{ a, b }"), vec!["a", "b"]);
        assert_eq!(extract_binding_names("{ key: value }"), vec!["value"]);
        assert_eq!(extract_binding_names("[x, y]"), vec!["x", "y"]);
    }
}
