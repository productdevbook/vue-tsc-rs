//! Script code generation.

use crate::context::CodegenContext;
use source_map::CodeBuilder;
use vue_parser::ScriptBlock;

/// Generate code for a regular script block.
pub fn generate_script(builder: &mut CodeBuilder, script: &ScriptBlock, _ctx: &mut CodegenContext) {
    // Add the script content with source mappings
    let content_start = script.content_span.start;

    // Check if this is a default export or module-level code
    let content = &script.content;

    // Output the script content
    builder.push_str("// Script block\n");
    builder.push_mapped(content, content_start);
    builder.newline();
    builder.newline();
}

/// Parse script ranges to find important constructs.
#[derive(Debug, Clone, Default)]
pub struct ScriptRanges {
    /// The export default span.
    pub export_default: Option<source_map::Span>,
    /// Import declarations.
    pub imports: Vec<ImportInfo>,
    /// Whether this uses Options API.
    pub is_options_api: bool,
}

/// Information about an import.
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// The import source.
    pub source: String,
    /// The span of the import.
    pub span: source_map::Span,
    /// Imported names.
    pub names: Vec<ImportedName>,
}

/// An imported name.
#[derive(Debug, Clone)]
pub struct ImportedName {
    /// The local name.
    pub local: String,
    /// The imported name (different if renamed).
    pub imported: Option<String>,
}

/// Analyze a script block to extract ranges.
pub fn analyze_script(content: &str) -> ScriptRanges {
    let mut ranges = ScriptRanges::default();

    // Simple heuristic checks
    if content.contains("export default") {
        ranges.is_options_api = content.contains("defineComponent")
            || content.contains("components:")
            || content.contains("props:")
            || content.contains("data()")
            || content.contains("data:")
            || content.contains("methods:")
            || content.contains("computed:");
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_options_api() {
        let content = r#"
import { defineComponent } from 'vue'
export default defineComponent({
    props: {
        msg: String
    },
    data() {
        return { count: 0 }
    }
})
"#;
        let ranges = analyze_script(content);
        assert!(ranges.is_options_api);
    }

    #[test]
    fn test_analyze_not_options_api() {
        let content = r#"
export const foo = 'bar'
"#;
        let ranges = analyze_script(content);
        assert!(!ranges.is_options_api);
    }
}
