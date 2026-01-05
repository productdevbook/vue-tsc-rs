//! Vue Single File Component parser.
//!
//! This crate provides a parser for Vue SFC files, extracting template,
//! script, scriptSetup, and style blocks.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;

pub use ast::*;
pub use error::{ParseError, ParseResult};
pub use parser::parse_sfc;

/// Parse a Vue SFC file and return the parsed result.
pub fn parse(source: &str) -> ParseResult<Sfc> {
    parse_sfc(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_sfc() {
        let source = r#"<template>
  <div>Hello {{ name }}</div>
</template>

<script setup lang="ts">
const name = ref('World')
</script>

<style scoped>
div { color: red; }
</style>
"#;
        let result = parse(source).unwrap();
        assert!(result.template.is_some());
        assert!(result.script_setup.is_some());
        assert_eq!(result.styles.len(), 1);
        assert!(result.styles[0].scoped);
    }

    #[test]
    fn test_parse_script_only() {
        let source = r#"<script lang="ts">
export default {
  name: 'MyComponent'
}
</script>
"#;
        let result = parse(source).unwrap();
        assert!(result.template.is_none());
        assert!(result.script.is_some());
        assert!(result.script_setup.is_none());
    }

    #[test]
    fn test_parse_with_generic() {
        let source = r#"<script setup lang="ts" generic="T extends string">
defineProps<{ value: T }>()
</script>
"#;
        let result = parse(source).unwrap();
        let script_setup = result.script_setup.unwrap();
        assert_eq!(script_setup.generic.as_deref(), Some("T extends string"));
    }
}
