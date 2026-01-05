# vue-tsc-rs

High-performance Vue type checker written in Rust. Drop-in replacement for `vue-tsc` with 10-100x faster performance.

## Installation

```bash
npm install -D vue-tsc-rs
# or
yarn add -D vue-tsc-rs
# or
pnpm add -D vue-tsc-rs
```

## Usage

### CLI

```bash
# Check current directory
npx vue-tsc-rs

# Check specific workspace
npx vue-tsc-rs --workspace ./my-vue-project

# Use specific tsconfig
npx vue-tsc-rs -p tsconfig.json

# Watch mode
npx vue-tsc-rs --watch

# JSON output
npx vue-tsc-rs --output json
```

### package.json scripts

```json
{
  "scripts": {
    "typecheck": "vue-tsc-rs",
    "typecheck:watch": "vue-tsc-rs --watch"
  }
}
```

### Programmatic API

```javascript
import { check, run } from 'vue-tsc-rs';

// Simple check
const result = await check('./my-vue-project', {
  output: 'json',
  failOnWarning: true
});

console.log(result.code); // 0 = success, 1 = errors
console.log(result.stdout); // JSON output

// Run with custom arguments
await run(['--workspace', './src', '--verbose']);
```

## CLI Options

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

## Supported Platforms

- macOS (arm64, x64)
- Linux (arm64, x64)
- Windows (arm64, x64)

## Requirements

- Node.js 24+
- TypeScript 5.0+ (for type checking)

## Comparison

| Tool | Cold Start | Incremental |
|------|------------|-------------|
| vue-tsc | ~15s | ~5s |
| vue-tsc-rs | ~1.5s | ~0.5s |

*Benchmarked on a medium-sized Vue project (100 components)*

## License

MIT
