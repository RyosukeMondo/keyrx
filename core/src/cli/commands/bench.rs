//! Bench command for latency measurement.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::config::LATENCY_THRESHOLD_NS;
use crate::engine::{Engine, InputEvent, KeyCode};
use crate::mocks::{MockInput, MockState};
use crate::profiling::{
    FlameGraphConfig, FlameGraphGenerator, ProfileResult, Profiler, ProfilerConfig,
};
use crate::scripting::RhaiRuntime;
use crate::traits::ScriptRuntime;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
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
    /// Path to generated flame graph, when requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flamegraph: Option<FlamegraphArtifact>,
    /// Path and metadata for allocation report, when requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation_report: Option<AllocationReportArtifact>,
}

/// Metadata for generated flame graph artifacts.
#[derive(Debug, Serialize)]
pub struct FlamegraphArtifact {
    /// Where the flame graph SVG was written.
    pub path: PathBuf,
    /// Number of samples included in the flame graph.
    pub samples: usize,
    /// Duration captured by the profiler in milliseconds.
    pub duration_ms: u128,
}

/// Metadata for generated allocation reports.
#[derive(Debug, Serialize)]
pub struct AllocationReportArtifact {
    /// Where the allocation report JSON was written.
    pub path: PathBuf,
    /// Number of detected allocation hot spots.
    pub hot_spots: usize,
    /// Any warnings raised during report generation.
    pub warnings: Vec<String>,
}

/// Bench command for latency measurement.
pub struct BenchCommand {
    pub iterations: usize,
    pub script_path: Option<PathBuf>,
    pub output: OutputWriter,
    pub flamegraph_output: Option<PathBuf>,
    pub allocation_report_output: Option<PathBuf>,
    pub collect_allocations: bool,
}

impl BenchCommand {
    /// Default number of iterations.
    pub const DEFAULT_ITERATIONS: usize = 10000;

    pub fn new(iterations: usize, script_path: Option<PathBuf>, format: OutputFormat) -> Self {
        Self {
            iterations,
            script_path,
            output: OutputWriter::new(format),
            flamegraph_output: None,
            allocation_report_output: None,
            collect_allocations: false,
        }
    }

    /// Enable flame graph generation, optionally overriding the output path.
    pub fn with_flamegraph_output(mut self, path: Option<PathBuf>) -> Self {
        self.flamegraph_output = Some(path.unwrap_or_else(Self::default_flamegraph_path));
        self
    }

    /// Enable allocation report generation, optionally overriding the output path.
    pub fn with_allocation_report_output(mut self, path: Option<PathBuf>) -> Self {
        self.collect_allocations = true;
        self.allocation_report_output =
            Some(path.unwrap_or_else(Self::default_allocation_report_path));
        self
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
            flamegraph: None,
            allocation_report: None,
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

        if let Some(ref flamegraph) = result.flamegraph {
            self.output.success(&format!(
                "Flame graph written to {} ({} samples, {}ms)",
                flamegraph.path.display(),
                flamegraph.samples,
                flamegraph.duration_ms
            ));
        }

        if let Some(ref allocation) = result.allocation_report {
            let mut message = format!("Allocation report written to {}", allocation.path.display());
            if allocation.hot_spots > 0 {
                message.push_str(&format!(" ({} hot spots)", allocation.hot_spots));
            }
            self.output.success(&message);
            for warning in &allocation.warnings {
                self.output.warning(warning);
            }
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
        let mut profiler = self.build_profiler();

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

        // Start profiling after warmup to focus on measured iterations.
        if let Some(p) = profiler.as_mut() {
            p.start()?;
        }

        // Measurement phase
        let mut latencies = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let start = Instant::now();
            let _ = engine.process_event(&test_event);
            let elapsed = start.elapsed().as_nanos() as u64;
            latencies.push(elapsed);
        }

        let mut result = self.calculate_stats(&mut latencies);

        if let Some(mut profiler) = profiler {
            let profile = profiler.stop()?;
            self.attach_profiling_outputs(&profile, &mut result)?;
        }

        Ok(result)
    }

    fn attach_profiling_outputs(
        &self,
        profile: &ProfileResult,
        result: &mut BenchResult,
    ) -> anyhow::Result<()> {
        if let Some(path) = self.flamegraph_output.clone() {
            let svg = self.generate_flamegraph(profile);
            self.write_artifact(&path, &svg)?;

            result.flamegraph = Some(FlamegraphArtifact {
                path,
                samples: profile.sample_count,
                duration_ms: profile.duration.as_millis(),
            });
        }

        if self.collect_allocations {
            if let (Some(report_json), Some(report)) = (
                profile.allocation_report_json.as_ref(),
                profile.allocation_report.as_ref(),
            ) {
                let path = self
                    .allocation_report_output
                    .clone()
                    .unwrap_or_else(Self::default_allocation_report_path);
                self.write_artifact(&path, report_json)?;

                result.allocation_report = Some(AllocationReportArtifact {
                    path,
                    hot_spots: report.hot_spots.len(),
                    warnings: report.warnings.clone(),
                });
            }
        }

        Ok(())
    }

    fn generate_flamegraph(&self, profile: &ProfileResult) -> String {
        let config = FlameGraphConfig {
            title: format!("keyrx bench ({} iterations)", self.iterations),
            ..FlameGraphConfig::default()
        };

        let generator = FlameGraphGenerator::new(config);
        generator.generate(&profile.stack_samples)
    }

    fn write_artifact(&self, path: &Path, contents: &str) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, contents)?;
        Ok(())
    }

    fn build_profiler(&self) -> Option<Profiler> {
        if !self.profiling_enabled() {
            return None;
        }

        Some(Profiler::new(self.profiler_config()))
    }

    fn profiling_enabled(&self) -> bool {
        self.flamegraph_output.is_some() || self.collect_allocations
    }

    fn profiler_config(&self) -> ProfilerConfig {
        ProfilerConfig {
            stack_sampling: self.flamegraph_output.is_some(),
            allocation_tracking: self.collect_allocations,
            ..ProfilerConfig::default()
        }
    }

    fn default_flamegraph_path() -> PathBuf {
        PathBuf::from("bench-flamegraph.svg")
    }

    fn default_allocation_report_path() -> PathBuf {
        PathBuf::from("bench-allocations.json")
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
    use tempfile::tempdir;

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

    #[test]
    fn generates_profiling_artifacts_when_requested() {
        let temp = tempdir().expect("tempdir should be created");
        let flamegraph_path = temp.path().join("flame.svg");
        let allocation_path = temp.path().join("alloc.json");

        let cmd = BenchCommand::new(200, None, OutputFormat::Json)
            .with_flamegraph_output(Some(flamegraph_path.clone()))
            .with_allocation_report_output(Some(allocation_path.clone()));

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime should build");

        let result = rt
            .block_on(cmd.execute())
            .expect("bench execution should succeed");

        assert!(
            flamegraph_path.exists(),
            "flamegraph file should be written"
        );
        assert!(
            allocation_path.exists(),
            "allocation report file should be written"
        );
        assert!(result.flamegraph.is_some(), "flamegraph metadata missing");
        assert!(
            result.allocation_report.is_some(),
            "allocation metadata missing"
        );
    }
}
