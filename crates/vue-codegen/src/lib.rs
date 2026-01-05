//! Vue to TypeScript code generation.
//!
//! This crate generates TypeScript code from Vue SFCs for type checking.
//! It produces virtual TypeScript files that preserve type information
//! from templates, scripts, and style bindings.

pub mod context;
pub mod helpers;
pub mod script;
pub mod template;

use source_map::{CodeBuilder, SourceMap};
use vue_parser::Sfc;

pub use context::CodegenContext;
pub use script::generate_script;
pub use template::generate_template;

/// Result of code generation.
#[derive(Debug, Clone)]
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

/// Script language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScriptLang {
    #[default]
    Ts,
    Tsx,
    Js,
    Jsx,
}

impl ScriptLang {
    /// Get the file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Ts => "ts",
            Self::Tsx => "tsx",
            Self::Js => "js",
            Self::Jsx => "jsx",
        }
    }

    /// Check if TypeScript.
    pub fn is_typescript(&self) -> bool {
        matches!(self, Self::Ts | Self::Tsx)
    }
}

/// A code generation error.
#[derive(Debug, Clone)]
pub struct CodegenError {
    pub message: String,
    pub span: source_map::Span,
}

/// Options for code generation.
#[derive(Debug, Clone, Default)]
pub struct CodegenOptions {
    /// The target Vue version (3.0, 3.3, 3.5, etc.).
    pub target: VueTarget,
    /// Whether to generate strict type checks.
    pub strict: bool,
    /// File name for the SFC.
    pub filename: Option<String>,
}

/// Vue target version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VueTarget {
    /// Vue 3.0
    V3_0,
    /// Vue 3.3 (with defineModel, etc.)
    V3_3,
    /// Vue 3.5 (latest)
    #[default]
    V3_5,
}

/// Generate TypeScript code from a Vue SFC.
pub fn generate(sfc: &Sfc, options: &CodegenOptions) -> CodegenResult {
    let mut ctx = CodegenContext::new(options.clone());
    let mut builder = CodeBuilder::new();

    // Detect script language
    let lang = detect_script_lang(sfc);
    ctx.lang = lang;

    // Generate imports and helpers
    generate_helpers(&mut builder, &ctx);

    // Generate script content
    if let Some(script) = &sfc.script {
        generate_script(&mut builder, script, &mut ctx);
    }

    // Generate script setup content
    if let Some(script_setup) = &sfc.script_setup {
        generate_script_setup(&mut builder, script_setup, sfc, &mut ctx);
    }

    // Generate template type checking code
    if let Some(template) = &sfc.template {
        if let Ok(ast) = vue_template_compiler::parse_template(&template.content) {
            generate_template(&mut builder, &ast, &mut ctx);
        }
    }

    // Generate component export
    generate_component_export(&mut builder, sfc, &ctx);

    let (code, source_map) = builder.finish();

    CodegenResult {
        code,
        source_map,
        lang,
        errors: ctx.errors,
    }
}

/// Detect the script language from an SFC.
fn detect_script_lang(sfc: &Sfc) -> ScriptLang {
    let lang_str = sfc.script_lang().unwrap_or("js");
    match lang_str {
        "ts" | "typescript" => ScriptLang::Ts,
        "tsx" => ScriptLang::Tsx,
        "jsx" => ScriptLang::Jsx,
        _ => ScriptLang::Js,
    }
}

/// Generate helper types and imports.
fn generate_helpers(builder: &mut CodeBuilder, _ctx: &CodegenContext) {
    // Import Vue types
    builder.push_str("import { ");
    builder.push_str("defineComponent as __VLS_defineComponent, ");
    builder.push_str("ref as __VLS_ref, ");
    builder.push_str("computed as __VLS_computed, ");
    builder.push_str("reactive as __VLS_reactive, ");
    builder.push_str("PropType as __VLS_PropType, ");
    builder.push_str("ExtractPropTypes as __VLS_ExtractPropTypes, ");
    builder.push_str("ComponentPublicInstance as __VLS_ComponentPublicInstance ");
    builder.push_str("} from 'vue';\n\n");

    // Helper types
    builder.push_str(helpers::VLS_HELPER_TYPES);
    builder.newline();
}

/// Generate script setup code.
fn generate_script_setup(
    builder: &mut CodeBuilder,
    script_setup: &vue_parser::ScriptSetupBlock,
    _sfc: &Sfc,
    ctx: &mut CodegenContext,
) {
    // Handle generics
    if let Some(generic) = &script_setup.generic {
        ctx.generics = Some(generic.clone());
    }

    // Extract macros from script setup
    let macros = extract_macros(&script_setup.content);
    ctx.macros = macros;

    // Generate the setup function wrapper
    if ctx.generics.is_some() {
        builder.push_str("function __VLS_setup<");
        builder.push_str(ctx.generics.as_deref().unwrap_or(""));
        builder.push_str(">() {\n");
    } else {
        builder.push_str("function __VLS_setup() {\n");
    }

    // Generate macro declarations
    generate_macro_declarations(builder, &ctx.macros, ctx);

    // Output the script content with mappings
    let content_start = script_setup.content_span.start;
    builder.push_mapped(&script_setup.content, content_start);
    builder.newline();

    // Generate return type
    builder.push_str("\nreturn {\n");
    for export in &ctx.macros.exposed {
        builder.push_str("  ");
        builder.push_str(export);
        builder.push_str(",\n");
    }
    builder.push_str("};\n");
    builder.push_str("}\n\n");
}

/// Generate macro declarations (defineProps, defineEmits, etc.).
fn generate_macro_declarations(
    builder: &mut CodeBuilder,
    macros: &MacroInfo,
    _ctx: &CodegenContext,
) {
    // defineProps
    if let Some(props) = &macros.define_props {
        builder.push_str("const __VLS_props = defineProps");
        if let Some(type_arg) = &props.type_arg {
            builder.push_str("<");
            builder.push_str(type_arg);
            builder.push_str(">");
        }
        builder.push_str("();\n");

        // Destructured props
        if let Some(pattern) = &props.destructure_pattern {
            builder.push_str("const ");
            builder.push_str(pattern);
            builder.push_str(" = __VLS_props;\n");
        }
    }

    // defineEmits
    if let Some(emits) = &macros.define_emits {
        builder.push_str("const __VLS_emit = defineEmits");
        if let Some(type_arg) = &emits.type_arg {
            builder.push_str("<");
            builder.push_str(type_arg);
            builder.push_str(">");
        }
        builder.push_str("();\n");
    }

    // defineSlots
    if let Some(slots) = &macros.define_slots {
        builder.push_str("const __VLS_slots = defineSlots");
        if let Some(type_arg) = &slots.type_arg {
            builder.push_str("<");
            builder.push_str(type_arg);
            builder.push_str(">");
        }
        builder.push_str("();\n");
    }

    // defineModel
    for model in &macros.define_models {
        builder.push_str("const ");
        builder.push_str(&model.name);
        builder.push_str(" = defineModel");
        if let Some(type_arg) = &model.type_arg {
            builder.push_str("<");
            builder.push_str(type_arg);
            builder.push_str(">");
        }
        builder.push_str("(");
        if model.name != "modelValue" {
            builder.push_str("'");
            builder.push_str(&model.name);
            builder.push_str("'");
        }
        builder.push_str(");\n");
    }

    // defineExpose
    if let Some(expose) = &macros.define_expose {
        builder.push_str("defineExpose(");
        builder.push_str(&expose.expression);
        builder.push_str(");\n");
    }
}

/// Generate component export.
fn generate_component_export(builder: &mut CodeBuilder, sfc: &Sfc, ctx: &CodegenContext) {
    builder.push_str("\n// Component definition\n");

    if sfc.has_script_setup() {
        // Export the setup-based component
        builder.push_str("export default __VLS_defineComponent({\n");

        // Props type
        if ctx.macros.define_props.is_some() {
            builder.push_str("  props: {} as __VLS_ExtractPropTypes<typeof __VLS_props>,\n");
        }

        // Emits type
        if ctx.macros.define_emits.is_some() {
            builder.push_str("  emits: {} as typeof __VLS_emit,\n");
        }

        builder.push_str("  setup: __VLS_setup,\n");
        builder.push_str("});\n");
    } else if sfc.script.is_some() {
        // Re-export the default export from script
        builder.push_str("// Using Options API component\n");
    } else {
        // Empty component
        builder.push_str("export default __VLS_defineComponent({});\n");
    }
}

/// Extract macro information from script setup content.
fn extract_macros(content: &str) -> MacroInfo {
    let mut info = MacroInfo::default();

    // Simple regex-based extraction (a full implementation would use AST parsing)
    // defineProps
    if let Some(props) = extract_define_props(content) {
        info.define_props = Some(props);
    }

    // defineEmits
    if let Some(emits) = extract_define_emits(content) {
        info.define_emits = Some(emits);
    }

    // defineSlots
    if let Some(slots) = extract_define_slots(content) {
        info.define_slots = Some(slots);
    }

    // defineModel
    info.define_models = extract_define_models(content);

    // defineExpose
    if let Some(expose) = extract_define_expose(content) {
        info.define_expose = Some(expose);
    }

    info
}

fn extract_define_props(content: &str) -> Option<DefinePropsInfo> {
    // Match: defineProps<Type>() or const { ... } = defineProps<Type>()
    let patterns = [
        r"defineProps\s*<([^>]+)>\s*\(\s*\)",
        r"defineProps\s*\(\s*\{([^}]*)\}\s*\)",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(content) {
                return Some(DefinePropsInfo {
                    type_arg: caps.get(1).map(|m| m.as_str().to_string()),
                    destructure_pattern: None,
                });
            }
        }
    }

    // Simple check for presence
    if content.contains("defineProps") {
        return Some(DefinePropsInfo {
            type_arg: None,
            destructure_pattern: None,
        });
    }

    None
}

fn extract_define_emits(content: &str) -> Option<DefineEmitsInfo> {
    if content.contains("defineEmits") {
        // Try to extract type argument
        if let Ok(re) = regex::Regex::new(r"defineEmits\s*<([^>]+)>") {
            if let Some(caps) = re.captures(content) {
                return Some(DefineEmitsInfo {
                    type_arg: caps.get(1).map(|m| m.as_str().to_string()),
                });
            }
        }
        return Some(DefineEmitsInfo { type_arg: None });
    }
    None
}

fn extract_define_slots(content: &str) -> Option<DefineSlotsInfo> {
    if content.contains("defineSlots") {
        if let Ok(re) = regex::Regex::new(r"defineSlots\s*<([^>]+)>") {
            if let Some(caps) = re.captures(content) {
                return Some(DefineSlotsInfo {
                    type_arg: caps.get(1).map(|m| m.as_str().to_string()),
                });
            }
        }
        return Some(DefineSlotsInfo { type_arg: None });
    }
    None
}

fn extract_define_models(content: &str) -> Vec<DefineModelInfo> {
    let mut models = Vec::new();

    if let Ok(re) = regex::Regex::new(r#"defineModel\s*(?:<([^>]+)>)?\s*\(\s*['"]?(\w*)['"]?"#) {
        for caps in re.captures_iter(content) {
            let name = caps
                .get(2)
                .map(|m| m.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("modelValue")
                .to_string();
            let type_arg = caps.get(1).map(|m| m.as_str().to_string());
            models.push(DefineModelInfo { name, type_arg });
        }
    }

    models
}

fn extract_define_expose(content: &str) -> Option<DefineExposeInfo> {
    if let Ok(re) = regex::Regex::new(r"defineExpose\s*\(\s*(\{[^}]*\})") {
        if let Some(caps) = re.captures(content) {
            return Some(DefineExposeInfo {
                expression: caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
            });
        }
    }
    None
}

/// Information about macros in script setup.
#[derive(Debug, Clone, Default)]
pub struct MacroInfo {
    pub define_props: Option<DefinePropsInfo>,
    pub define_emits: Option<DefineEmitsInfo>,
    pub define_slots: Option<DefineSlotsInfo>,
    pub define_models: Vec<DefineModelInfo>,
    pub define_expose: Option<DefineExposeInfo>,
    pub exposed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DefinePropsInfo {
    pub type_arg: Option<String>,
    pub destructure_pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DefineEmitsInfo {
    pub type_arg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DefineSlotsInfo {
    pub type_arg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DefineModelInfo {
    pub name: String,
    pub type_arg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DefineExposeInfo {
    pub expression: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vue_parser::parse_sfc;

    #[test]
    fn test_generate_simple_component() {
        let source = r#"<script setup lang="ts">
const msg = ref('Hello')
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;
        let sfc = parse_sfc(source).unwrap();
        let result = generate(&sfc, &CodegenOptions::default());
        assert!(!result.code.is_empty());
        assert!(result.code.contains("__VLS_setup"));
    }

    #[test]
    fn test_generate_with_props() {
        let source = r#"<script setup lang="ts">
defineProps<{ message: string }>()
</script>
"#;
        let sfc = parse_sfc(source).unwrap();
        let result = generate(&sfc, &CodegenOptions::default());
        assert!(result.code.contains("defineProps"));
    }

    #[test]
    fn test_detect_typescript() {
        let source = r#"<script setup lang="ts">
const x = 1
</script>
"#;
        let sfc = parse_sfc(source).unwrap();
        let result = generate(&sfc, &CodegenOptions::default());
        assert_eq!(result.lang, ScriptLang::Ts);
    }
}
