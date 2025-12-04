//! Bench command for latency measurement.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::config::LATENCY_THRESHOLD_NS;
use crate::engine::{Engine, InputEvent, KeyCode};
use crate::mocks::{MockInput, MockState};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

/// Benchmark result statistics.
#[derive(Debug, Serialize)]
pub struct BenchResult {
    /// Minimum latency in nanoseconds.
    pub min_ns: u64,
    /// Maximum latency in nanoseconds.
    pub max_ns: u64,
    /// Mean latency in nanoseconds.
    pub mean_ns: u64,
    /// 99th percentile latency in nanoseconds.
    pub p99_ns: u64,
    /// Total iterations run.
    pub iterations: usize,
    /// Warning message if performance threshold exceeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Bench command for latency measurement.
pub struct BenchCommand {
    pub iterations: usize,
    pub script_path: Option<PathBuf>,
    pub output: OutputWriter,
}

impl BenchCommand {
    /// Default number of iterations.
    pub const DEFAULT_ITERATIONS: usize = 10000;

    pub fn new(iterations: usize, script_path: Option<PathBuf>, format: OutputFormat) -> Self {
        Self {
            iterations,
            script_path,
            output: OutputWriter::new(format),
        }
    }

    /// Calculate statistics from latency measurements.
    fn calculate_stats(&self, latencies: &mut [u64]) -> BenchResult {
        latencies.sort_unstable();

        let min_ns = latencies.first().copied().unwrap_or(0);
        let max_ns = latencies.last().copied().unwrap_or(0);
        let sum: u64 = latencies.iter().sum();
        let mean_ns = if latencies.is_empty() {
            0
        } else {
            sum / latencies.len() as u64
        };

        // Calculate p99 index
        let p99_idx = ((latencies.len() as f64 * 0.99) as usize).saturating_sub(1);
        let p99_ns = latencies.get(p99_idx).copied().unwrap_or(max_ns);

        let warning = if mean_ns > LATENCY_THRESHOLD_NS {
            Some(format!(
                "Mean latency ({:.2}ms) exceeds 1ms threshold",
                mean_ns as f64 / 1_000_000.0
            ))
        } else {
            None
        };

        BenchResult {
            min_ns,
            max_ns,
            mean_ns,
            p99_ns,
            iterations: latencies.len(),
            warning,
        }
    }

    pub async fn run(&self) -> CommandResult<()> {
        let result = match self.execute().await {
            Ok(r) => r,
            Err(e) => {
                return CommandResult::failure(
                    ExitCode::GeneralError,
                    format!("Benchmark failed: {}", e),
                )
            }
        };

        // Output warning to stderr if present
        if let Some(ref warning) = result.warning {
            self.output.error(warning);
        }

        if let Err(e) = self.output.data(&result) {
            return CommandResult::failure(
                ExitCode::GeneralError,
                format!("Failed to output results: {}", e),
            );
        }

        CommandResult::success(())
    }

    /// Execute benchmark and return results directly.
    pub async fn execute(&self) -> anyhow::Result<BenchResult> {
        // Create runtime and load script if provided
        let mut runtime = RhaiRuntime::new()?;
        if let Some(path) = &self.script_path {
            let path_str = path.to_string_lossy();
            runtime.load_file(&path_str)?;

            // Call on_init if defined
            if runtime.has_hook("on_init") {
                runtime.call_hook("on_init")?;
            }
        }

        // Create engine with mocks
        let mock_input = MockInput::new();
        let engine = Engine::new(
            mock_input,
            runtime,
            MockState::new(),
            crate::metrics::default_noop_collector(),
        );

        // Create test event
        let test_event = InputEvent::key_down(KeyCode::A, 0);

        // Warmup phase (10% of iterations or minimum 100)
        let warmup_count = (self.iterations / 10).max(100).min(self.iterations);
        for _ in 0..warmup_count {
            let _ = engine.process_event(&test_event);
        }

        // Measurement phase
        let mut latencies = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let start = Instant::now();
            let _ = engine.process_event(&test_event);
            let elapsed = start.elapsed().as_nanos() as u64;
            latencies.push(elapsed);
        }

        // Calculate and return results
        Ok(self.calculate_stats(&mut latencies))
    }
}

impl Command for BenchCommand {
    fn name(&self) -> &str {
        "bench"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        // Create a new runtime for async execution
        let rt = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => {
                return CommandResult::failure(
                    ExitCode::GeneralError,
                    format!("Failed to create tokio runtime: {err}"),
                )
            }
        };

        // Run the async logic
        rt.block_on(self.run())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_stats_works() {
        let cmd = BenchCommand::new(100, None, OutputFormat::Human);
        let mut latencies: Vec<u64> = (1..=100).collect();

        let result = cmd.calculate_stats(&mut latencies);

        assert_eq!(result.min_ns, 1);
        assert_eq!(result.max_ns, 100);
        assert_eq!(result.mean_ns, 50);
        assert_eq!(result.p99_ns, 99);
        assert_eq!(result.iterations, 100);
        assert!(result.warning.is_none());
    }

    #[test]
    fn calculate_stats_warning_on_high_latency() {
        let cmd = BenchCommand::new(10, None, OutputFormat::Human);
        // All latencies above 1ms (1,000,000 ns)
        let mut latencies: Vec<u64> = vec![2_000_000; 10];

        let result = cmd.calculate_stats(&mut latencies);

        assert!(result.warning.is_some());
        assert!(result.warning.unwrap().contains("exceeds 1ms threshold"));
    }

    #[test]
    fn calculate_stats_empty() {
        let cmd = BenchCommand::new(0, None, OutputFormat::Human);
        let mut latencies: Vec<u64> = vec![];

        let result = cmd.calculate_stats(&mut latencies);

        assert_eq!(result.min_ns, 0);
        assert_eq!(result.max_ns, 0);
        assert_eq!(result.mean_ns, 0);
        assert_eq!(result.p99_ns, 0);
        assert_eq!(result.iterations, 0);
    }
}
