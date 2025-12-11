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

#![allow(clippy::print_stdout, clippy::print_stderr)]

mod pipeline;

use anyhow::Result;
use clap::Parser;
use generate_dart_bindings::cli::Cli;
use pipeline::GenerationPipeline;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let pipeline = GenerationPipeline::new(&cli);

    if cli.verbose {
        eprintln!("Dart Binding Code Generator");
        eprintln!("Contracts directory: {:?}", cli.contracts_dir());
        eprintln!("Output directory: {:?}", cli.output_dir());
        if let Some(domain) = &cli.domain {
            eprintln!("Domain filter: {domain}");
        }
        if cli.check {
            eprintln!("Check mode: enabled (will not write files)");
        }
    }

    let result = pipeline.run()?;

    if cli.verbose {
        eprintln!("Generated {} bindings file(s)", result.bindings_generated);
        eprintln!("Generated {} models file(s)", result.models_generated);
        if result.files_skipped > 0 {
            eprintln!("Skipped {} unchanged file(s)", result.files_skipped);
        }
    }

    if cli.check && result.needs_regeneration {
        eprintln!("Error: Generated bindings are out of date. Run without --check to regenerate.");
        std::process::exit(1);
    }

    if !cli.check {
        eprintln!(
            "Successfully generated Dart bindings ({} file(s) written)",
            result.files_written
        );
    } else {
        eprintln!("Bindings are up to date.");
    }

    Ok(())
}
