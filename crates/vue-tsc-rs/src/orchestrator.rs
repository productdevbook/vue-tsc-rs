//! Orchestrator for running type checking.

use crate::cli::Args;
use crate::config::Config;
use crate::output::OutputFormatter;
use miette::{IntoDiagnostic, Result, WrapErr};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ts_runner::{TsDiagnostics, TsRunner, TsRunnerOptions};
use vue_diagnostics::{diagnose_sfc, Diagnostic, Severity};

/// Result of a check run.
#[derive(Debug, Default)]
pub struct CheckResult {
    /// Number of files checked.
    pub file_count: usize,
    /// Number of errors.
    pub error_count: usize,
    /// Number of warnings.
    pub warning_count: usize,
    /// Time taken.
    pub duration_ms: u64,
}

/// Orchestrator for running vue-tsc-rs.
pub struct Orchestrator {
    /// Configuration.
    config: Config,
    /// CLI arguments.
    args: Args,
    /// Output formatter.
    formatter: OutputFormatter,
}

impl Orchestrator {
    /// Create a new orchestrator.
    pub fn new(workspace: PathBuf, args: Args) -> Result<Self> {
        let config = Config::load(&workspace, &args)?;
        let formatter = OutputFormatter::new(args.output);

        Ok(Self {
            config,
            args,
            formatter,
        })
    }

    /// Run a single check.
    pub async fn run_single_check(&mut self) -> Result<CheckResult> {
        let start = Instant::now();

        // Find Vue files
        let vue_files = self.find_vue_files()?;

        if self.args.verbose {
            eprintln!("Found {} Vue files", vue_files.len());
        }

        // Run Vue diagnostics in parallel
        let vue_diagnostics = self.run_vue_diagnostics(&vue_files)?;

        // Run TypeScript type checking
        let ts_diagnostics = if !self.args.skip_typecheck {
            self.run_ts_check().await?
        } else {
            TsDiagnostics::default()
        };

        // Combine and output results
        let result = self.output_results(&vue_files, &vue_diagnostics, &ts_diagnostics);

        let duration = start.elapsed();
        let check_result = CheckResult {
            file_count: vue_files.len(),
            error_count: result.0,
            warning_count: result.1,
            duration_ms: duration.as_millis() as u64,
        };

        // Show timing if requested
        if self.args.timings {
            eprintln!("\nTiming: {}ms", check_result.duration_ms);
        }

        // Show summary
        self.formatter.print_summary(&check_result);

        Ok(check_result)
    }

    /// Run in watch mode.
    pub async fn run_watch_mode(&mut self) -> Result<()> {
        use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc::channel;
        use std::time::Duration;

        eprintln!("Starting watch mode...\n");

        // Initial check
        let _ = self.run_single_check().await;

        // Set up file watcher
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            NotifyConfig::default().with_poll_interval(Duration::from_millis(500)),
        )
        .into_diagnostic()?;

        watcher
            .watch(&self.config.workspace, RecursiveMode::Recursive)
            .into_diagnostic()?;

        // Watch loop
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    // Check if the changed file is relevant
                    let should_recheck = event.paths.iter().any(|p| {
                        p.extension()
                            .map(|e| e == "vue" || e == "ts" || e == "tsx")
                            .unwrap_or(false)
                    });

                    if should_recheck {
                        if !self.args.preserve_watch_output {
                            // Clear screen
                            print!("\x1B[2J\x1B[1;1H");
                        }

                        eprintln!("File change detected. Rerunning...\n");
                        let _ = self.run_single_check().await;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Continue waiting
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Find all Vue files in the workspace.
    fn find_vue_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(&self.config.workspace)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check if should process
            if !self.should_process_path(path) {
                continue;
            }

            files.push(path.to_path_buf());
        }

        Ok(files)
    }

    /// Check if a path should be processed.
    fn should_process_path(&self, path: &Path) -> bool {
        // Check extension
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        if ext != "vue" {
            return false;
        }

        // Skip node_modules and hidden directories
        let path_str = path.to_string_lossy();
        if path_str.contains("node_modules") || path_str.contains("/.") {
            return false;
        }

        // Check ignore patterns
        for pattern in &self.config.ignore_patterns {
            if path_str.contains(pattern.trim_matches('*')) {
                return false;
            }
        }

        true
    }

    /// Run Vue-specific diagnostics on files.
    #[allow(clippy::type_complexity)]
    fn run_vue_diagnostics(
        &self,
        files: &[PathBuf],
    ) -> Result<Vec<(PathBuf, String, Vec<Diagnostic>)>> {
        let results: Arc<Mutex<Vec<(PathBuf, String, Vec<Diagnostic>)>>> =
            Arc::new(Mutex::new(Vec::new()));

        files
            .par_iter()
            .for_each(|file| match self.check_vue_file(file) {
                Ok((source, diagnostics)) => {
                    if !diagnostics.is_empty() {
                        let mut results = results.lock().unwrap();
                        results.push((file.clone(), source, diagnostics));
                    }
                }
                Err(e) => {
                    eprintln!("Error checking {}: {}", file.display(), e);
                }
            });

        Ok(Arc::try_unwrap(results)
            .unwrap_or_else(|_| panic!("Arc still has multiple references"))
            .into_inner()
            .unwrap())
    }

    /// Check a single Vue file.
    fn check_vue_file(&self, path: &Path) -> Result<(String, Vec<Diagnostic>)> {
        let content = std::fs::read_to_string(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to read {}", path.display()))?;

        let sfc = vue_parser::parse(&content)
            .map_err(|e| miette::miette!("Parse error in {}: {}", path.display(), e))?;

        let diagnostics = diagnose_sfc(&sfc, &self.config.diagnostic_options);

        Ok((content, diagnostics))
    }

    /// Run TypeScript type checking.
    async fn run_ts_check(&self) -> Result<TsDiagnostics> {
        let options = TsRunnerOptions {
            tsconfig: self.config.tsconfig_path.clone(),
            use_tsgo: self.args.use_tsgo,
            emit: self.args.emit_ts,
            generate_virtual: true,
            temp_dir: None,
            tsc_args: Vec::new(),
        };

        let runner = TsRunner::new(&self.config.workspace, options)
            .map_err(|e| miette::miette!("Failed to create TypeScript runner: {}", e))?;

        runner
            .run()
            .await
            .map_err(|e| miette::miette!("TypeScript check failed: {}", e))
    }

    /// Output results and return error/warning counts.
    fn output_results(
        &self,
        _files: &[PathBuf],
        vue_diagnostics: &[(PathBuf, String, Vec<Diagnostic>)],
        ts_diagnostics: &TsDiagnostics,
    ) -> (usize, usize) {
        let mut error_count = 0;
        let mut warning_count = 0;

        // Output Vue diagnostics
        for (file, source, diagnostics) in vue_diagnostics {
            for diag in diagnostics {
                self.formatter
                    .print_vue_diagnostic(file, diag, Some(source));
                match diag.severity {
                    Severity::Error => error_count += 1,
                    Severity::Warning => warning_count += 1,
                    Severity::Hint => {}
                }
            }
        }

        // Output TypeScript diagnostics
        for diag in &ts_diagnostics.diagnostics {
            // Try to read source for context
            let source = diag
                .file
                .as_ref()
                .and_then(|f| std::fs::read_to_string(f).ok());
            self.formatter.print_ts_diagnostic(diag, source.as_deref());
        }
        error_count += ts_diagnostics.error_count;
        warning_count += ts_diagnostics.warning_count;

        (error_count, warning_count)
    }
}
