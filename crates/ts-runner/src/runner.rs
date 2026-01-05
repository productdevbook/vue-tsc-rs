//! TypeScript compiler runner.

use crate::config::TsConfig;
use crate::diagnostics::{parse_ts_output, DiagnosticRemapper, TsDiagnostics};
use crate::virtual_files::VirtualFileSystem;
use crate::{TsError, TsResult};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Options for the TypeScript runner.
#[derive(Debug, Clone, Default)]
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
    /// Temp directory for virtual files.
    pub temp_dir: Option<PathBuf>,
}

/// TypeScript compiler runner.
pub struct TsRunner {
    /// Workspace root.
    workspace: PathBuf,
    /// Options.
    options: TsRunnerOptions,
    /// TypeScript configuration.
    tsconfig: Option<TsConfig>,
    /// Virtual file system.
    vfs: VirtualFileSystem,
    /// Diagnostic remapper.
    remapper: DiagnosticRemapper,
}

impl TsRunner {
    /// Create a new runner.
    pub fn new(workspace: &Path, options: TsRunnerOptions) -> TsResult<Self> {
        // Load tsconfig if specified or find it
        let tsconfig = if let Some(path) = &options.tsconfig {
            Some(TsConfig::load(path)?)
        } else if let Some(path) = TsConfig::find(workspace) {
            Some(TsConfig::load(path.as_std_path())?)
        } else {
            None
        };

        let temp_dir = options.temp_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir().join("vue-tsc-rs")
        });

        Ok(Self {
            workspace: workspace.to_path_buf(),
            options,
            tsconfig,
            vfs: VirtualFileSystem::new(temp_dir),
            remapper: DiagnosticRemapper::new(),
        })
    }

    /// Run type checking.
    pub async fn run(&self) -> TsResult<TsDiagnostics> {
        // Generate virtual files for Vue components
        if self.options.generate_virtual {
            self.generate_virtual_files()?;
        }

        // Run the TypeScript compiler
        let output = if self.options.use_tsgo {
            self.run_tsgo().await?
        } else {
            self.run_tsc().await?
        };

        // Parse diagnostics
        let mut diagnostics = TsDiagnostics::new();
        for diag in parse_ts_output(&output) {
            diagnostics.add(diag);
        }

        // Remap diagnostics from virtual files to original files
        self.remapper.remap_all(&mut diagnostics);

        // Sort diagnostics
        diagnostics.sort();

        Ok(diagnostics)
    }

    /// Generate virtual TypeScript files for Vue components.
    fn generate_virtual_files(&self) -> TsResult<()> {
        // Find all Vue files
        let vue_files = self.find_vue_files()?;

        for file in vue_files {
            // Read and parse the Vue file
            let content = std::fs::read_to_string(&file).map_err(|e| {
                TsError::process(format!("Failed to read {}: {}", file.display(), e))
            })?;

            // Parse the SFC
            let sfc = vue_parser::parse(&content).map_err(|e| {
                TsError::parse(format!("Failed to parse {}: {}", file.display(), e))
            })?;

            // Generate TypeScript code
            let result = vue_codegen::generate(&sfc, &vue_codegen::CodegenOptions::default());

            // Write virtual file
            let virtual_path = self.vfs.virtual_path(&file, result.lang.extension());
            self.vfs.write(&virtual_path, &result.code)?;

            // Register for remapping
            // self.remapper.register(virtual_path, file, result.source_map, &content);
        }

        Ok(())
    }

    /// Find all Vue files in the workspace.
    fn find_vue_files(&self) -> TsResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        let extensions = self
            .tsconfig
            .as_ref()
            .map(|c| c.vue_compiler_options.file_extensions())
            .unwrap_or_else(|| vec![".vue"]);

        for entry in walkdir::WalkDir::new(&self.workspace)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip node_modules and hidden directories
            if path
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            {
                continue;
            }
            if path.components().any(|c| c.as_os_str() == "node_modules") {
                continue;
            }

            // Check extension
            if let Some(ext) = path.extension() {
                let ext_str = format!(".{}", ext.to_string_lossy());
                if extensions.iter().any(|e| e == &ext_str) {
                    files.push(path.to_path_buf());
                }
            }
        }

        Ok(files)
    }

    /// Run the TypeScript compiler (tsc).
    async fn run_tsc(&self) -> TsResult<String> {
        let tsc = self.find_tsc()?;

        let mut cmd = Command::new(&tsc);
        cmd.current_dir(&self.workspace);

        // Add noEmit if not emitting
        if !self.options.emit {
            cmd.arg("--noEmit");
        }

        // Add tsconfig if specified
        if let Some(tsconfig) = &self.options.tsconfig {
            cmd.arg("--project").arg(tsconfig);
        }

        // Add custom arguments
        for arg in &self.options.tsc_args {
            cmd.arg(arg);
        }

        // Capture output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            TsError::process(format!("Failed to run tsc: {}", e))
        })?;

        // Combine stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        Ok(format!("{}{}", stdout, stderr))
    }

    /// Run tsgo (Go-based TypeScript compiler).
    async fn run_tsgo(&self) -> TsResult<String> {
        let tsgo = self.find_tsgo()?;

        let mut cmd = Command::new(&tsgo);
        cmd.current_dir(&self.workspace);

        // Add virtual files directory
        cmd.arg("--virtualDir").arg(self.vfs.root());

        // Add tsconfig if specified
        if let Some(tsconfig) = &self.options.tsconfig {
            cmd.arg("--project").arg(tsconfig);
        }

        // Capture output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            TsError::process(format!("Failed to run tsgo: {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        Ok(format!("{}{}", stdout, stderr))
    }

    /// Find the tsc executable.
    fn find_tsc(&self) -> TsResult<PathBuf> {
        // Try local node_modules first
        let local = self.workspace.join("node_modules/.bin/tsc");
        if local.exists() {
            return Ok(local);
        }

        // Try npx
        if which::which("npx").is_ok() {
            return Ok(PathBuf::from("npx"));
        }

        // Try global tsc
        which::which("tsc").map_err(|_| {
            TsError::process("TypeScript compiler (tsc) not found. Install with: npm install -g typescript")
        })
    }

    /// Find the tsgo executable.
    fn find_tsgo(&self) -> TsResult<PathBuf> {
        // Try local node_modules first
        let local = self.workspace.join("node_modules/.bin/tsgo");
        if local.exists() {
            return Ok(local);
        }

        // Try global tsgo
        which::which("tsgo").map_err(|_| {
            TsError::process("tsgo not found. Install with: npm install -g @anthropic/tsgo")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_options() {
        let opts = TsRunnerOptions {
            use_tsgo: true,
            emit: false,
            ..Default::default()
        };
        assert!(opts.use_tsgo);
        assert!(!opts.emit);
    }
}
