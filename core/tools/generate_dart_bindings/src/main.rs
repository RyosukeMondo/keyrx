//! Dart Binding Code Generator
//!
//! This tool generates type-safe Dart FFI bindings from JSON contracts,
//! eliminating manual synchronization between Rust exports and Dart imports.
//!
//! Usage:
//!   cargo run --bin generate-dart-bindings
//!   cargo run --bin generate-dart-bindings --domain config
//!   cargo run --bin generate-dart-bindings --check
//!   cargo run --bin generate-dart-bindings --verbose

use anyhow::Result;
use clap::Parser;

mod cli;
mod loader;

use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("Dart Binding Code Generator");
        eprintln!("Contracts directory: {:?}", cli.contracts_dir());
        eprintln!("Output directory: {:?}", cli.output_dir());
        if let Some(domain) = &cli.domain {
            eprintln!("Domain filter: {}", domain);
        }
        if cli.check {
            eprintln!("Check mode: enabled (will not write files)");
        }
    }

    // TODO: Implement generation pipeline in subsequent tasks
    // 1. Load contracts from directory
    // 2. Map types to Dart FFI types
    // 3. Generate bindings code
    // 4. Generate model classes
    // 5. Write files (unless --check mode)
    // 6. Format with dart format

    eprintln!("Dart bindings generator initialized successfully");

    Ok(())
}
