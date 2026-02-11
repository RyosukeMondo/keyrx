//! keyrx_compiler - Rhai-to-binary configuration compiler
//!
//! This binary compiles Rhai DSL configuration scripts into static .krx binary files.

use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process;

mod cli;
mod error;
mod import_resolver;
mod parser;
mod serialize;

#[derive(Parser)]
#[command(name = "keyrx_compiler")]
#[command(
    version,
    about = "Compile Rhai configuration scripts to .krx binary files"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Rhai script to a .krx binary file
    Compile {
        /// Input Rhai configuration file
        input: PathBuf,

        /// Output .krx binary file (defaults to input file with .krx extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify a .krx binary file
    Verify {
        /// .krx binary file to verify
        file: PathBuf,
    },

    /// Extract and display the SHA256 hash from a .krx file
    Hash {
        /// .krx binary file
        file: PathBuf,

        /// Verify hash matches computed hash of data section
        #[arg(long)]
        verify: bool,
    },

    /// Parse a Rhai script and display the configuration
    Parse {
        /// Input Rhai configuration file
        input: PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate HTML visualization of key mappings
    View {
        /// Input Rhai configuration file
        input: PathBuf,

        /// Output HTML file (defaults to input file with .html extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Open in browser after generation
        #[arg(long)]
        open: bool,
    },
}

fn main() {
    // Check NO_COLOR environment variable to disable colored output
    // This is a standard convention: https://no-color.org/
    let no_color = env::var("NO_COLOR").is_ok();

    // Note: Individual CLI handlers check NO_COLOR when formatting output.
    // This could be extended to use a colored output library like `colored`
    // if needed in the future.
    if no_color {
        // Currently, handlers use basic eprintln!/println! which are uncolored.
        // If we add colored output in the future, we would disable it here.
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Compile { input, output } => {
            // Determine output path (default to input with .krx extension)
            let output_path = output.unwrap_or_else(|| {
                let mut path = input.clone();
                path.set_extension("krx");
                path
            });
            cli::compile::handle_compile(&input, &output_path).map_err(|e| e.to_string())
        }
        Commands::Verify { file } => cli::verify::handle_verify(&file).map_err(|e| e.to_string()),
        Commands::Hash { file, verify } => {
            cli::hash::handle_hash(&file, verify).map_err(|e| e.to_string())
        }
        Commands::Parse { input, json } => {
            cli::parse::handle_parse(&input, json).map_err(|e| e.to_string())
        }
        Commands::View {
            input,
            output,
            open,
        } => {
            let output_path = output.unwrap_or_else(|| {
                let mut path = input.clone();
                path.set_extension("html");
                path
            });
            cli::view::handle_view(&input, &output_path, open).map_err(|e| e.to_string())
        }
    };

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
