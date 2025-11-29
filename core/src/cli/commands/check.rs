//! Script validation command.

use crate::cli::{OutputFormat, OutputWriter};
use anyhow::Result;
use std::path::PathBuf;

/// Validate and lint a Rhai script.
pub struct CheckCommand {
    pub script_path: PathBuf,
    pub output: OutputWriter,
}

impl CheckCommand {
    pub fn new(script_path: PathBuf, format: OutputFormat) -> Self {
        Self {
            script_path,
            output: OutputWriter::new(format),
        }
    }

    pub fn run(&self) -> Result<()> {
        let script = std::fs::read_to_string(&self.script_path)?;

        // Parse with Rhai to check syntax
        let engine = rhai::Engine::new();
        match engine.compile(&script) {
            Ok(_ast) => {
                self.output.success(&format!(
                    "Script '{}' is valid",
                    self.script_path.display()
                ));
                Ok(())
            }
            Err(e) => {
                self.output.error(&format!("Parse error: {}", e));
                std::process::exit(2);
            }
        }
    }
}
