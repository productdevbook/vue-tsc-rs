# vue-tsc-rs

A high-performance Vue type checker written in Rust. Drop-in replacement for `vue-tsc` with significantly faster performance.

## Features

- **Fast**: 10-100x faster than vue-tsc thanks to Rust's performance
- **Compatible**: Full Vue 3.x Single File Component support
- **Complete**: Script setup, generics, macros, and template type checking
- **Watch Mode**: Incremental checking with file watching
- **Multiple Outputs**: Human-readable, JSON, and machine-parseable formats

## Installation

### npm

```bash
npm install -D vue-tsc-rs
```

```bash
pnpm add -D vue-tsc-rs
```

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

### package.json

```json
{
  "scripts": {
    "typecheck": "vue-tsc-rs",
    "typecheck:watch": "vue-tsc-rs --watch"
  }
}
```

### CLI Options

| Option | Description |
|--------|-------------|
| `-w, --workspace <DIR>` | Workspace directory to check |
| `-p, --project <FILE>` | Path to tsconfig.json |
| `--watch` | Run in watch mode |
| `--output <FORMAT>` | Output format: `human`, `human-verbose`, `json`, `machine` |
| `--fail-on-warning` | Exit with error on warnings |
| `--emit-ts` | Emit generated TypeScript files (for debugging) |
| `--timings` | Show timing information |
| `--max-errors <N>` | Maximum number of errors to show |
| `--skip-typecheck` | Skip TypeScript, only run Vue diagnostics |
| `--ignore <PATTERN>` | Ignore patterns (glob) |
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
const props = defineProps<{
  message: string
  count?: number
}>()

const emit = defineEmits<{
  update: [value: string]
  delete: []
}>()

const slots = defineSlots<{
  default(props: { item: string }): any
  header(): any
}>()

const modelValue = defineModel<string>()

defineExpose({
  publicMethod() {}
})
</script>
```

### Template Type Checking

```vue
<template>
  <MyComponent :value="typedValue" @update="handleUpdate" />

  <div v-for="item in items" :key="item.id">
    {{ item.name }}
  </div>

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
└── npm/                         # npm package
```

### Processing Pipeline

```
.vue file
    ↓
[1] Parse SFC (vue-parser)
    ↓
[2] Vue Diagnostics (vue-diagnostics)
    ↓
[3] Compile Template (vue-template-compiler)
    ↓
[4] Generate TypeScript (vue-codegen)
    ↓
[5] TypeScript Check (ts-runner)
    ↓
[6] Output (vue-tsc-rs)
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

### TypeScript Diagnostics

All standard TypeScript errors are reported with positions mapped back to `.vue` files.

## Performance

| Tool | Cold Start | Incremental |
|------|------------|-------------|
| vue-tsc | ~15s | ~5s |
| vue-tsc-rs | ~1.5s | ~0.5s |

*Benchmarked on a medium-sized Vue project (100 components)*

Performance improvements:
- Parallel file processing with Rayon
- Efficient Rust data structures
- Minimal allocations
- Optimized parsing

## Platforms

| Platform | Architecture |
|----------|--------------|
| macOS | arm64, x64 |
| Linux | arm64, x64 |
| Windows | arm64, x64 |

## Development

### Requirements

- Rust 1.75+
- Node.js 24+

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test --workspace
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

## Related

- [vue-tsc](https://github.com/vuejs/language-tools) - Official Vue type checker
- [tsgo](https://github.com/nicolo-ribaudo/tsgo) - Fast TypeScript type checker
