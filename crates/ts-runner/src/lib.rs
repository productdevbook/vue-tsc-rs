//! TypeScript runner for vue-tsc-rs.
//!
//! This crate provides TypeScript type checking integration by:
//! - Generating virtual TypeScript files from Vue SFCs
//! - Running the TypeScript compiler (tsc) or tsgo
//! - Parsing and remapping diagnostics back to Vue files

pub mod config;
pub mod diagnostics;
pub mod runner;
pub mod virtual_files;

pub use config::TsConfig;
pub use diagnostics::{TsDiagnostic, TsDiagnostics, TsSeverity};
pub use runner::{TsRunner, TsRunnerOptions};
pub use virtual_files::VirtualFileSystem;

use std::path::Path;

/// Result type for TypeScript operations.
pub type TsResult<T> = Result<T, TsError>;

/// An error from TypeScript operations.
#[derive(Debug, Clone)]
pub struct TsError {
    /// The error message.
    pub message: String,
    /// The error kind.
    pub kind: TsErrorKind,
}

impl TsError {
    /// Create a new TypeScript error.
    pub fn new(message: impl Into<String>, kind: TsErrorKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }

    /// Create a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(message, TsErrorKind::Config)
    }

    /// Create a process error.
    pub fn process(message: impl Into<String>) -> Self {
        Self::new(message, TsErrorKind::Process)
    }

    /// Create a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(message, TsErrorKind::Parse)
    }
}

impl std::fmt::Display for TsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TsError {}

/// Kind of TypeScript error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsErrorKind {
    /// Configuration error (tsconfig.json).
    Config,
    /// Process execution error.
    Process,
    /// Parse error.
    Parse,
    /// File not found.
    NotFound,
}

/// Run TypeScript type checking on a workspace.
pub async fn check_workspace(
    workspace: &Path,
    options: &TsRunnerOptions,
) -> TsResult<TsDiagnostics> {
    let runner = TsRunner::new(workspace, options.clone())?;
    runner.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ts_error() {
        let err = TsError::config("Invalid tsconfig.json");
        assert_eq!(err.kind, TsErrorKind::Config);
    }
}
