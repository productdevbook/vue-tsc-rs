//! Configuration loading and management.

use crate::cli::Args;
use miette::{IntoDiagnostic, Result};
use std::path::{Path, PathBuf};
use ts_runner::TsConfig;
use vue_diagnostics::DiagnosticOptions;

/// Configuration for vue-tsc-rs.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    /// Workspace root directory.
    pub workspace: PathBuf,
    /// Path to tsconfig.json.
    pub tsconfig_path: Option<PathBuf>,
    /// Parsed TypeScript configuration.
    pub tsconfig: Option<TsConfig>,
    /// Vue diagnostic options.
    pub diagnostic_options: DiagnosticOptions,
    /// File extensions to process.
    pub extensions: Vec<String>,
    /// Ignore patterns.
    pub ignore_patterns: Vec<String>,
}

impl Config {
    /// Load configuration from CLI arguments and workspace.
    pub fn load(workspace: &Path, args: &Args) -> Result<Self> {
        // Find or use specified tsconfig
        let tsconfig_path = args
            .tsconfig()
            .cloned()
            .or_else(|| TsConfig::find(workspace).map(|p| p.into_std_path_buf()));

        // Load tsconfig
        let tsconfig = if let Some(ref path) = tsconfig_path {
            Some(TsConfig::load(path).into_diagnostic()?)
        } else {
            None
        };

        // Build diagnostic options
        let diagnostic_options = DiagnosticOptions {
            check_unknown_components: tsconfig
                .as_ref()
                .and_then(|c| c.vue_compiler_options.check_unknown_components)
                .unwrap_or(false),
            check_unknown_directives: tsconfig
                .as_ref()
                .and_then(|c| c.vue_compiler_options.check_unknown_directives)
                .unwrap_or(false),
            check_v_for_keys: true,
            known_components: Vec::new(),
            known_directives: Vec::new(),
        };

        // Get extensions
        let extensions = tsconfig
            .as_ref()
            .map(|c| c.vue_compiler_options.extensions.to_vec())
            .unwrap_or_else(|| vec![".vue".to_string()]);

        // Build ignore patterns
        let mut ignore_patterns = vec![
            "**/node_modules/**".to_string(),
            "**/dist/**".to_string(),
            "**/.git/**".to_string(),
        ];
        ignore_patterns.extend(args.ignore.iter().cloned());

        Ok(Self {
            workspace: workspace.to_path_buf(),
            tsconfig_path,
            tsconfig,
            diagnostic_options,
            extensions,
            ignore_patterns,
        })
    }

    /// Check if a file should be processed.
    #[allow(dead_code)]
    pub fn should_process(&self, path: &Path) -> bool {
        // Check extension
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();

        if !self.extensions.iter().any(|e| e == &ext) {
            return false;
        }

        // Check ignore patterns
        let path_str = path.to_string_lossy();
        for pattern in &self.ignore_patterns {
            if let Ok(glob) = globset::Glob::new(pattern) {
                let matcher = glob.compile_matcher();
                if matcher.is_match(&*path_str) {
                    return false;
                }
            }
        }

        true
    }
}
