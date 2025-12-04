//! Binary to generate error documentation from the error registry.
//!
//! This tool reads the error registry and generates markdown documentation
//! files for all error categories.
//!
//! This binary uses println! for user-facing output.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use keyrx_core::errors::{ErrorCategory, ErrorDocGenerator};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docs_dir = Path::new("docs/errors");

    // Ensure docs directory exists
    fs::create_dir_all(docs_dir)?;

    println!("Generating error documentation...");

    // Generate index page
    let index = ErrorDocGenerator::generate_index();
    let index_path = docs_dir.join("index.md");
    fs::write(&index_path, index)?;
    println!("  ✓ Generated {}", index_path.display());

    // Generate documentation for each category
    let categories = [
        ErrorCategory::Config,
        ErrorCategory::Runtime,
        ErrorCategory::Driver,
        ErrorCategory::Validation,
        ErrorCategory::Ffi,
        ErrorCategory::Internal,
    ];

    for category in &categories {
        let docs = ErrorDocGenerator::generate_category(*category);
        let filename = match category {
            ErrorCategory::Config => "config.md",
            ErrorCategory::Runtime => "runtime.md",
            ErrorCategory::Driver => "driver.md",
            ErrorCategory::Validation => "validation.md",
            ErrorCategory::Ffi => "ffi.md",
            ErrorCategory::Internal => "internal.md",
        };
        let file_path = docs_dir.join(filename);
        fs::write(&file_path, docs)?;
        println!("  ✓ Generated {}", file_path.display());
    }

    println!("\nError documentation generated successfully!");
    println!("Location: {}", docs_dir.display());

    Ok(())
}
