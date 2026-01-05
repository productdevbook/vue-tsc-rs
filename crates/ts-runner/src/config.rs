//! TypeScript configuration handling.

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{TsError, TsResult};

/// TypeScript configuration (tsconfig.json).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    /// Compiler options.
    #[serde(default)]
    pub compiler_options: CompilerOptions,
    /// Include patterns.
    #[serde(default)]
    pub include: Vec<String>,
    /// Exclude patterns.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Files to include.
    #[serde(default)]
    pub files: Vec<String>,
    /// Extends another config.
    #[serde(default)]
    pub extends: Option<String>,
    /// Vue compiler options.
    #[serde(default)]
    pub vue_compiler_options: VueCompilerOptions,
}

impl TsConfig {
    /// Load tsconfig.json from a path.
    pub fn load(path: &Path) -> TsResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            TsError::config(format!("Failed to read {}: {}", path.display(), e))
        })?;

        // Remove comments (simple implementation)
        let content = remove_json_comments(&content);

        serde_json::from_str(&content).map_err(|e| {
            TsError::config(format!("Failed to parse {}: {}", path.display(), e))
        })
    }

    /// Find tsconfig.json in a directory or its parents.
    pub fn find(dir: &Path) -> Option<Utf8PathBuf> {
        let mut current = dir;
        loop {
            let tsconfig = current.join("tsconfig.json");
            if tsconfig.exists() {
                return Utf8PathBuf::from_path_buf(tsconfig).ok();
            }

            let jsconfig = current.join("jsconfig.json");
            if jsconfig.exists() {
                return Utf8PathBuf::from_path_buf(jsconfig).ok();
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => return None,
            }
        }
    }

    /// Resolve the configuration by handling extends.
    pub fn resolve(&mut self, base_dir: &Path) -> TsResult<()> {
        if let Some(extends) = &self.extends.take() {
            let extends_path = base_dir.join(extends);
            let mut base = TsConfig::load(&extends_path)?;
            base.resolve(extends_path.parent().unwrap_or(base_dir))?;

            // Merge base into self
            self.merge_from(&base);
        }
        Ok(())
    }

    /// Merge another config into this one.
    fn merge_from(&mut self, other: &TsConfig) {
        // Merge compiler options
        if self.compiler_options.target.is_none() {
            self.compiler_options.target = other.compiler_options.target.clone();
        }
        if self.compiler_options.module.is_none() {
            self.compiler_options.module = other.compiler_options.module.clone();
        }
        if self.compiler_options.module_resolution.is_none() {
            self.compiler_options.module_resolution = other.compiler_options.module_resolution.clone();
        }
        if self.compiler_options.strict.is_none() {
            self.compiler_options.strict = other.compiler_options.strict;
        }
        // Merge paths
        if self.compiler_options.paths.is_empty() {
            self.compiler_options.paths = other.compiler_options.paths.clone();
        }
        // Merge includes
        if self.include.is_empty() {
            self.include = other.include.clone();
        }
        // Merge excludes
        if self.exclude.is_empty() {
            self.exclude = other.exclude.clone();
        }
    }
}

/// TypeScript compiler options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    /// Target ECMAScript version.
    pub target: Option<String>,
    /// Module system.
    pub module: Option<String>,
    /// Module resolution strategy.
    pub module_resolution: Option<String>,
    /// Strict mode.
    pub strict: Option<bool>,
    /// No emit output.
    pub no_emit: Option<bool>,
    /// Skip library checking.
    pub skip_lib_check: Option<bool>,
    /// ES module interop.
    pub es_module_interop: Option<bool>,
    /// Allow synthetic default imports.
    pub allow_synthetic_default_imports: Option<bool>,
    /// Allow JS files.
    pub allow_js: Option<bool>,
    /// Check JS files.
    pub check_js: Option<bool>,
    /// JSX mode.
    pub jsx: Option<String>,
    /// Base URL.
    pub base_url: Option<String>,
    /// Path aliases.
    #[serde(default)]
    pub paths: HashMap<String, Vec<String>>,
    /// Root directory.
    pub root_dir: Option<String>,
    /// Output directory.
    pub out_dir: Option<String>,
    /// Types to include.
    #[serde(default)]
    pub types: Vec<String>,
    /// Type roots.
    #[serde(default)]
    pub type_roots: Vec<String>,
    /// Declaration files.
    pub declaration: Option<bool>,
    /// Source maps.
    pub source_map: Option<bool>,
    /// Isolated modules.
    pub isolated_modules: Option<bool>,
    /// Verbatim module syntax.
    pub verbatim_module_syntax: Option<bool>,
}

impl CompilerOptions {
    /// Check if using NodeNext module resolution.
    pub fn is_node_next(&self) -> bool {
        self.module_resolution
            .as_ref()
            .map(|m| m.eq_ignore_ascii_case("nodenext") || m.eq_ignore_ascii_case("node16"))
            .unwrap_or(false)
    }

    /// Check if strict mode is enabled.
    pub fn is_strict(&self) -> bool {
        self.strict.unwrap_or(false)
    }
}

/// Vue compiler options in tsconfig.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VueCompilerOptions {
    /// Target Vue version.
    pub target: Option<f32>,
    /// Strict templates.
    pub strict_templates: Option<bool>,
    /// Check unknown components.
    pub check_unknown_components: Option<bool>,
    /// Check unknown directives.
    pub check_unknown_directives: Option<bool>,
    /// Check unknown props.
    pub check_unknown_props: Option<bool>,
    /// Check unknown events.
    pub check_unknown_events: Option<bool>,
    /// Extensions to treat as Vue files.
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Native tags to not treat as components.
    #[serde(default)]
    pub native_tags: Vec<String>,
}

impl VueCompilerOptions {
    /// Get the target Vue version.
    pub fn target_version(&self) -> f32 {
        self.target.unwrap_or(3.5)
    }

    /// Get file extensions to process.
    pub fn file_extensions(&self) -> Vec<&str> {
        if self.extensions.is_empty() {
            vec![".vue"]
        } else {
            self.extensions.iter().map(|s| s.as_str()).collect()
        }
    }
}

/// Remove JSON comments (// and /* */).
fn remove_json_comments(json: &str) -> String {
    let mut result = String::with_capacity(json.len());
    let mut chars = json.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if in_string {
            result.push(c);
            continue;
        }

        // Handle comments
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // Single-line comment
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                }
                Some('*') => {
                    // Multi-line comment
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_json_comments() {
        let input = r#"{
            // This is a comment
            "key": "value", /* inline comment */
            "key2": "value with // in string"
        }"#;
        let result = remove_json_comments(input);
        assert!(!result.contains("// This"));
        assert!(!result.contains("/* inline"));
        assert!(result.contains("// in string")); // In string, should be preserved
    }

    #[test]
    fn test_compiler_options() {
        let opts = CompilerOptions {
            module_resolution: Some("NodeNext".to_string()),
            strict: Some(true),
            ..Default::default()
        };
        assert!(opts.is_node_next());
        assert!(opts.is_strict());
    }
}
