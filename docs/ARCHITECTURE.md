# Architecture

This document describes the internal architecture of vue-tsc-rs.

## Overview

vue-tsc-rs follows a pipeline architecture similar to the original vue-tsc but implemented in Rust for performance.

```
┌─────────────────────────────────────────────────────────────────┐
│                         vue-tsc-rs CLI                          │
│                     (crates/vue-tsc-rs)                        │
├─────────────────────────────────────────────────────────────────┤
│                         Orchestrator                            │
│  - File discovery                                               │
│  - Parallel processing                                          │
│  - Result aggregation                                           │
└───────────┬─────────────────────────────────────────────────────┘
            │
            ▼
┌───────────────────────────────────────────────────────────────────┐
│                      Processing Pipeline                          │
├───────────────────┬───────────────────┬──────────────────────────┤
│   vue-parser      │  vue-template-    │    vue-codegen           │
│                   │  compiler         │                          │
│  - SFC parsing    │  - Template AST   │  - TS generation         │
│  - Block extract  │  - Directives     │  - Source maps           │
│  - Attributes     │  - Scope tracking │  - Type helpers          │
└───────────────────┴───────────────────┴──────────────────────────┘
            │
            ▼
┌───────────────────────────────────────────────────────────────────┐
│                        Diagnostics                                │
├───────────────────────────────┬──────────────────────────────────┤
│     vue-diagnostics           │         ts-runner                │
│                               │                                  │
│  - Component checks           │  - Virtual file system           │
│  - Template validation        │  - tsc/tsgo execution            │
│  - Macro validation           │  - Diagnostic parsing            │
│                               │  - Position remapping            │
└───────────────────────────────┴──────────────────────────────────┘
            │
            ▼
┌───────────────────────────────────────────────────────────────────┐
│                         source-map                                │
│                                                                   │
│  - Position tracking (Span, LineCol)                             │
│  - Source mappings (generated ↔ original)                        │
│  - Code builder with mapping support                             │
└───────────────────────────────────────────────────────────────────┘
```

## Crate Descriptions

### source-map

Foundation crate for position tracking:

```rust
// Span represents a byte range in source
pub struct Span {
    pub start: u32,
    pub end: u32,
}

// LineIndex converts between offsets and line/column
pub struct LineIndex {
    line_starts: Vec<u32>,
}

// SourceMap maps generated positions to original
pub struct SourceMap {
    mappings: Vec<SourceMapping>,
}

// CodeBuilder generates code with mappings
pub struct CodeBuilder {
    code: String,
    source_map: SourceMap,
}
```

### vue-parser

Parses Vue Single File Components:

```rust
pub struct Sfc {
    pub template: Option<TemplateBlock>,
    pub script: Option<ScriptBlock>,
    pub script_setup: Option<ScriptSetupBlock>,
    pub styles: Vec<StyleBlock>,
    pub custom_blocks: Vec<CustomBlock>,
}

pub struct ScriptSetupBlock {
    pub block: SfcBlock,
    pub lang: Option<String>,
    pub generic: Option<String>,  // <script setup generic="T">
}
```

Key features:
- Handles all SFC block types
- Extracts attributes (lang, scoped, module, src, generic)
- Preserves source positions for all constructs
- Error recovery for malformed input

### vue-template-compiler

Compiles Vue templates to AST:

```rust
pub enum TemplateNode {
    Element(ElementNode),
    Text(TextNode),
    Interpolation(InterpolationNode),  // {{ expr }}
    If(IfNode),                         // v-if/v-else-if/v-else
    For(ForNode),                       // v-for
    SlotOutlet(SlotOutletNode),         // <slot>
    Template(TemplateElementNode),      // <template v-slot>
}

pub struct ElementNode {
    pub tag: SmolStr,
    pub is_component: bool,
    pub attrs: Vec<Attribute>,
    pub directives: Vec<Directive>,
    pub props: Vec<Prop>,              // :prop bindings
    pub events: Vec<EventListener>,    // @event bindings
    pub children: Vec<TemplateNode>,
    pub slots: IndexMap<SmolStr, SlotNode>,
}
```

Key features:
- Full directive support (v-if, v-for, v-model, v-slot, etc.)
- Component vs element detection
- Scope variable tracking
- Expression parsing

### vue-codegen

Generates TypeScript code for type checking:

```rust
pub fn generate(sfc: &Sfc, options: &CodegenOptions) -> CodegenResult {
    // 1. Generate helper types
    generate_helpers(&mut builder, &ctx);

    // 2. Generate script content
    if let Some(script) = &sfc.script {
        generate_script(&mut builder, script, &mut ctx);
    }

    // 3. Generate script setup
    if let Some(script_setup) = &sfc.script_setup {
        generate_script_setup(&mut builder, script_setup, sfc, &mut ctx);
    }

    // 4. Generate template type checking
    if let Some(template) = &sfc.template {
        generate_template(&mut builder, &ast, &mut ctx);
    }

    // 5. Generate component export
    generate_component_export(&mut builder, sfc, &ctx);
}
```

Generated code example:
```typescript
// Helper types
type __VLS_Prettify<T> = { [K in keyof T]: T[K] } & {};

// Setup function
function __VLS_setup() {
  const __VLS_props = defineProps<{ message: string }>();
  const __VLS_emit = defineEmits<{ (e: 'update'): void }>();

  // Original script content with mappings
  const count = ref(0)

  return {};
}

// Template type checking
function __VLS_template() {
  const __VLS_ctx = {} as __VLS_TemplateContext;

  // Type check interpolations
  (__VLS_ctx.count);

  // Type check props
  (__VLS_ctx.message);
}

export default __VLS_defineComponent({
  props: {} as typeof __VLS_props,
  emits: {} as typeof __VLS_emit,
  setup: __VLS_setup,
});
```

### vue-diagnostics

Vue-specific linting and validation:

```rust
pub fn diagnose_sfc(sfc: &Sfc, options: &DiagnosticOptions) -> Vec<Diagnostic> {
    // Component-level checks
    check_component_name(name);
    check_duplicate_macros(script_setup);

    // Template checks
    check_v_for_keys(for_node);
    check_v_model_usage(element);
    check_unknown_components(element);
    check_unknown_directives(directive);
}
```

### ts-runner

TypeScript compiler integration:

```rust
pub struct TsRunner {
    workspace: PathBuf,
    vfs: VirtualFileSystem,      // Manages generated .ts files
    remapper: DiagnosticRemapper, // Maps positions back to .vue
}

impl TsRunner {
    pub async fn run(&self) -> TsResult<TsDiagnostics> {
        // 1. Generate virtual TypeScript files
        self.generate_virtual_files()?;

        // 2. Run TypeScript compiler
        let output = self.run_tsc().await?;

        // 3. Parse diagnostics
        let diagnostics = parse_ts_output(&output);

        // 4. Remap to original positions
        self.remapper.remap_all(&mut diagnostics);

        Ok(diagnostics)
    }
}
```

### vue-tsc-rs (CLI)

Main application orchestrator:

```rust
pub struct Orchestrator {
    config: Config,
    args: Args,
    formatter: OutputFormatter,
}

impl Orchestrator {
    pub async fn run_single_check(&mut self) -> Result<CheckResult> {
        // 1. Find Vue files
        let vue_files = self.find_vue_files()?;

        // 2. Run Vue diagnostics (parallel)
        let vue_diagnostics = self.run_vue_diagnostics(&vue_files)?;

        // 3. Run TypeScript check
        let ts_diagnostics = self.run_ts_check().await?;

        // 4. Output results
        self.output_results(&vue_files, &vue_diagnostics, &ts_diagnostics)
    }
}
```

## Data Flow

### Single File Processing

```
Input: App.vue
┌────────────────────────────────────────────────────┐
│ <script setup lang="ts">                           │
│ const msg = ref('Hello')                           │
│ </script>                                          │
│                                                    │
│ <template>                                         │
│   <div>{{ msg }}</div>                             │
│ </template>                                        │
└────────────────────────────────────────────────────┘
                    │
                    ▼
            ┌───────────────┐
            │  vue-parser   │
            └───────┬───────┘
                    │
                    ▼
┌────────────────────────────────────────────────────┐
│ Sfc {                                              │
│   script_setup: ScriptSetupBlock {                 │
│     lang: Some("ts"),                              │
│     content: "const msg = ref('Hello')",           │
│   },                                               │
│   template: TemplateBlock {                        │
│     content: "<div>{{ msg }}</div>",               │
│   },                                               │
│ }                                                  │
└────────────────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
┌───────────────┐       ┌───────────────┐
│ vue-template- │       │ vue-codegen   │
│ compiler      │       │ (script)      │
└───────┬───────┘       └───────┬───────┘
        │                       │
        ▼                       │
┌───────────────┐               │
│ TemplateAst   │               │
└───────┬───────┘               │
        │                       │
        ▼                       │
┌───────────────┐               │
│ vue-codegen   │◄──────────────┘
│ (template)    │
└───────┬───────┘
        │
        ▼
┌────────────────────────────────────────────────────┐
│ // Generated: App.vue.ts                           │
│ function __VLS_setup() {                           │
│   const msg = ref('Hello')  // ← source mapped     │
│   return {};                                       │
│ }                                                  │
│                                                    │
│ function __VLS_template() {                        │
│   (__VLS_ctx.msg);  // ← type checks 'msg'         │
│ }                                                  │
└────────────────────────────────────────────────────┘
                    │
                    ▼
            ┌───────────────┐
            │  ts-runner    │
            │  (tsc/tsgo)   │
            └───────┬───────┘
                    │
                    ▼
┌────────────────────────────────────────────────────┐
│ Diagnostics (remapped to App.vue):                 │
│ - App.vue:2:7 error TS2322: Type 'string'...       │
└────────────────────────────────────────────────────┘
```

## Parallelization

vue-tsc-rs uses Rayon for parallel processing:

```rust
// Parallel file processing
vue_files.par_iter().for_each(|file| {
    // Parse
    let sfc = vue_parser::parse(&content)?;

    // Run Vue diagnostics
    let diagnostics = diagnose_sfc(&sfc, options);

    // Generate TypeScript
    let result = vue_codegen::generate(&sfc, options);

    // Collect results thread-safely
    results.lock().unwrap().push((file, diagnostics, result));
});
```

## Memory Efficiency

Key optimizations:
- `SmolStr` for short strings (identifiers, tag names)
- `IndexMap` for ordered maps with fast lookup
- `rustc-hash` for faster hashing
- Lazy evaluation where possible
- Minimal cloning with references

## Error Handling

All public APIs return `Result`:

```rust
// Never panic on user input
pub fn parse_sfc(source: &str) -> ParseResult<Sfc> {
    let mut parser = SfcParser::new(source);
    parser.parse()  // Returns Result, handles malformed input
}
```

Error types provide context:

```rust
pub struct ParseError {
    pub message: String,
    pub span: Span,        // Where in the source
    pub code: ErrorCode,   // Categorization
}
```
