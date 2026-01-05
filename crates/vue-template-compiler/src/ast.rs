//! AST types for Vue templates.

use indexmap::IndexMap;
use smol_str::SmolStr;
use source_map::Span;

/// The root of a parsed template.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateAst {
    /// Child nodes of the template.
    pub children: Vec<TemplateNode>,
    /// Hoisted static nodes (for optimization).
    pub hoists: Vec<TemplateNode>,
    /// Template scope variables from parent (for slots).
    pub scope_vars: Vec<ScopeVar>,
    /// Source span of the entire template.
    pub span: Span,
}

impl TemplateAst {
    /// Create a new empty template AST.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a template AST with children.
    pub fn with_children(children: Vec<TemplateNode>, span: Span) -> Self {
        Self {
            children,
            span,
            ..Default::default()
        }
    }
}

/// A node in the template AST.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TemplateNode {
    /// An element (HTML or component).
    Element(ElementNode),
    /// A text node.
    Text(TextNode),
    /// An interpolation ({{ expr }}).
    Interpolation(InterpolationNode),
    /// A comment.
    Comment(CommentNode),
    /// A conditional block (v-if/v-else-if/v-else).
    If(IfNode),
    /// A loop block (v-for).
    For(ForNode),
    /// A slot outlet (<slot>).
    SlotOutlet(SlotOutletNode),
    /// A template element with v-slot.
    Template(TemplateElementNode),
}

impl TemplateNode {
    /// Get the span of this node.
    pub fn span(&self) -> Span {
        match self {
            Self::Element(n) => n.span,
            Self::Text(n) => n.span,
            Self::Interpolation(n) => n.span,
            Self::Comment(n) => n.span,
            Self::If(n) => n.span,
            Self::For(n) => n.span,
            Self::SlotOutlet(n) => n.span,
            Self::Template(n) => n.span,
        }
    }
}

/// An element node (HTML element or component).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElementNode {
    /// The tag name.
    pub tag: SmolStr,
    /// Whether this is a component (PascalCase or registered).
    pub is_component: bool,
    /// Static attributes.
    pub attrs: Vec<Attribute>,
    /// Dynamic attributes and directives.
    pub directives: Vec<Directive>,
    /// Props (for components).
    pub props: Vec<Prop>,
    /// Event listeners.
    pub events: Vec<EventListener>,
    /// Child nodes.
    pub children: Vec<TemplateNode>,
    /// Named slots (for components).
    pub slots: IndexMap<SmolStr, SlotNode>,
    /// Self-closing tag.
    pub self_closing: bool,
    /// Source span.
    pub span: Span,
    /// Span of the tag name.
    pub tag_span: Span,
}

impl ElementNode {
    /// Check if this element has a specific directive.
    pub fn has_directive(&self, name: &str) -> bool {
        self.directives.iter().any(|d| d.name == name)
    }

    /// Get a directive by name.
    pub fn get_directive(&self, name: &str) -> Option<&Directive> {
        self.directives.iter().find(|d| d.name == name)
    }

    /// Check if this is a built-in element.
    pub fn is_builtin(&self) -> bool {
        matches!(
            self.tag.as_str(),
            "template"
                | "slot"
                | "component"
                | "keep-alive"
                | "transition"
                | "transition-group"
                | "teleport"
                | "suspense"
        )
    }
}

/// A static attribute.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attribute {
    /// Attribute name.
    pub name: SmolStr,
    /// Attribute value.
    pub value: Option<String>,
    /// Source span.
    pub span: Span,
    /// Value span.
    pub value_span: Option<Span>,
}

/// A directive (v-*, :, @, #).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Directive {
    /// Directive name (without v- prefix).
    pub name: SmolStr,
    /// Directive argument (e.g., v-bind:arg).
    pub arg: Option<DirectiveArg>,
    /// Modifiers (e.g., .prevent, .stop).
    pub modifiers: Vec<SmolStr>,
    /// Expression value.
    pub value: Option<Expression>,
    /// Source span.
    pub span: Span,
}

impl Directive {
    /// Check if this is a v-bind directive.
    pub fn is_bind(&self) -> bool {
        self.name == "bind"
    }

    /// Check if this is a v-on directive.
    pub fn is_on(&self) -> bool {
        self.name == "on"
    }

    /// Check if this is a v-model directive.
    pub fn is_model(&self) -> bool {
        self.name == "model"
    }

    /// Check if this is a v-slot directive.
    pub fn is_slot(&self) -> bool {
        self.name == "slot"
    }
}

/// A directive argument.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DirectiveArg {
    /// Static argument (e.g., v-bind:foo).
    Static(SmolStr, Span),
    /// Dynamic argument (e.g., v-bind:[foo]).
    Dynamic(Expression),
}

impl DirectiveArg {
    /// Get the argument as a static string, if it is one.
    pub fn as_static(&self) -> Option<&str> {
        match self {
            Self::Static(s, _) => Some(s.as_str()),
            Self::Dynamic(_) => None,
        }
    }
}

/// A prop for a component.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Prop {
    /// Prop name.
    pub name: SmolStr,
    /// Prop value expression.
    pub value: Expression,
    /// Whether this is a dynamic prop name.
    pub is_dynamic: bool,
    /// Source span.
    pub span: Span,
}

/// An event listener.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventListener {
    /// Event name.
    pub name: SmolStr,
    /// Handler expression.
    pub handler: Expression,
    /// Whether this is a dynamic event name.
    pub is_dynamic: bool,
    /// Modifiers.
    pub modifiers: Vec<SmolStr>,
    /// Source span.
    pub span: Span,
}

/// A slot node (content for a named slot).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotNode {
    /// Slot name.
    pub name: SmolStr,
    /// Slot props expression (for scoped slots).
    pub props: Option<SlotProps>,
    /// Slot content.
    pub children: Vec<TemplateNode>,
    /// Source span.
    pub span: Span,
}

/// Slot props for scoped slots.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotProps {
    /// The props expression or destructuring pattern.
    pub pattern: String,
    /// Source span.
    pub span: Span,
}

/// A text node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextNode {
    /// The text content.
    pub content: String,
    /// Source span.
    pub span: Span,
}

/// An interpolation node ({{ expr }}).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterpolationNode {
    /// The expression.
    pub expression: Expression,
    /// Source span.
    pub span: Span,
}

/// A comment node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommentNode {
    /// The comment content.
    pub content: String,
    /// Source span.
    pub span: Span,
}

/// A conditional node (v-if/v-else-if/v-else).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfNode {
    /// Branches of the conditional.
    pub branches: Vec<IfBranch>,
    /// Source span.
    pub span: Span,
}

/// A branch in a conditional.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfBranch {
    /// The condition (None for v-else).
    pub condition: Option<Expression>,
    /// The branch type.
    pub branch_type: IfBranchType,
    /// Child nodes.
    pub children: Vec<TemplateNode>,
    /// Source span.
    pub span: Span,
}

/// Type of if branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IfBranchType {
    /// v-if
    If,
    /// v-else-if
    ElseIf,
    /// v-else
    Else,
}

/// A for loop node (v-for).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForNode {
    /// The source expression (iterable).
    pub source: Expression,
    /// The value alias.
    pub value: ForAlias,
    /// The key alias (optional).
    pub key: Option<ForAlias>,
    /// The index alias (optional).
    pub index: Option<ForAlias>,
    /// Child nodes.
    pub children: Vec<TemplateNode>,
    /// The key attribute expression (for optimization).
    pub key_attr: Option<Expression>,
    /// Source span.
    pub span: Span,
}

/// An alias in a v-for expression.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForAlias {
    /// The alias name or pattern.
    pub pattern: String,
    /// Source span.
    pub span: Span,
}

/// A slot outlet (<slot>).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotOutletNode {
    /// Slot name expression.
    pub name: Expression,
    /// Slot props (passed to scoped slot).
    pub props: Vec<Prop>,
    /// Fallback content.
    pub fallback: Vec<TemplateNode>,
    /// Source span.
    pub span: Span,
}

/// A template element (for v-slot, v-for, v-if without wrapper).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateElementNode {
    /// Directives on the template.
    pub directives: Vec<Directive>,
    /// Child nodes.
    pub children: Vec<TemplateNode>,
    /// Source span.
    pub span: Span,
}

/// A JavaScript/TypeScript expression.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Expression {
    /// The raw expression text.
    pub content: String,
    /// Source span.
    pub span: Span,
    /// Whether this expression is static (no runtime evaluation needed).
    pub is_static: bool,
    /// Identifiers referenced in this expression.
    pub identifiers: Vec<Identifier>,
}

impl Expression {
    /// Create a new expression.
    pub fn new(content: impl Into<String>, span: Span) -> Self {
        Self {
            content: content.into(),
            span,
            is_static: false,
            identifiers: Vec::new(),
        }
    }

    /// Create a static expression.
    pub fn static_expr(content: impl Into<String>, span: Span) -> Self {
        Self {
            content: content.into(),
            span,
            is_static: true,
            identifiers: Vec::new(),
        }
    }
}

/// An identifier referenced in an expression.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Identifier {
    /// The identifier name.
    pub name: SmolStr,
    /// Source span.
    pub span: Span,
}

/// A scope variable introduced by parent.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScopeVar {
    /// Variable name.
    pub name: SmolStr,
    /// Source (e.g., "v-for", "slot-props").
    pub source: SmolStr,
    /// Source span where it was defined.
    pub span: Span,
}

/// Element types for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    /// Native HTML element.
    Element,
    /// Vue component.
    Component,
    /// Built-in Vue element (slot, template, etc.).
    Builtin,
}

/// Determine the element type from a tag name.
pub fn get_element_type(tag: &str) -> ElementType {
    // Built-in Vue elements
    if matches!(
        tag,
        "template"
            | "slot"
            | "component"
            | "keep-alive"
            | "transition"
            | "transition-group"
            | "teleport"
            | "suspense"
    ) {
        return ElementType::Builtin;
    }

    // Components are PascalCase or have dashes
    if tag.chars().next().is_some_and(|c| c.is_uppercase()) || tag.contains('-') {
        return ElementType::Component;
    }

    // Assume HTML element
    ElementType::Element
}
