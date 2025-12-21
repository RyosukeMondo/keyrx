//! CLI argument parsing for the Dart binding generator

use clap::Parser;
use std::path::PathBuf;

/// Generate Dart FFI bindings from JSON contracts
#[derive(Parser, Debug)]
#[command(name = "generate-dart-bindings")]
#[command(about = "Generate Dart FFI bindings from JSON contracts")]
#[command(version)]
pub struct Cli {
    /// Generate bindings for a specific domain only
    #[arg(short, long)]
    pub domain: Option<String>,

    /// Check if bindings are up-to-date without generating
    #[arg(short, long)]
    pub check: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Force regeneration, bypassing cache
    #[arg(short, long)]
    pub force: bool,

    /// Path to contracts directory (default: core/src/ffi/contracts)
    #[arg(long)]
    pub contracts: Option<PathBuf>,

    /// Path to output directory for bindings and models (default: packages/core_logic/lib)
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Path to output directory for IPC client (default: ui/lib)
    #[arg(long)]
    pub ipc_output: Option<PathBuf>,
}

impl Cli {
    /// Get the contracts directory path
    pub fn contracts_dir(&self) -> PathBuf {
        self.contracts
            .clone()
            .unwrap_or_else(|| PathBuf::from("core/src/ffi/contracts"))
    }

    /// Get the output directory path for bindings and models
    pub fn output_dir(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from("packages/core_logic/lib"))
    }

    /// Get the output directory path for IPC client
    pub fn ipc_output_dir(&self) -> PathBuf {
        self.ipc_output
            .clone()
            .unwrap_or_else(|| PathBuf::from("ui/lib"))
    }
}
