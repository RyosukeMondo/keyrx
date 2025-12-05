//! Documentation generation command.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat};
use crate::scripting::docs::generators::{generate_html, generate_json, generate_markdown};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// Documentation generation command.
pub struct DocsCommand {
    output: OutputFormat,
    format: DocFormat,
    output_dir: PathBuf,
}

/// Documentation output format.
#[derive(Debug, Clone, Copy)]
pub enum DocFormat {
    /// Markdown documentation.
    Markdown,
    /// HTML documentation.
    Html,
    /// JSON schema.
    Json,
}

impl FromStr for DocFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(DocFormat::Markdown),
            "html" => Ok(DocFormat::Html),
            "json" => Ok(DocFormat::Json),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

impl DocFormat {
    /// Get file extension for this format.
    pub const fn extension(&self) -> &'static str {
        match self {
            DocFormat::Markdown => "md",
            DocFormat::Html => "html",
            DocFormat::Json => "json",
        }
    }

    /// Get default filename for this format.
    pub fn default_filename(&self) -> String {
        match self {
            DocFormat::Markdown => "api.md".to_string(),
            DocFormat::Html => "api.html".to_string(),
            DocFormat::Json => "api.json".to_string(),
        }
    }
}

impl DocsCommand {
    /// Create a new docs command.
    pub fn new(format: DocFormat, output_dir: PathBuf, output_format: OutputFormat) -> Self {
        Self {
            output: output_format,
            format,
            output_dir,
        }
    }

    /// Run the documentation generation.
    pub fn run(&self) -> CommandResult<()> {
        // Create output directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&self.output_dir) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to create output directory: {}", e),
            );
        }

        // Generate documentation in the requested format
        let content = match self.format {
            DocFormat::Markdown => generate_markdown(),
            DocFormat::Html => generate_html(),
            DocFormat::Json => generate_json(),
        };

        // Write to output file
        let output_file = self.output_dir.join(self.format.default_filename());
        if let Err(e) = fs::write(&output_file, &content) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to write output file: {}", e),
            );
        }

        // Print success message
        match self.output {
            OutputFormat::Json => {
                // In JSON mode, output structured data
                let output_data = serde_json::json!({
                    "success": true,
                    "format": format!("{:?}", self.format).to_lowercase(),
                    "output_file": output_file.display().to_string(),
                });
                if let Ok(json) = serde_json::to_string_pretty(&output_data) {
                    println!("{}", json);
                }
            }
            _ => {
                println!(
                    "Documentation generated successfully: {}",
                    output_file.display()
                );
            }
        }

        CommandResult::success(())
    }
}

impl Command for DocsCommand {
    fn name(&self) -> &str {
        "docs"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.run()
    }
}
