//! KeyRx CLI entry point.

use anyhow::Result;
use clap::{Parser, Subcommand};
use keyrx_core::cli::{
    commands::{CheckCommand, DoctorCommand, RunCommand, StateCommand},
    OutputFormat,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "keyrx")]
#[command(about = "KeyRx - The Ultimate Input Remapping Engine")]
#[command(version)]
struct Cli {
    /// Output format (human or json)
    #[arg(long, default_value = "human")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate and lint a Rhai script
    Check {
        /// Path to the script file
        script: PathBuf,
    },

    /// Run the engine in headless mode
    Run {
        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,
    },

    /// Inspect current engine state
    State {
        /// Show layer information
        #[arg(long)]
        layers: bool,

        /// Show modifier information
        #[arg(long)]
        modifiers: bool,
    },

    /// Run self-diagnostics
    Doctor {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start interactive REPL
    Repl,

    /// Run latency benchmark
    Bench {
        /// Number of iterations
        #[arg(long, default_value = "10000")]
        iterations: usize,
    },
}

fn parse_format(s: &str) -> OutputFormat {
    match s.to_lowercase().as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Human,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let format = parse_format(&cli.format);

    match cli.command {
        Commands::Check { script } => {
            CheckCommand::new(script, format).run()?;
        }
        Commands::Run { script, debug } => {
            RunCommand::new(script, debug, format).run().await?;
        }
        Commands::State { layers, modifiers } => {
            StateCommand::new(layers, modifiers, format).run()?;
        }
        Commands::Doctor { verbose } => {
            DoctorCommand::new(verbose, format).run()?;
        }
        Commands::Repl => {
            println!("REPL not yet implemented");
        }
        Commands::Bench { iterations } => {
            println!("Benchmark with {} iterations not yet implemented", iterations);
        }
    }

    Ok(())
}
