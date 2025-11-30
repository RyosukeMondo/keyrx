//! KeyRx CLI entry point.

use clap::{Parser, Subcommand};
use keyrx_core::cli::{
    commands::{
        BenchCommand, CheckCommand, DoctorCommand, RunCommand, SimulateCommand, StateCommand,
    },
    OutputFormat,
};
use keyrx_core::KeyRxError;
use std::path::PathBuf;
use std::process::ExitCode;

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

        /// Use mock input instead of real keyboard driver
        #[arg(short, long)]
        mock: bool,
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

        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,
    },

    /// Simulate key events without real keyboard
    Simulate {
        /// Comma-separated list of keys to simulate (e.g., "A,B,CapsLock")
        #[arg(short, long)]
        input: String,

        /// Path to the script file
        #[arg(short, long)]
        script: Option<PathBuf>,
    },
}

fn parse_format(s: &str) -> OutputFormat {
    match s.to_lowercase().as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Human,
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = parse_format(&cli.format);

    let result = run_command(cli.command, format).await;

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let exit_code = determine_exit_code(&err);
            eprintln!("Error: {err:#}");
            exit_code
        }
    }
}

async fn run_command(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Check { script } => {
            CheckCommand::new(script, format).run()?;
        }
        Commands::Run {
            script,
            debug,
            mock,
        } => {
            RunCommand::new(script, debug, mock, format).run().await?;
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
        Commands::Bench { iterations, script } => {
            BenchCommand::new(iterations, script, format).run().await?;
        }
        Commands::Simulate { input, script } => {
            SimulateCommand::new(input, script, format).run().await?;
        }
    }
    Ok(())
}

/// Determine the exit code based on the error type.
///
/// - Exit code 1: General runtime errors
/// - Exit code 2: Validation/compilation errors (script syntax issues)
fn determine_exit_code(err: &anyhow::Error) -> ExitCode {
    // Check if the root cause is a KeyRxError
    if let Some(keyrx_err) = err.downcast_ref::<KeyRxError>() {
        return match keyrx_err {
            KeyRxError::ScriptCompileError { .. } => ExitCode::from(2),
            _ => ExitCode::from(1),
        };
    }

    // Walk the error chain for wrapped errors
    for cause in err.chain() {
        if let Some(keyrx_err) = cause.downcast_ref::<KeyRxError>() {
            return match keyrx_err {
                KeyRxError::ScriptCompileError { .. } => ExitCode::from(2),
                _ => ExitCode::from(1),
            };
        }
    }

    ExitCode::from(1)
}
