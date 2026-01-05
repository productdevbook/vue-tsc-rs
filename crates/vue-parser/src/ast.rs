//! AST types for Vue Single File Components.

use smol_str::SmolStr;
use source_map::Span;

/// A parsed Vue Single File Component.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sfc {
    /// The full source content.
    pub content: String,
    /// The template block, if present.
    pub template: Option<TemplateBlock>,
    /// The script block (Options API or regular), if present.
    pub script: Option<ScriptBlock>,
    /// The script setup block, if present.
    pub script_setup: Option<ScriptSetupBlock>,
    /// All style blocks.
    pub styles: Vec<StyleBlock>,
    /// Custom blocks (e.g., <i18n>, <docs>).
    pub custom_blocks: Vec<CustomBlock>,
    /// Comments found at the root level.
    pub comments: Vec<Comment>,
}

impl Sfc {
    /// Create a new empty SFC.
    pub fn new(content: String) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }

    /// Check if this SFC uses script setup.
    pub fn has_script_setup(&self) -> bool {
        self.script_setup.is_some()
    }

    /// Get the script language (ts, tsx, js, jsx).
    pub fn script_lang(&self) -> Option<&str> {
        self.script_setup
            .as_ref()
            .and_then(|s| s.lang.as_deref())
            .or_else(|| self.script.as_ref().and_then(|s| s.lang.as_deref()))
    }

    /// Check if the script uses TypeScript.
    pub fn is_typescript(&self) -> bool {
        matches!(self.script_lang(), Some("ts" | "tsx"))
    }
}

/// A block in the SFC with common properties.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SfcBlock {
    /// The span of the entire block including tags.
    pub span: Span,
    /// The span of the content only (excluding tags).
    pub content_span: Span,
    /// The raw content of the block.
    pub content: String,
    /// Block attributes.
    pub attrs: Vec<BlockAttr>,
}

impl SfcBlock {
    /// Get an attribute value by name.
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|a| a.name == name)
            .and_then(|a| a.value.as_deref())
    }

    /// Check if an attribute exists (for boolean attributes).
    pub fn has_attr(&self, name: &str) -> bool {
        self.attrs.iter().any(|a| a.name == name)
    }
}

/// An attribute on a block tag.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockAttr {
    /// The attribute name.
    pub name: SmolStr,
    /// The attribute value (None for boolean attributes).
    pub value: Option<String>,
    /// The span of the attribute.
    pub span: Span,
    /// The span of the value (if present).
    pub value_span: Option<Span>,
}

impl BlockAttr {
    /// Create a new boolean attribute.
    pub fn boolean(name: impl Into<SmolStr>, span: Span) -> Self {
        Self {
            name: name.into(),
            value: None,
            span,
            value_span: None,
        }
    }

    /// Create a new attribute with a value.
    pub fn with_value(
        name: impl Into<SmolStr>,
        value: impl Into<String>,
        span: Span,
        value_span: Span,
    ) -> Self {
        Self {
            name: name.into(),
            value: Some(value.into()),
            span,
            value_span: Some(value_span),
        }
    }
}

/// The template block.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateBlock {
    /// Common block properties.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub block: SfcBlock,
    /// The template language (html, pug, etc.).
    pub lang: Option<String>,
    /// Whether this is a functional template.
    pub functional: bool,
    /// The src attribute for external templates.
    pub src: Option<SrcAttr>,
}

impl std::ops::Deref for TemplateBlock {
    type Target = SfcBlock;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

/// The script block (not setup).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScriptBlock {
    /// Common block properties.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub block: SfcBlock,
    /// The script language (ts, tsx, js, jsx).
    pub lang: Option<String>,
    /// The src attribute for external scripts.
    pub src: Option<SrcAttr>,
}

impl std::ops::Deref for ScriptBlock {
    type Target = SfcBlock;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

/// The script setup block.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScriptSetupBlock {
    /// Common block properties.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub block: SfcBlock,
    /// The script language (ts, tsx, js, jsx).
    pub lang: Option<String>,
    /// Generic type parameters.
    pub generic: Option<String>,
    /// Span of the generic attribute value.
    pub generic_span: Option<Span>,
}

impl std::ops::Deref for ScriptSetupBlock {
    type Target = SfcBlock;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

/// A style block.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleBlock {
    /// Common block properties.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub block: SfcBlock,
    /// The style language (css, scss, less, etc.).
    pub lang: Option<String>,
    /// Whether this is a scoped style.
    pub scoped: bool,
    /// CSS module name (if using module attribute).
    pub module: Option<String>,
    /// The src attribute for external styles.
    pub src: Option<SrcAttr>,
}

impl std::ops::Deref for StyleBlock {
    type Target = SfcBlock;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

/// A custom block (e.g., <i18n>, <docs>).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomBlock {
    /// Common block properties.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub block: SfcBlock,
    /// The block type (tag name).
    pub block_type: SmolStr,
}

impl std::ops::Deref for CustomBlock {
    type Target = SfcBlock;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

/// The src attribute for external files.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SrcAttr {
    /// The src path.
    pub value: String,
    /// The span of the attribute.
    pub span: Span,
    /// The span of the value.
    pub value_span: Span,
}

/// A comment in the SFC.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Comment {
    /// The comment content (without delimiters).
    pub content: String,
    /// The span of the comment.
    pub span: Span,
}

/// Script language variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScriptLang {
    /// JavaScript
    #[default]
    Js,
    /// JavaScript with JSX
    Jsx,
    /// TypeScript
    Ts,
    /// TypeScript with JSX
    Tsx,
}

impl ScriptLang {
    /// Parse from a language string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "js" | "javascript" => Some(Self::Js),
            "jsx" => Some(Self::Jsx),
            "ts" | "typescript" => Some(Self::Ts),
            "tsx" => Some(Self::Tsx),
            _ => None,
        }
    }

    /// Get the file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Js => "js",
            Self::Jsx => "jsx",
            Self::Ts => "ts",
            Self::Tsx => "tsx",
        }
    }

    /// Check if this is TypeScript.
    pub fn is_typescript(&self) -> bool {
        matches!(self, Self::Ts | Self::Tsx)
    }

    /// Check if this supports JSX.
    pub fn is_jsx(&self) -> bool {
        matches!(self, Self::Jsx | Self::Tsx)
    }
}
