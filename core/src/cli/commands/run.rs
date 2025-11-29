//! Engine run command.

use crate::cli::{OutputFormat, OutputWriter};
use anyhow::Result;
use std::path::PathBuf;

/// Run the engine in headless mode.
pub struct RunCommand {
    pub script_path: Option<PathBuf>,
    pub debug: bool,
    pub output: OutputWriter,
}

impl RunCommand {
    pub fn new(script_path: Option<PathBuf>, debug: bool, format: OutputFormat) -> Self {
        Self {
            script_path,
            debug,
            output: OutputWriter::new(format),
        }
    }

    pub async fn run(&self) -> Result<()> {
        self.output.success("Starting KeyRx engine...");

        if let Some(path) = &self.script_path {
            self.output
                .success(&format!("Loading script: {}", path.display()));
        }

        if self.debug {
            self.output.success("Debug mode enabled");
        }

        // TODO: Initialize engine with real/mock input source
        self.output
            .success("Engine started. Press Ctrl+C to stop.");

        // Keep running until interrupted
        tokio::signal::ctrl_c().await?;
        self.output.success("Engine stopped.");

        Ok(())
    }
}
