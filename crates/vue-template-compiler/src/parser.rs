//! Parser for Vue templates.

use crate::ast::*;
use crate::error::{CompileError, CompileErrorCode, CompileResult};
use source_map::Span;
use smol_str::SmolStr;

/// Parse a Vue template into an AST.
pub fn parse_template(source: &str) -> CompileResult<TemplateAst> {
    let mut parser = TemplateParser::new(source);
    parser.parse()
}

/// Parser for Vue templates.
#[allow(dead_code)]
struct TemplateParser<'a> {
    source: &'a str,
    pos: usize,
    errors: Vec<CompileError>,
}

impl<'a> TemplateParser<'a> {
    /// Create a new parser.
    fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Parse the template.
    fn parse(&mut self) -> CompileResult<TemplateAst> {
        let children = self.parse_children(None)?;
        let span = Span::new(0, self.source.len() as u32);
        Ok(TemplateAst::with_children(children, span))
    }

    /// Get remaining source.
    fn remaining(&self) -> &'a str {
        &self.source[self.pos..]
    }

    /// Check if at end.
    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    /// Peek at next char.
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Consume next char.
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Check if remaining starts with string.
    fn starts_with(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    /// Consume string if it matches.
    fn consume(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    /// Skip whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read until predicate is false.
    fn read_while<F: Fn(char) -> bool>(&mut self, pred: F) -> &'a str {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if pred(c) {
                self.advance();
            } else {
                break;
            }
        }
        &self.source[start..self.pos]
    }

    /// Read until string is found.
    fn read_until(&mut self, s: &str) -> &'a str {
        let start = self.pos;
        while !self.is_eof() && !self.starts_with(s) {
            self.advance();
        }
        &self.source[start..self.pos]
    }

    /// Parse children until end tag or EOF.
    fn parse_children(&mut self, end_tag: Option<&str>) -> CompileResult<Vec<TemplateNode>> {
        let mut children = Vec::new();

        loop {
            if self.is_eof() {
                break;
            }

            // Check for end tag
            if let Some(tag) = end_tag {
                if self.starts_with("</") {
                    let remaining = &self.remaining()[2..];
                    if remaining
                        .split(|c: char| c.is_whitespace() || c == '>')
                        .next()
                        .is_some_and(|t| t.eq_ignore_ascii_case(tag))
                    {
                        break;
                    }
                }
            }

            // Parse node
            if let Some(node) = self.parse_node()? {
                children.push(node);
            }
        }

        Ok(children)
    }

    /// Parse a single node.
    fn parse_node(&mut self) -> CompileResult<Option<TemplateNode>> {
        // Comment
        if self.starts_with("<!--") {
            return self.parse_comment().map(|n| Some(TemplateNode::Comment(n)));
        }

        // End tag (handled by parent)
        if self.starts_with("</") {
            return Ok(None);
        }

        // Element
        if self.starts_with("<") {
            return self.parse_element().map(Some);
        }

        // Interpolation
        if self.starts_with("{{") {
            return self
                .parse_interpolation()
                .map(|n| Some(TemplateNode::Interpolation(n)));
        }

        // Text
        self.parse_text().map(|n| Some(TemplateNode::Text(n)))
    }

    /// Parse a comment.
    fn parse_comment(&mut self) -> CompileResult<CommentNode> {
        let start = self.pos;
        self.consume("<!--");
        let content = self.read_until("-->");
        self.consume("-->");
        let span = Span::new(start as u32, self.pos as u32);
        Ok(CommentNode {
            content: content.to_string(),
            span,
        })
    }

    /// Parse an interpolation.
    fn parse_interpolation(&mut self) -> CompileResult<InterpolationNode> {
        let start = self.pos;
        self.consume("{{");
        let expr_start = self.pos;
        let content = self.read_until("}}").trim();
        let expr_end = self.pos;
        self.consume("}}");
        let span = Span::new(start as u32, self.pos as u32);
        let expr_span = Span::new(expr_start as u32, expr_end as u32);

        Ok(InterpolationNode {
            expression: Expression::new(content, expr_span),
            span,
        })
    }

    /// Parse a text node.
    fn parse_text(&mut self) -> CompileResult<TextNode> {
        let start = self.pos;
        let mut content = String::new();

        while !self.is_eof() && !self.starts_with("<") && !self.starts_with("{{") {
            if let Some(c) = self.advance() {
                content.push(c);
            }
        }

        let span = Span::new(start as u32, self.pos as u32);
        Ok(TextNode { content, span })
    }

    /// Parse an element.
    fn parse_element(&mut self) -> CompileResult<TemplateNode> {
        let start = self.pos;
        self.consume("<");
        self.skip_whitespace();

        // Read tag name
        let tag_start = self.pos;
        let tag = self
            .read_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':')
            .to_string();
        let tag_span = Span::new(tag_start as u32, self.pos as u32);

        if tag.is_empty() {
            return Err(CompileError::new(
                "Expected tag name",
                Span::new(start as u32, self.pos as u32),
                CompileErrorCode::UnexpectedToken,
            ));
        }

        // Parse attributes
        let (attrs, directives, props, events) = self.parse_attributes()?;

        self.skip_whitespace();

        // Self-closing?
        let self_closing = self.consume("/>");
        if !self_closing {
            self.consume(">");
        }

        // Void elements
        let is_void = is_void_element(&tag);

        // Parse children
        let children = if self_closing || is_void {
            Vec::new()
        } else {
            self.parse_children(Some(&tag))?
        };

        // Consume closing tag
        if !self_closing && !is_void {
            self.skip_whitespace();
            if self.starts_with("</") {
                self.consume("</");
                self.skip_whitespace();
                self.read_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':');
                self.skip_whitespace();
                self.consume(">");
            }
        }

        let span = Span::new(start as u32, self.pos as u32);

        // Check for structural directives
        let v_if = directives.iter().find(|d| d.name == "if");
        let v_else_if = directives.iter().find(|d| d.name == "else-if");
        let v_else = directives.iter().find(|d| d.name == "else");
        let v_for = directives.iter().find(|d| d.name == "for");

        // Handle v-for
        if let Some(dir) = v_for {
            if let Some(ref value) = dir.value {
                let for_node = self.parse_v_for_expression(&value.content, value.span)?;
                let mut for_node = for_node;
                // Check for :key before moving props
                let key_attr = props.iter().find(|p| p.name == "key").map(|p| p.value.clone());

                for_node.children = vec![self.create_element_node(
                    tag.into(),
                    tag_span,
                    attrs,
                    directives.into_iter().filter(|d| d.name != "for").collect(),
                    props,
                    events,
                    children,
                    self_closing,
                    span,
                )];
                for_node.span = span;
                for_node.key_attr = key_attr;

                return Ok(TemplateNode::For(for_node));
            }
        }

        // Handle v-if/v-else-if/v-else
        if v_if.is_some() || v_else_if.is_some() || v_else.is_some() {
            let (branch_type, condition) = if let Some(dir) = v_if {
                (IfBranchType::If, dir.value.clone())
            } else if let Some(dir) = v_else_if {
                (IfBranchType::ElseIf, dir.value.clone())
            } else {
                (IfBranchType::Else, None)
            };

            let filtered_directives: Vec<_> = directives
                .into_iter()
                .filter(|d| d.name != "if" && d.name != "else-if" && d.name != "else")
                .collect();

            let element_node = self.create_element_node(
                tag.into(),
                tag_span,
                attrs,
                filtered_directives,
                props,
                events,
                children,
                self_closing,
                span,
            );

            let branch = IfBranch {
                condition,
                branch_type,
                children: vec![element_node],
                span,
            };

            return Ok(TemplateNode::If(IfNode {
                branches: vec![branch],
                span,
            }));
        }

        // Handle slot element
        if tag == "slot" {
            let name_expr = props
                .iter()
                .find(|p| p.name == "name")
                .map(|p| p.value.clone())
                .unwrap_or_else(|| Expression::static_expr("default", span));

            return Ok(TemplateNode::SlotOutlet(SlotOutletNode {
                name: name_expr,
                props: props.into_iter().filter(|p| p.name != "name").collect(),
                fallback: children,
                span,
            }));
        }

        // Handle template element (for slots, v-if without wrapper, etc.)
        if tag == "template" {
            // Check for v-slot
            let v_slot = directives.iter().find(|d| d.name == "slot");
            if v_slot.is_some() {
                return Ok(TemplateNode::Template(TemplateElementNode {
                    directives,
                    children,
                    span,
                }));
            }
        }

        // Regular element or component
        Ok(self.create_element_node(
            tag.into(),
            tag_span,
            attrs,
            directives,
            props,
            events,
            children,
            self_closing,
            span,
        ))
    }

    /// Create an element node.
    #[allow(clippy::too_many_arguments)]
    fn create_element_node(
        &self,
        tag: SmolStr,
        tag_span: Span,
        attrs: Vec<Attribute>,
        directives: Vec<Directive>,
        props: Vec<Prop>,
        events: Vec<EventListener>,
        children: Vec<TemplateNode>,
        self_closing: bool,
        span: Span,
    ) -> TemplateNode {
        let is_component = get_element_type(&tag) == ElementType::Component;
        TemplateNode::Element(ElementNode {
            tag,
            is_component,
            attrs,
            directives,
            props,
            events,
            children,
            slots: Default::default(),
            self_closing,
            span,
            tag_span,
        })
    }

    /// Parse attributes, directives, props, and events.
    #[allow(clippy::type_complexity)]
    fn parse_attributes(
        &mut self,
    ) -> CompileResult<(Vec<Attribute>, Vec<Directive>, Vec<Prop>, Vec<EventListener>)> {
        let mut attrs = Vec::new();
        let mut directives = Vec::new();
        let mut props = Vec::new();
        let mut events = Vec::new();

        loop {
            self.skip_whitespace();

            if self.is_eof() || self.starts_with(">") || self.starts_with("/>") {
                break;
            }

            let attr_start = self.pos;

            // Read attribute name
            let name = self
                .read_while(|c| {
                    c.is_ascii_alphanumeric()
                        || c == '-'
                        || c == '_'
                        || c == ':'
                        || c == '.'
                        || c == '@'
                        || c == '#'
                        || c == '['
                        || c == ']'
                })
                .to_string();

            if name.is_empty() {
                self.advance();
                continue;
            }

            self.skip_whitespace();

            // Read value if present
            let value = if self.consume("=") {
                self.skip_whitespace();
                Some(self.parse_attribute_value()?)
            } else {
                None
            };

            let span = Span::new(attr_start as u32, self.pos as u32);

            // Parse based on prefix
            if let Some(directive_name) = name.strip_prefix("v-") {
                // Directive: v-name:arg.mod="value"
                let directive = self.parse_directive(directive_name, value, span)?;
                directives.push(directive);
            } else if let Some(prop_name) = name.strip_prefix(':').or_else(|| name.strip_prefix("v-bind:")) {
                // Binding: :prop or v-bind:prop
                let (prop_name, is_dynamic) = parse_prop_name(prop_name);
                if let Some((val, val_span)) = value {
                    props.push(Prop {
                        name: prop_name.into(),
                        value: Expression::new(val, val_span),
                        is_dynamic,
                        span,
                    });
                }
            } else if let Some(event_name) = name.strip_prefix('@').or_else(|| name.strip_prefix("v-on:")) {
                // Event: @event or v-on:event
                let (event_name, modifiers) = parse_event_with_modifiers(event_name);
                let is_dynamic = event_name.starts_with('[') && event_name.ends_with(']');
                let clean_name = if is_dynamic {
                    &event_name[1..event_name.len() - 1]
                } else {
                    event_name
                };
                if let Some((val, val_span)) = value {
                    events.push(EventListener {
                        name: clean_name.into(),
                        handler: Expression::new(val, val_span),
                        is_dynamic,
                        modifiers: modifiers.into_iter().map(SmolStr::from).collect(),
                        span,
                    });
                }
            } else if let Some(slot_name) = name.strip_prefix('#') {
                // Slot shorthand: #name or #[dynamic]
                let directive = Directive {
                    name: "slot".into(),
                    arg: Some(if slot_name.starts_with('[') && slot_name.ends_with(']') {
                        DirectiveArg::Dynamic(Expression::new(
                            &slot_name[1..slot_name.len() - 1],
                            span,
                        ))
                    } else {
                        DirectiveArg::Static(slot_name.into(), span)
                    }),
                    modifiers: Vec::new(),
                    value: value.map(|(v, s)| Expression::new(v, s)),
                    span,
                };
                directives.push(directive);
            } else {
                // Static attribute
                let (attr_value, attr_value_span) = match value {
                    Some((v, s)) => (Some(v), Some(s)),
                    None => (None, None),
                };
                attrs.push(Attribute {
                    name: name.into(),
                    value: attr_value,
                    span,
                    value_span: attr_value_span,
                });
            }
        }

        Ok((attrs, directives, props, events))
    }

    /// Parse an attribute value.
    fn parse_attribute_value(&mut self) -> CompileResult<(String, Span)> {
        let start = self.pos;

        if self.starts_with("\"") || self.starts_with("'") {
            let quote = self.advance().unwrap();
            let value_start = self.pos;
            let mut value = String::new();
            while !self.is_eof() {
                if self.peek() == Some(quote) {
                    break;
                }
                if let Some(c) = self.advance() {
                    value.push(c);
                }
            }
            let value_end = self.pos;
            self.advance(); // consume closing quote
            Ok((value, Span::new(value_start as u32, value_end as u32)))
        } else {
            // Unquoted value
            let value = self
                .read_while(|c| !c.is_whitespace() && c != '>' && c != '/')
                .to_string();
            let span = Span::new(start as u32, self.pos as u32);
            Ok((value, span))
        }
    }

    /// Parse a directive.
    fn parse_directive(
        &mut self,
        name_with_mods: &str,
        value: Option<(String, Span)>,
        span: Span,
    ) -> CompileResult<Directive> {
        // Parse: name:arg.mod1.mod2
        let mut parts: Vec<&str> = name_with_mods.split('.').collect();
        let name_and_arg = parts.remove(0);
        let modifiers: Vec<SmolStr> = parts.iter().map(|s| SmolStr::from(*s)).collect();

        let (name, arg) = if let Some(colon_pos) = name_and_arg.find(':') {
            let name = &name_and_arg[..colon_pos];
            let arg_str = &name_and_arg[colon_pos + 1..];
            let arg = if arg_str.starts_with('[') && arg_str.ends_with(']') {
                DirectiveArg::Dynamic(Expression::new(&arg_str[1..arg_str.len() - 1], span))
            } else {
                DirectiveArg::Static(arg_str.into(), span)
            };
            (name, Some(arg))
        } else {
            (name_and_arg, None)
        };

        Ok(Directive {
            name: name.into(),
            arg,
            modifiers,
            value: value.map(|(v, s)| Expression::new(v, s)),
            span,
        })
    }

    /// Parse a v-for expression.
    fn parse_v_for_expression(&self, expr: &str, span: Span) -> CompileResult<ForNode> {
        // Patterns:
        // item in items
        // (item, index) in items
        // (value, key, index) in items
        let expr = expr.trim();

        let (alias_part, source_part) = if let Some(in_pos) = expr.find(" in ") {
            (&expr[..in_pos], &expr[in_pos + 4..])
        } else if let Some(of_pos) = expr.find(" of ") {
            (&expr[..of_pos], &expr[of_pos + 4..])
        } else {
            return Err(CompileError::new(
                "Invalid v-for expression",
                span,
                CompileErrorCode::InvalidVFor,
            ));
        };

        let alias_part = alias_part.trim();
        let source_part = source_part.trim();

        // Parse aliases
        let (value, key, index) = if alias_part.starts_with('(') && alias_part.ends_with(')') {
            let inner = &alias_part[1..alias_part.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            match parts.len() {
                1 => (
                    ForAlias {
                        pattern: parts[0].to_string(),
                        span,
                    },
                    None,
                    None,
                ),
                2 => (
                    ForAlias {
                        pattern: parts[0].to_string(),
                        span,
                    },
                    Some(ForAlias {
                        pattern: parts[1].to_string(),
                        span,
                    }),
                    None,
                ),
                3 => (
                    ForAlias {
                        pattern: parts[0].to_string(),
                        span,
                    },
                    Some(ForAlias {
                        pattern: parts[1].to_string(),
                        span,
                    }),
                    Some(ForAlias {
                        pattern: parts[2].to_string(),
                        span,
                    }),
                ),
                _ => {
                    return Err(CompileError::new(
                        "Invalid v-for aliases",
                        span,
                        CompileErrorCode::InvalidVFor,
                    ));
                }
            }
        } else {
            (
                ForAlias {
                    pattern: alias_part.to_string(),
                    span,
                },
                None,
                None,
            )
        };

        Ok(ForNode {
            source: Expression::new(source_part, span),
            value,
            key,
            index,
            children: Vec::new(),
            key_attr: None,
            span,
        })
    }
}

/// Check if an element is a void element (self-closing).
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Parse a prop name, handling dynamic syntax.
fn parse_prop_name(name: &str) -> (&str, bool) {
    if name.starts_with('[') && name.ends_with(']') {
        (&name[1..name.len() - 1], true)
    } else {
        // Handle modifiers like .sync, .prop, .camel
        let base = name.split('.').next().unwrap_or(name);
        (base, false)
    }
}

/// Parse event name with modifiers.
fn parse_event_with_modifiers(name: &str) -> (&str, Vec<&str>) {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() > 1 {
        (parts[0], parts[1..].to_vec())
    } else {
        (name, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let ast = parse_template("<div>Hello</div>").unwrap();
        assert_eq!(ast.children.len(), 1);
    }

    #[test]
    fn test_parse_interpolation() {
        let ast = parse_template("{{ message }}").unwrap();
        assert_eq!(ast.children.len(), 1);
        match &ast.children[0] {
            TemplateNode::Interpolation(node) => {
                assert_eq!(node.expression.content.trim(), "message");
            }
            _ => panic!("Expected interpolation"),
        }
    }

    #[test]
    fn test_parse_v_for() {
        let ast = parse_template(r#"<div v-for="item in items" :key="item.id">{{ item }}</div>"#).unwrap();
        assert_eq!(ast.children.len(), 1);
        match &ast.children[0] {
            TemplateNode::For(node) => {
                assert_eq!(node.value.pattern, "item");
                assert_eq!(node.source.content.trim(), "items");
            }
            _ => panic!("Expected for node"),
        }
    }

    #[test]
    fn test_parse_v_if() {
        let ast = parse_template(r#"<div v-if="show">Visible</div>"#).unwrap();
        assert_eq!(ast.children.len(), 1);
        match &ast.children[0] {
            TemplateNode::If(node) => {
                assert_eq!(node.branches.len(), 1);
                assert_eq!(node.branches[0].branch_type, IfBranchType::If);
            }
            _ => panic!("Expected if node"),
        }
    }

    #[test]
    fn test_parse_component() {
        let ast = parse_template(r#"<MyComponent :prop="value" @click="handler" />"#).unwrap();
        assert_eq!(ast.children.len(), 1);
        match &ast.children[0] {
            TemplateNode::Element(node) => {
                assert!(node.is_component);
                assert_eq!(node.tag.as_str(), "MyComponent");
                assert_eq!(node.props.len(), 1);
                assert_eq!(node.events.len(), 1);
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_slot() {
        let ast = parse_template(r#"<slot name="header">Default</slot>"#).unwrap();
        assert_eq!(ast.children.len(), 1);
        match &ast.children[0] {
            TemplateNode::SlotOutlet(node) => {
                assert!(!node.fallback.is_empty());
            }
            _ => panic!("Expected slot outlet"),
        }
    }
}
