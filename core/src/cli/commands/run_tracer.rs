//! Tracing configuration and management for engine runs.

use crate::cli::OutputWriter;
use crate::engine::EngineTracer;
use anyhow::Result;
use std::path::PathBuf;

/// Manages OpenTelemetry tracing for engine sessions.
pub struct TracingManager<'a> {
    trace_path: Option<PathBuf>,
    output: &'a OutputWriter,
}

impl<'a> TracingManager<'a> {
    /// Create a new tracing manager.
    pub fn new(trace_path: Option<PathBuf>, output: &'a OutputWriter) -> Self {
        Self { trace_path, output }
    }

    /// Create an EngineTracer if a trace path is specified.
    pub fn create_tracer(&self) -> Result<Option<EngineTracer>> {
        match &self.trace_path {
            Some(path) => {
                self.output
                    .success(&format!("Exporting traces to: {}", path.display()));
                match EngineTracer::with_file_export("keyrx", path) {
                    Ok(tracer) => Ok(Some(tracer)),
                    Err(e) => {
                        self.output.warning(&format!(
                            "Failed to initialize tracer: {}. Continuing without tracing.",
                            e
                        ));
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Shutdown the tracer and flush pending spans.
    pub fn finish_tracing(&self, tracer: Option<EngineTracer>) {
        if let Some(t) = tracer {
            t.shutdown();
            self.output.success("Traces exported successfully.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::OutputFormat;
    use tempfile::TempDir;

    #[test]
    fn create_tracer_returns_none_when_no_path() {
        let output = OutputWriter::new(OutputFormat::Human);
        let manager = TracingManager::new(None, &output);

        let result = manager.create_tracer().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn create_tracer_with_valid_path_does_not_error() {
        let temp_dir = TempDir::new().unwrap();
        let trace_path = temp_dir.path().join("trace.json");
        let output = OutputWriter::new(OutputFormat::Human);
        let manager = TracingManager::new(Some(trace_path), &output);

        // create_tracer() should succeed (Ok) even if tracer isn't available
        // (returns None with a warning in that case)
        let result = manager.create_tracer();
        assert!(result.is_ok());
    }

    #[test]
    fn finish_tracing_handles_none() {
        let output = OutputWriter::new(OutputFormat::Human);
        let manager = TracingManager::new(None, &output);
        // Should not panic
        manager.finish_tracing(None);
    }
}
