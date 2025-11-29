//! Output formatting for CLI commands.

use serde::Serialize;
use std::io::{self, Write};

/// Output format selection.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Human,
    /// JSON output for programmatic parsing.
    Json,
}

/// Writer for formatted CLI output.
pub struct OutputWriter {
    format: OutputFormat,
}

impl OutputWriter {
    /// Create a new output writer.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Get the output format.
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Write a success message.
    pub fn success(&self, message: &str) {
        match self.format {
            OutputFormat::Human => println!("[OK] {}", message),
            OutputFormat::Json => {
                println!(r#"{{"status":"success","message":"{}"}}"#, message);
            }
        }
    }

    /// Write an error message.
    pub fn error(&self, message: &str) {
        match self.format {
            OutputFormat::Human => eprintln!("[ERROR] {}", message),
            OutputFormat::Json => {
                eprintln!(r#"{{"status":"error","message":"{}"}}"#, message);
            }
        }
    }

    /// Write a warning message.
    pub fn warning(&self, message: &str) {
        match self.format {
            OutputFormat::Human => println!("[WARN] {}", message),
            OutputFormat::Json => {
                println!(r#"{{"status":"warning","message":"{}"}}"#, message);
            }
        }
    }

    /// Write structured data.
    pub fn data<T: Serialize + ?Sized>(&self, data: &T) -> io::Result<()> {
        match self.format {
            OutputFormat::Human => {
                // Pretty print for humans
                let json = serde_json::to_string_pretty(data)?;
                println!("{}", json);
            }
            OutputFormat::Json => {
                // Compact JSON for machines
                let json = serde_json::to_string(data)?;
                println!("{}", json);
            }
        }
        io::stdout().flush()
    }
}
