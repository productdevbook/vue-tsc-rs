//! vue-tsc-rs - High-performance Vue type checker.

use clap::Parser;
use miette::Result;
use std::path::PathBuf;
use std::process::ExitCode;

mod cli;
mod config;
mod orchestrator;
mod output;

use cli::Args;
use orchestrator::Orchestrator;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    // Set up miette for nice error output
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .context_lines(2)
                .build(),
        )
    }))
    .ok();

    match run(args).await {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> Result<ExitCode> {
    // Determine workspace
    let workspace = args
        .workspace
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Capture watch flag before moving args
    let watch = args.watch;

    // Create orchestrator
    let mut orchestrator = Orchestrator::new(workspace, args)?;

    // Run type checking
    if watch {
        orchestrator.run_watch_mode().await?;
        Ok(ExitCode::SUCCESS)
    } else {
        let result = orchestrator.run_single_check().await?;

        if result.error_count > 0 {
            Ok(ExitCode::from(1))
        } else {
            Ok(ExitCode::SUCCESS)
        }
    }
}
