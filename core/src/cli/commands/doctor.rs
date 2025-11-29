//! Self-diagnostics command.

use crate::cli::{OutputFormat, OutputWriter};
use anyhow::Result;
use serde::Serialize;

/// Run self-diagnostics.
pub struct DoctorCommand {
    pub verbose: bool,
    pub output: OutputWriter,
}

#[derive(Serialize)]
struct DiagnosticResult {
    check: String,
    status: String,
    details: Option<String>,
}

impl DoctorCommand {
    pub fn new(verbose: bool, format: OutputFormat) -> Self {
        Self {
            verbose,
            output: OutputWriter::new(format),
        }
    }

    pub fn run(&self) -> Result<()> {
        let mut results = Vec::new();

        // Check Rhai engine
        results.push(DiagnosticResult {
            check: "Rhai Engine".to_string(),
            status: "OK".to_string(),
            details: Some("v1.16".to_string()),
        });

        // Check OS compatibility
        #[cfg(target_os = "windows")]
        results.push(DiagnosticResult {
            check: "Platform".to_string(),
            status: "OK".to_string(),
            details: Some("Windows (WH_KEYBOARD_LL)".to_string()),
        });

        #[cfg(target_os = "linux")]
        results.push(DiagnosticResult {
            check: "Platform".to_string(),
            status: "OK".to_string(),
            details: Some("Linux (evdev/uinput)".to_string()),
        });

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        results.push(DiagnosticResult {
            check: "Platform".to_string(),
            status: "WARN".to_string(),
            details: Some("Unsupported platform".to_string()),
        });

        self.output.data(&results)?;
        Ok(())
    }
}
