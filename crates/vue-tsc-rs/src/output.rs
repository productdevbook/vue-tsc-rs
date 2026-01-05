//! Output formatting for diagnostics.

use crate::cli::OutputFormat;
use crate::orchestrator::CheckResult;
use std::path::Path;
use ts_runner::TsDiagnostic;
use vue_diagnostics::{Diagnostic, Severity};

// ANSI colors
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const GRAY: &str = "\x1b[90m";
const GREEN: &str = "\x1b[32m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

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
    pub fn print_vue_diagnostic(&self, file: &Path, diagnostic: &Diagnostic, source: Option<&str>) {
        match self.format {
            OutputFormat::Human | OutputFormat::HumanVerbose => {
                self.print_vue_human(file, diagnostic, source);
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
    pub fn print_ts_diagnostic(&self, diagnostic: &TsDiagnostic, source: Option<&str>) {
        match self.format {
            OutputFormat::Human | OutputFormat::HumanVerbose => {
                self.print_ts_human(diagnostic, source);
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
            OutputFormat::Machine => {}
        }
    }

    // Human format - modern style like tsc/vite

    fn print_vue_human(&self, file: &Path, diagnostic: &Diagnostic, source: Option<&str>) {
        let (icon, color, label) = match diagnostic.severity {
            Severity::Error => ("✖", RED, "error"),
            Severity::Warning => ("⚠", YELLOW, "warning"),
            Severity::Hint => ("ℹ", CYAN, "hint"),
        };

        // File location
        let line = 1; // TODO: calculate from span
        let col = diagnostic.span.start;
        println!(
            "\n{BOLD}{}{RESET}:{GRAY}{}:{}{RESET}",
            file.display(),
            line,
            col
        );

        // Show source line if available
        if let Some(src) = source {
            if let Some(line_content) = src.lines().nth(0) {
                let trimmed = line_content.trim_start();
                let indent = line_content.len() - trimmed.len();
                println!("  {GRAY}│{RESET}");
                println!("  {GRAY}│{RESET} {}", trimmed);

                // Underline
                let underline_start = (diagnostic.span.start as usize).saturating_sub(indent);
                let underline_len = (diagnostic.span.end - diagnostic.span.start) as usize;
                let underline_len = underline_len
                    .max(1)
                    .min(trimmed.len().saturating_sub(underline_start));

                if underline_len > 0 && underline_start < trimmed.len() {
                    println!(
                        "  {GRAY}│{RESET} {}{color}{}{RESET}",
                        " ".repeat(underline_start),
                        "~".repeat(underline_len)
                    );
                }
            }
        }

        // Error message
        println!(
            "  {GRAY}╰─{RESET} {color}{icon} {label}{RESET}: {} {GRAY}[{}]{RESET}",
            diagnostic.message,
            diagnostic.code.as_str()
        );
    }

    fn print_ts_human(&self, diagnostic: &TsDiagnostic, source: Option<&str>) {
        let (icon, color, label) = match diagnostic.severity {
            ts_runner::TsSeverity::Error => ("✖", RED, "error"),
            ts_runner::TsSeverity::Warning => ("⚠", YELLOW, "warning"),
            _ => ("ℹ", CYAN, "info"),
        };

        // File location
        if let Some(file) = &diagnostic.file {
            let line = diagnostic.line.unwrap_or(1);
            let col = diagnostic.column.unwrap_or(1);
            println!(
                "\n{BOLD}{}{RESET}:{GRAY}{}:{}{RESET}",
                file.display(),
                line,
                col
            );

            // Show source line if available
            if let Some(src) = source {
                if let Some(line_content) = src.lines().nth((line as usize).saturating_sub(1)) {
                    let trimmed = line_content.trim_start();
                    println!("  {GRAY}│{RESET}");
                    println!("  {GRAY}│{RESET} {}", trimmed);

                    // Simple underline at column position
                    let col_pos = (col as usize).saturating_sub(1);
                    if col_pos < trimmed.len() {
                        println!(
                            "  {GRAY}│{RESET} {}{color}^{RESET}",
                            " ".repeat(col_pos.min(trimmed.len()))
                        );
                    }
                }
            }
        }

        // Error message
        println!(
            "  {GRAY}╰─{RESET} {color}{icon} {label}{RESET}: {} {GRAY}[TS{}]{RESET}",
            diagnostic.message, diagnostic.code
        );
    }

    fn print_summary_human(&self, result: &CheckResult) {
        println!();
        println!("{GRAY}───────────────────────────────────────────{RESET}");

        if result.error_count == 0 && result.warning_count == 0 {
            println!(
                "{GREEN}{BOLD}✓{RESET} {GREEN}No issues found{RESET} {GRAY}({} files in {}ms){RESET}",
                result.file_count,
                result.duration_ms
            );
        } else {
            let mut parts = Vec::new();

            if result.error_count > 0 {
                parts.push(format!(
                    "{RED}{BOLD}{}{RESET} {RED}error{}{RESET}",
                    result.error_count,
                    if result.error_count == 1 { "" } else { "s" }
                ));
            }

            if result.warning_count > 0 {
                parts.push(format!(
                    "{YELLOW}{BOLD}{}{RESET} {YELLOW}warning{}{RESET}",
                    result.warning_count,
                    if result.warning_count == 1 { "" } else { "s" }
                ));
            }

            println!(
                "{} {GRAY}({} files in {}ms){RESET}",
                parts.join(", "),
                result.file_count,
                result.duration_ms
            );
        }
        println!();
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
            .unwrap_or_else(|| "-".to_string());
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
