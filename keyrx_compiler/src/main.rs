//! keyrx_compiler - Rhai-to-binary configuration compiler
//!
//! This binary compiles Rhai DSL configuration scripts into static .krx binary files.

use clap::Parser;

mod dfa_gen;
mod error;
mod mphf_gen;
mod parser;
mod serialize;

#[derive(Parser)]
#[command(name = "keyrx_compiler")]
#[command(about = "Compile Rhai configuration scripts to .krx binary files")]
struct Cli {
    /// Input Rhai configuration file
    #[arg(value_name = "INPUT")]
    input: String,

    /// Output .krx binary file
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Input: {}", cli.input);
        println!("Output: {:?}", cli.output);
    }

    // Placeholder - implementation to follow
    println!("keyrx_compiler placeholder - compilation not yet implemented");
}
