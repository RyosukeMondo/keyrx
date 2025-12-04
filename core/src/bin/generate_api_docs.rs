//! Binary to generate API documentation from the DocRegistry.
//!
//! This tool generates comprehensive API documentation in multiple formats:
//! - Markdown: Human-readable documentation
//! - HTML: Interactive, searchable web documentation
//! - JSON: Machine-readable schema for IDE integration
//!
//! This binary uses println! for user-facing output.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use keyrx_core::scripting::docs::generators::{generate_html, generate_json, generate_markdown};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docs_dir = Path::new("docs/api");

    // Ensure docs directory exists
    fs::create_dir_all(docs_dir)?;

    println!("Generating API documentation...");

    // Generate Markdown documentation
    let markdown = generate_markdown();
    let md_path = docs_dir.join("api.md");
    fs::write(&md_path, markdown)?;
    println!("  ✓ Generated {}", md_path.display());

    // Generate HTML documentation
    let html = generate_html();
    let html_path = docs_dir.join("api.html");
    fs::write(&html_path, html)?;
    println!("  ✓ Generated {}", html_path.display());

    // Generate JSON schema
    let json = generate_json();
    let json_path = docs_dir.join("api.json");
    fs::write(&json_path, json)?;
    println!("  ✓ Generated {}", json_path.display());

    println!("\nAPI documentation generated successfully!");
    println!("Location: {}", docs_dir.display());
    println!("\nGenerated files:");
    println!("  - api.md   : Markdown documentation");
    println!("  - api.html : Interactive HTML documentation");
    println!("  - api.json : JSON schema for IDE integration");

    Ok(())
}
