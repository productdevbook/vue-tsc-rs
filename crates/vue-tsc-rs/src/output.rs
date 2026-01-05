//! Output formatting for diagnostics.

use crate::cli::OutputFormat;
use crate::orchestrator::CheckResult;
use std::path::Path;
use ts_runner::TsDiagnostic;
use vue_diagnostics::{Diagnostic, Severity};

/// Formatter for diagnostic output.
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    /// Create a new formatter.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Print a Vue diagnostic.
    pub fn print_vue_diagnostic(&self, file: &Path, diagnostic: &Diagnostic) {
        match self.format {
            OutputFormat::Human | OutputFormat::HumanVerbose => {
                self.print_vue_human(file, diagnostic);
            }
            OutputFormat::Json => {
                self.print_vue_json(file, diagnostic);
            }
            OutputFormat::Machine => {
                self.print_vue_machine(file, diagnostic);
            }
        }
    }

    /// Print a TypeScript diagnostic.
    pub fn print_ts_diagnostic(&self, diagnostic: &TsDiagnostic) {
        match self.format {
            OutputFormat::Human | OutputFormat::HumanVerbose => {
                self.print_ts_human(diagnostic);
            }
            OutputFormat::Json => {
                self.print_ts_json(diagnostic);
            }
            OutputFormat::Machine => {
                self.print_ts_machine(diagnostic);
            }
        }
    }

    /// Print the summary.
    pub fn print_summary(&self, result: &CheckResult) {
        match self.format {
            OutputFormat::Human | OutputFormat::HumanVerbose => {
                self.print_summary_human(result);
            }
            OutputFormat::Json => {
                self.print_summary_json(result);
            }
            OutputFormat::Machine => {
                // No summary for machine format
            }
        }
    }

    // Human format

    fn print_vue_human(&self, file: &Path, diagnostic: &Diagnostic) {
        let severity_str = match diagnostic.severity {
            Severity::Error => "\x1b[31merror\x1b[0m",
            Severity::Warning => "\x1b[33mwarning\x1b[0m",
            Severity::Hint => "\x1b[34mhint\x1b[0m",
        };

        println!(
            "{}:{}:{}: {}: {}",
            file.display(),
            diagnostic.span.start,
            0,
            severity_str,
            diagnostic.message
        );
    }

    fn print_ts_human(&self, diagnostic: &TsDiagnostic) {
        let severity_str = match diagnostic.severity {
            ts_runner::TsSeverity::Error => "\x1b[31merror\x1b[0m",
            ts_runner::TsSeverity::Warning => "\x1b[33mwarning\x1b[0m",
            _ => "\x1b[34minfo\x1b[0m",
        };

        let location = if let Some(file) = &diagnostic.file {
            let line = diagnostic.line.unwrap_or(1);
            let col = diagnostic.column.unwrap_or(1);
            format!("{}:{}:{}", file.display(), line, col)
        } else {
            "unknown".to_string()
        };

        println!(
            "{}: {} TS{}: {}",
            location, severity_str, diagnostic.code, diagnostic.message
        );
    }

    fn print_summary_human(&self, result: &CheckResult) {
        println!();
        if result.error_count == 0 && result.warning_count == 0 {
            println!(
                "\x1b[32m✓\x1b[0m No issues found in {} files ({}ms)",
                result.file_count, result.duration_ms
            );
        } else {
            if result.error_count > 0 {
                println!(
                    "\x1b[31m✗\x1b[0m Found {} error{} in {} files",
                    result.error_count,
                    if result.error_count == 1 { "" } else { "s" },
                    result.file_count
                );
            }
            if result.warning_count > 0 {
                println!(
                    "\x1b[33m⚠\x1b[0m Found {} warning{}",
                    result.warning_count,
                    if result.warning_count == 1 { "" } else { "s" }
                );
            }
            println!("Time: {}ms", result.duration_ms);
        }
    }

    // JSON format

    fn print_vue_json(&self, file: &Path, diagnostic: &Diagnostic) {
        let json = serde_json::json!({
            "type": "vue",
            "file": file.to_string_lossy(),
            "severity": diagnostic.severity.as_str(),
            "message": diagnostic.message,
            "code": diagnostic.code.as_str(),
            "span": {
                "start": diagnostic.span.start,
                "end": diagnostic.span.end
            }
        });
        println!("{}", json);
    }

    fn print_ts_json(&self, diagnostic: &TsDiagnostic) {
        let json = serde_json::json!({
            "type": "typescript",
            "file": diagnostic.file.as_ref().map(|f| f.to_string_lossy().to_string()),
            "severity": diagnostic.severity.as_str(),
            "message": diagnostic.message,
            "code": diagnostic.code,
            "line": diagnostic.line,
            "column": diagnostic.column
        });
        println!("{}", json);
    }

    fn print_summary_json(&self, result: &CheckResult) {
        let json = serde_json::json!({
            "type": "summary",
            "files": result.file_count,
            "errors": result.error_count,
            "warnings": result.warning_count,
            "duration_ms": result.duration_ms
        });
        println!("{}", json);
    }

    // Machine format

    fn print_vue_machine(&self, file: &Path, diagnostic: &Diagnostic) {
        println!(
            "{}:{}:{}:{}:{}",
            file.display(),
            diagnostic.span.start,
            diagnostic.span.end,
            diagnostic.severity.as_str(),
            diagnostic.message.replace(':', "\\:")
        );
    }

    fn print_ts_machine(&self, diagnostic: &TsDiagnostic) {
        let file = diagnostic
            .file
            .as_ref()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let line = diagnostic.line.unwrap_or(0);
        let col = diagnostic.column.unwrap_or(0);

        println!(
            "{}:{}:{}:{}:TS{}:{}",
            file,
            line,
            col,
            diagnostic.severity.as_str(),
            diagnostic.code,
            diagnostic.message.replace(':', "\\:")
        );
    }
}
