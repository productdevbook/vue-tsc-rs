# vue-tsc-rs

A high-performance Vue type checker written in Rust. Drop-in replacement for `vue-tsc` with significantly faster performance.

## Features

- **Fast**: 10-100x faster than vue-tsc thanks to Rust's performance
- **Compatible**: Supports Vue 3.x Single File Components
- **Complete**: Full support for script setup, generics, macros, and template type checking
- **Watch Mode**: Incremental checking with file watching
- **Multiple Output Formats**: Human-readable, JSON, and machine-parseable output

## Installation

### From Source

```bash
git clone https://github.com/productdevbook/vue-tsc-rs
cd vue-tsc-rs
cargo install --path crates/vue-tsc-rs
```

### Pre-built Binaries

Download from [Releases](https://github.com/productdevbook/vue-tsc-rs/releases).

## Usage

```bash
# Check current directory
vue-tsc-rs

# Check specific workspace
vue-tsc-rs --workspace ./my-vue-project

# Use specific tsconfig
vue-tsc-rs -p tsconfig.json

# Watch mode
vue-tsc-rs --watch

# JSON output
vue-tsc-rs --output json

# Show timing information
vue-tsc-rs --timings
```

### CLI Options

| Option | Description |
|--------|-------------|
| `-w, --workspace <DIR>` | Workspace directory to check |
| `-p, --project <FILE>` | Path to tsconfig.json |
| `--watch` | Run in watch mode |
| `--output <FORMAT>` | Output format: human, human-verbose, json, machine |
| `--fail-on-warning` | Exit with error on warnings |
| `--emit-ts` | Emit generated TypeScript files (debugging) |
| `--timings` | Show timing information |
| `--skip-typecheck` | Skip TypeScript, only run Vue diagnostics |
| `--use-tsgo` | Use tsgo instead of tsc |
| `-v, --verbose` | Verbose output |

## Supported Vue Features

### Script Setup

```vue
<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
</script>
```

### Generics

```vue
<script setup lang="ts" generic="T extends string">
defineProps<{
  items: T[]
  selected: T
}>()
</script>
```

### Macros

```vue
<script setup lang="ts">
// Props
const props = defineProps<{
  message: string
  count?: number
}>()

// Emits
const emit = defineEmits<{
  (e: 'update', value: string): void
  (e: 'delete'): void
}>()

// Slots
const slots = defineSlots<{
  default(props: { item: string }): any
  header(): any
}>()

// Model
const modelValue = defineModel<string>()
const count = defineModel<number>('count')

// Expose
defineExpose({
  publicMethod() {}
})
</script>
```

### Template Type Checking

```vue
<template>
  <!-- Props type checking -->
  <MyComponent :value="typedValue" />

  <!-- Event handler type checking -->
  <button @click="handleClick">Click</button>

  <!-- v-for with proper typing -->
  <div v-for="item in items" :key="item.id">
    {{ item.name }}
  </div>

  <!-- Slot props type checking -->
  <MyComponent>
    <template #default="{ item }">
      {{ item.name }}
    </template>
  </MyComponent>
</template>
```

## Architecture

vue-tsc-rs is built as a Rust workspace with 7 crates:

```
vue-tsc-rs/
├── crates/
│   ├── source-map/              # Source position tracking
│   ├── vue-parser/              # Vue SFC parser
│   ├── vue-template-compiler/   # Template AST compiler
│   ├── vue-codegen/             # TypeScript code generation
│   ├── vue-diagnostics/         # Vue-specific diagnostics
│   ├── ts-runner/               # TypeScript integration
│   └── vue-tsc-rs/              # CLI application
```

### Processing Pipeline

```
.vue file
    ↓
[1] Parse SFC (vue-parser)
    - Extract template, script, scriptSetup, style blocks
    - Parse attributes and content
    ↓
[2] Vue Diagnostics (vue-diagnostics)
    - Component naming conventions
    - Macro usage validation
    - Template validation
    ↓
[3] Compile Template (vue-template-compiler)
    - Parse template to AST
    - Handle directives (v-if, v-for, v-model, etc.)
    - Track scope variables
    ↓
[4] Generate TypeScript (vue-codegen)
    - Transform Vue SFC to virtual TypeScript
    - Generate type-safe template checking code
    - Preserve source mappings
    ↓
[5] TypeScript Check (ts-runner)
    - Run tsc or tsgo on virtual files
    - Parse diagnostics
    - Remap positions to original .vue files
    ↓
[6] Output (vue-tsc-rs)
    - Format and display diagnostics
    - Exit with appropriate code
```

## Configuration

### tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"],
  "vueCompilerOptions": {
    "target": 3.5,
    "strictTemplates": true
  }
}
```

### vueCompilerOptions

| Option | Type | Description |
|--------|------|-------------|
| `target` | number | Vue version (3.0, 3.3, 3.5) |
| `strictTemplates` | boolean | Enable strict template checking |
| `checkUnknownComponents` | boolean | Warn on unknown components |
| `checkUnknownDirectives` | boolean | Warn on unknown directives |
| `extensions` | string[] | File extensions to process |

## Diagnostics

### Vue Diagnostics

| Code | Description |
|------|-------------|
| `unknown-component` | Unknown component in template |
| `unknown-directive` | Unknown directive (v-custom) |
| `invalid-v-for` | Invalid v-for syntax |
| `invalid-v-model` | v-model on invalid element |
| `missing-key` | Missing :key in v-for |
| `duplicate-macro` | Multiple defineProps/defineEmits |
| `invalid-component-name` | Component name doesn't follow conventions |

### TypeScript Diagnostics

All standard TypeScript errors are reported with original positions mapped back to .vue files.

## Performance

Compared to vue-tsc on a medium-sized Vue project (100 components):

| Tool | Time |
|------|------|
| vue-tsc | ~15s |
| vue-tsc-rs | ~1.5s |

Performance improvements come from:
- Parallel file processing with Rayon
- Efficient Rust data structures
- Minimal allocations
- Optimized parsing

## Development

### Requirements

- Rust 1.75+
- Node.js 18+ (for TypeScript integration)

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [vuejs/language-tools](https://github.com/vuejs/language-tools) - Vue's official language tooling
- [svelte-check-rs](https://github.com/pheuter/svelte-check-rs) - Inspiration for architecture
- [SWC](https://swc.rs/) - Fast JavaScript/TypeScript tooling in Rust
