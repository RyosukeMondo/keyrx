//! keyrx_compiler - Rhai-to-binary configuration compiler
//!
//! This binary compiles Rhai DSL configuration scripts into static .krx binary files.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod dfa_gen;
mod error;
mod import_resolver;
mod mphf_gen;
mod parser;
mod serialize;

use error::{DeserializeError, ParseError, SerializeError};

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

        /// Output .krx binary file (default: input.krx)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify a .krx binary file
    Verify {
        /// .krx binary file to verify
        config: PathBuf,
    },

    /// Extract and display the SHA256 hash from a .krx file
    Hash {
        /// .krx binary file
        config: PathBuf,
    },

    /// Parse a Rhai script and output JSON
    Parse {
        /// Input Rhai configuration file
        input: PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Compile { input, output } => compile_command(input, output),
        Commands::Verify { config } => verify_command(config),
        Commands::Hash { config } => hash_command(config),
        Commands::Parse { input, json } => parse_command(input, json),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn compile_command(input: PathBuf, output: Option<PathBuf>) -> Result<(), String> {
    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("krx");
        path
    });

    // Parse the Rhai script
    let mut parser = parser::Parser::new();
    let config = parser.parse_script(&input).map_err(format_parse_error)?;

    // Serialize to binary
    let binary = serialize::serialize(&config).map_err(format_serialize_error)?;

    // Write to output file
    std::fs::write(&output_path, binary)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    println!(
        "Successfully compiled {} to {}",
        input.display(),
        output_path.display()
    );

    Ok(())
}

fn verify_command(config: PathBuf) -> Result<(), String> {
    // Read the .krx file
    let bytes = std::fs::read(&config)
        .map_err(|e| format!("Failed to read file {}: {}", config.display(), e))?;

    // Deserialize and validate
    match serialize::deserialize(&bytes) {
        Ok(_archived_config) => {
            println!("✓ {} is valid", config.display());
            println!("  Magic bytes: OK");
            println!("  Version: OK");
            println!("  Hash: OK");
            println!("  Data: OK");
            Ok(())
        }
        Err(e) => Err(format_deserialize_error(e)),
    }
}

fn hash_command(config: PathBuf) -> Result<(), String> {
    // Read the .krx file
    let bytes = std::fs::read(&config)
        .map_err(|e| format!("Failed to read file {}: {}", config.display(), e))?;

    // Validate minimum size
    if bytes.len() < serialize::HEADER_SIZE {
        return Err(format!(
            "File too small: {} bytes (expected at least {} bytes for header)",
            bytes.len(),
            serialize::HEADER_SIZE
        ));
    }

    // Extract hash from bytes 8-40 (after magic and version)
    let hash_bytes = &bytes[8..40];

    // Format as hex string
    let hash_hex: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    println!("{}", hash_hex);

    Ok(())
}

fn parse_command(input: PathBuf, json: bool) -> Result<(), String> {
    // Parse the Rhai script
    let mut parser = parser::Parser::new();
    let config = parser.parse_script(&input).map_err(format_parse_error)?;

    if json {
        // Output as JSON
        let json_str =
            serde_json::to_string_pretty(&config).map_err(|e| format!("JSON error: {}", e))?;
        println!("{}", json_str);
    } else {
        // Output human-readable format
        println!("Configuration parsed successfully:");
        println!("  Version: {}", config.version);
        println!("  Devices: {}", config.devices.len());
        for (i, device) in config.devices.iter().enumerate() {
            println!(
                "    Device {}: {} ({} mappings)",
                i + 1,
                device.identifier.pattern,
                device.mappings.len()
            );
        }
        println!("  Metadata:");
        println!("    Compiler version: {}", config.metadata.compiler_version);
        println!(
            "    Compilation timestamp: {}",
            config.metadata.compilation_timestamp
        );
        println!("    Source hash: {}", config.metadata.source_hash);
    }

    Ok(())
}

fn format_parse_error(e: ParseError) -> String {
    format!("Parse error: {}", e)
}

fn format_serialize_error(e: SerializeError) -> String {
    format!("Serialization error: {}", e)
}

fn format_deserialize_error(e: DeserializeError) -> String {
    format!("Deserialization error: {}", e)
}
