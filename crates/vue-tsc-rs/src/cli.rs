//! Command-line argument parsing.

use clap::Parser;
use std::path::PathBuf;

/// Vue type checker - A high-performance alternative to vue-tsc
#[derive(Parser, Debug, Clone)]
#[command(name = "vue-tsc-rs")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Workspace directory to check
    #[arg(short, long)]
    pub workspace: Option<PathBuf>,

    /// Path to tsconfig.json
    #[arg(short = 'p', long)]
    pub project: Option<PathBuf>,

    /// Run in watch mode
    #[arg(short, long)]
    pub watch: bool,

    /// Output format
    #[arg(long, default_value = "human")]
    pub output: OutputFormat,

    /// Fail on warnings
    #[arg(long)]
    pub fail_on_warning: bool,

    /// Emit generated TypeScript files (for debugging)
    #[arg(long)]
    pub emit_ts: bool,

    /// Show timing information
    #[arg(long)]
    pub timings: bool,

    /// Maximum number of errors to show
    #[arg(long)]
    pub max_errors: Option<usize>,

    /// Skip type checking (only run Vue diagnostics)
    #[arg(long)]
    pub skip_typecheck: bool,

    /// Ignore patterns (glob)
    #[arg(long)]
    pub ignore: Vec<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Use tsgo instead of tsc
    #[arg(long)]
    pub use_tsgo: bool,

    /// Preserve watch output (don't clear screen)
    #[arg(long)]
    pub preserve_watch_output: bool,
}

/// Output format for diagnostics.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output
    #[default]
    Human,
    /// Human-readable with more details
    HumanVerbose,
    /// JSON output
    Json,
    /// Machine-readable output
    Machine,
}

impl Args {
    /// Get the tsconfig path.
    pub fn tsconfig(&self) -> Option<&PathBuf> {
        self.project.as_ref()
    }

    /// Check if output should be verbose.
    pub fn is_verbose(&self) -> bool {
        self.verbose || matches!(self.output, OutputFormat::HumanVerbose)
    }
}
