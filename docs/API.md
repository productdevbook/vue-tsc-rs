# API Reference

This document describes the public APIs of vue-tsc-rs crates.

## source-map

Position tracking and source mapping utilities.

### Span

```rust
/// A span in the source code, representing a half-open range [start, end).
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    /// Create a new span from start and end offsets.
    pub const fn new(start: u32, end: u32) -> Self;

    /// Create an empty span at the given offset.
    pub const fn empty(offset: u32) -> Self;

    /// Get the length of the span.
    pub const fn len(&self) -> u32;

    /// Check if the span is empty.
    pub const fn is_empty(&self) -> bool;

    /// Check if this span contains another span.
    pub const fn contains(&self, other: Span) -> bool;

    /// Check if this span contains an offset.
    pub const fn contains_offset(&self, offset: u32) -> bool;

    /// Merge two spans into one that covers both.
    pub fn merge(self, other: Span) -> Span;
}
```

### LineIndex

```rust
/// A line index for converting between byte offsets and line/column positions.
pub struct LineIndex {
    line_starts: Vec<u32>,
    len: u32,
}

impl LineIndex {
    /// Create a new line index from source text.
    pub fn new(text: &str) -> Self;

    /// Get the line and column for a byte offset.
    pub fn line_col(&self, offset: u32) -> LineCol;

    /// Get the byte offset for a line and column.
    pub fn offset(&self, line_col: LineCol) -> Option<u32>;

    /// Get the number of lines.
    pub fn line_count(&self) -> usize;
}
```

### SourceMap

```rust
/// A source map containing multiple mappings.
pub struct SourceMap {
    mappings: Vec<SourceMapping>,
}

impl SourceMap {
    /// Create a new empty source map.
    pub fn new() -> Self;

    /// Add a simple mapping with equal lengths.
    pub fn add(&mut self, generated_offset: u32, source_offset: u32, length: u32);

    /// Find the source position for a generated offset.
    pub fn find_source(&self, generated_offset: u32) -> Option<&SourceMapping>;

    /// Map a generated offset to a source offset.
    pub fn to_source_offset(&self, generated_offset: u32) -> Option<u32>;
}
```

### CodeBuilder

```rust
/// Builder for generating code with source mappings.
pub struct CodeBuilder {
    code: String,
    source_map: SourceMap,
}

impl CodeBuilder {
    /// Create a new code builder.
    pub fn new() -> Self;

    /// Append code without mapping.
    pub fn push_str(&mut self, code: &str);

    /// Append code with a mapping to the source.
    pub fn push_mapped(&mut self, code: &str, source_offset: u32);

    /// Append a newline.
    pub fn newline(&mut self);

    /// Consume the builder and return the code and source map.
    pub fn finish(self) -> (String, SourceMap);
}
```

## vue-parser

Vue Single File Component parser.

### Main Function

```rust
/// Parse a Vue SFC file and return the parsed result.
pub fn parse(source: &str) -> ParseResult<Sfc>;
```

### Sfc

```rust
/// A parsed Vue Single File Component.
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
}

impl Sfc {
    /// Check if this SFC uses script setup.
    pub fn has_script_setup(&self) -> bool;

    /// Get the script language (ts, tsx, js, jsx).
    pub fn script_lang(&self) -> Option<&str>;

    /// Check if the script uses TypeScript.
    pub fn is_typescript(&self) -> bool;
}
```

### Script Blocks

```rust
/// The script setup block.
pub struct ScriptSetupBlock {
    pub block: SfcBlock,
    /// The script language (ts, tsx, js, jsx).
    pub lang: Option<String>,
    /// Generic type parameters.
    pub generic: Option<String>,
    /// Span of the generic attribute value.
    pub generic_span: Option<Span>,
}

/// The script block (not setup).
pub struct ScriptBlock {
    pub block: SfcBlock,
    /// The script language (ts, tsx, js, jsx).
    pub lang: Option<String>,
    /// The src attribute for external scripts.
    pub src: Option<SrcAttr>,
}
```

### Style Blocks

```rust
/// A style block.
pub struct StyleBlock {
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
```

## vue-template-compiler

Vue template parser and AST.

### Main Function

```rust
/// Compile a Vue template to AST.
pub fn compile(source: &str) -> CompileResult<TemplateAst>;

/// Parse a Vue template into an AST.
pub fn parse_template(source: &str) -> CompileResult<TemplateAst>;
```

### TemplateAst

```rust
/// The root of a parsed template.
pub struct TemplateAst {
    /// Child nodes of the template.
    pub children: Vec<TemplateNode>,
    /// Template scope variables from parent (for slots).
    pub scope_vars: Vec<ScopeVar>,
    /// Source span of the entire template.
    pub span: Span,
}
```

### TemplateNode

```rust
/// A node in the template AST.
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
```

### ElementNode

```rust
/// An element node (HTML element or component).
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
}
```

### Directives

```rust
/// A directive (v-*, :, @, #).
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

/// A directive argument.
pub enum DirectiveArg {
    /// Static argument (e.g., v-bind:foo).
    Static(SmolStr, Span),
    /// Dynamic argument (e.g., v-bind:[foo]).
    Dynamic(Expression),
}
```

## vue-codegen

TypeScript code generation from Vue SFCs.

### Main Function

```rust
/// Generate TypeScript code from a Vue SFC.
pub fn generate(sfc: &Sfc, options: &CodegenOptions) -> CodegenResult;
```

### CodegenResult

```rust
/// Result of code generation.
pub struct CodegenResult {
    /// The generated TypeScript code.
    pub code: String,
    /// Source mappings from generated to original.
    pub source_map: SourceMap,
    /// The detected script language.
    pub lang: ScriptLang,
    /// Errors encountered during code generation.
    pub errors: Vec<CodegenError>,
}
```

### CodegenOptions

```rust
/// Options for code generation.
pub struct CodegenOptions {
    /// The target Vue version (3.0, 3.3, 3.5, etc.).
    pub target: VueTarget,
    /// Whether to generate strict type checks.
    pub strict: bool,
    /// File name for the SFC.
    pub filename: Option<String>,
}
```

## vue-diagnostics

Vue-specific diagnostics.

### Main Function

```rust
/// Run diagnostics on an SFC.
pub fn diagnose_sfc(sfc: &Sfc, options: &DiagnosticOptions) -> Vec<Diagnostic>;

/// Run diagnostics on a template AST.
pub fn diagnose_template(ast: &TemplateAst, options: &DiagnosticOptions) -> Vec<Diagnostic>;
```

### Diagnostic

```rust
/// A diagnostic message.
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
```

### DiagnosticOptions

```rust
/// Options for diagnostics.
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
```

## ts-runner

TypeScript compiler integration.

### TsRunner

```rust
/// TypeScript compiler runner.
pub struct TsRunner {
    workspace: PathBuf,
    options: TsRunnerOptions,
}

impl TsRunner {
    /// Create a new runner.
    pub fn new(workspace: &Path, options: TsRunnerOptions) -> TsResult<Self>;

    /// Run type checking.
    pub async fn run(&self) -> TsResult<TsDiagnostics>;
}
```

### TsRunnerOptions

```rust
/// Options for the TypeScript runner.
pub struct TsRunnerOptions {
    /// The TypeScript configuration.
    pub tsconfig: Option<PathBuf>,
    /// Use tsgo instead of tsc.
    pub use_tsgo: bool,
    /// Additional tsc arguments.
    pub tsc_args: Vec<String>,
    /// Emit output (default: false for type checking only).
    pub emit: bool,
    /// Generate virtual TypeScript files for Vue components.
    pub generate_virtual: bool,
}
```

### TsDiagnostics

```rust
/// A collection of TypeScript diagnostics.
pub struct TsDiagnostics {
    /// All diagnostics.
    pub diagnostics: Vec<TsDiagnostic>,
    /// Total error count.
    pub error_count: usize,
    /// Total warning count.
    pub warning_count: usize,
}

/// A single TypeScript diagnostic.
pub struct TsDiagnostic {
    /// The diagnostic message.
    pub message: String,
    /// The TypeScript error code.
    pub code: u32,
    /// The severity.
    pub severity: TsSeverity,
    /// The file path (if any).
    pub file: Option<PathBuf>,
    /// Line number (1-indexed).
    pub line: Option<u32>,
    /// Column number (1-indexed).
    pub column: Option<u32>,
}
```
