//! UAT command for running User Acceptance Tests.

use crate::cli::{
    Command, CommandContext, CommandError, CommandResult, ExitCode, OutputFormat, OutputWriter,
};
use crate::uat::{
    CoverageMapper, FuzzConfig, FuzzEngine, GateResult, PerformanceUat, QualityGateEnforcer,
    ReportData, ReportGenerator, UatFilter, UatRunner,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Exit codes for UAT command.
pub mod exit_codes {
    // Legacy exit code constants for backward compatibility
    pub const PASS: i32 = 0;
    pub const TEST_FAIL: i32 = 1;
    pub const GATE_FAIL: i32 = 2;
    pub const CRASH: i32 = 3;
}

/// UAT command for running User Acceptance Tests.
pub struct UatCommand {
    /// Filter by categories.
    pub categories: Vec<String>,
    /// Filter by priorities.
    pub priorities: Vec<String>,
    /// Output in JSON format.
    pub json: bool,
    /// Fail fast on first test failure.
    pub fail_fast: bool,
    /// Run performance tests.
    pub perf: bool,
    /// Run fuzz tests.
    pub fuzz: bool,
    /// Fuzz test duration in seconds.
    pub fuzz_duration: u64,
    /// Fuzz test sequence count.
    pub fuzz_count: Option<u64>,
    /// Generate coverage report.
    pub coverage_report: bool,
    /// Generate full report.
    pub report: bool,
    /// Report format (html, md, json).
    pub report_format: String,
    /// Report output path.
    pub report_output: Option<PathBuf>,
    /// Quality gate name.
    pub gate: Option<String>,
    /// Output writer.
    pub output: OutputWriter,
}

impl UatCommand {
    /// Create a new UAT command with default settings.
    pub fn new(format: OutputFormat) -> Self {
        Self {
            categories: Vec::new(),
            priorities: Vec::new(),
            json: matches!(format, OutputFormat::Json),
            fail_fast: false,
            perf: false,
            fuzz: false,
            fuzz_duration: 60,
            fuzz_count: None,
            coverage_report: false,
            report: false,
            report_format: "html".to_string(),
            report_output: None,
            gate: None,
            output: OutputWriter::new(format),
        }
    }

    /// Set category filters.
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = categories;
        self
    }

    /// Set priority filters.
    pub fn with_priorities(mut self, priorities: Vec<String>) -> Self {
        self.priorities = priorities;
        self
    }

    /// Enable JSON output.
    pub fn with_json(mut self, json: bool) -> Self {
        self.json = json;
        if json {
            self.output = OutputWriter::new(OutputFormat::Json);
        }
        self
    }

    /// Enable fail-fast mode.
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Enable performance tests.
    pub fn with_perf(mut self, perf: bool) -> Self {
        self.perf = perf;
        self
    }

    /// Enable fuzz tests.
    pub fn with_fuzz(mut self, fuzz: bool) -> Self {
        self.fuzz = fuzz;
        self
    }

    /// Set fuzz duration.
    pub fn with_fuzz_duration(mut self, duration: u64) -> Self {
        self.fuzz_duration = duration;
        self
    }

    /// Set fuzz count.
    pub fn with_fuzz_count(mut self, count: Option<u64>) -> Self {
        self.fuzz_count = count;
        self
    }

    /// Enable coverage report.
    pub fn with_coverage_report(mut self, coverage_report: bool) -> Self {
        self.coverage_report = coverage_report;
        self
    }

    /// Enable full report.
    pub fn with_report(mut self, report: bool) -> Self {
        self.report = report;
        self
    }

    /// Set report format.
    pub fn with_report_format(mut self, format: String) -> Self {
        self.report_format = format;
        self
    }

    /// Set report output path.
    pub fn with_report_output(mut self, output: Option<PathBuf>) -> Self {
        self.report_output = output;
        self
    }

    /// Set quality gate.
    pub fn with_gate(mut self, gate: Option<String>) -> Self {
        self.gate = gate;
        self
    }

    /// Execute the UAT command with CommandResult.
    ///
    /// Returns a CommandResult indicating test success or failure with pass/fail counts.
    pub fn execute_uat(&self) -> CommandResult<()> {
        match self.run_internal() {
            Ok((passed, failed, gate_passed, crash_count)) => {
                if crash_count > 0 {
                    CommandResult::failure(
                        ExitCode::Panic,
                        format!("Fuzz testing found {} crash(es)", crash_count),
                    )
                } else if failed > 0 {
                    CommandResult::failure(
                        ExitCode::AssertionFailed,
                        CommandError::test_failure(
                            format!("{} test(s) failed", failed),
                            passed,
                            failed,
                        )
                        .to_string(),
                    )
                } else if !gate_passed {
                    CommandResult::failure(
                        ExitCode::AssertionFailed,
                        "Quality gate criteria not met".to_string(),
                    )
                } else {
                    CommandResult::success_with_message(
                        (),
                        format!("All {} test(s) passed", passed),
                    )
                }
            }
            Err(e) => CommandResult::failure(ExitCode::GeneralError, e.to_string()),
        }
    }

    /// Run the UAT command.
    ///
    /// Returns the exit code:
    /// - 0: All tests passed and gate passed (if specified)
    /// - 1: Test failure
    /// - 2: Gate failure (tests passed but gate criteria not met)
    /// - 3: Crash detected (in fuzz testing)
    pub fn run(&self) -> Result<i32> {
        match self.run_internal() {
            Ok((_, failed, gate_passed, crash_count)) => {
                if crash_count > 0 {
                    Ok(exit_codes::CRASH)
                } else if failed > 0 {
                    Ok(exit_codes::TEST_FAIL)
                } else if !gate_passed {
                    Ok(exit_codes::GATE_FAIL)
                } else {
                    Ok(exit_codes::PASS)
                }
            }
            Err(_) => Ok(exit_codes::TEST_FAIL),
        }
    }

    fn run_internal(&self) -> Result<(usize, usize, bool, usize)> {
        // Build filter from options
        let filter = self.build_filter();

        // Run UAT tests
        let runner = UatRunner::new();
        let uat_results = if self.fail_fast {
            runner.run_fail_fast(&filter)
        } else {
            runner.run(&filter)
        };

        // Run performance tests if requested
        let perf_results = if self.perf {
            let perf_runner = PerformanceUat::new();
            Some(perf_runner.run(&filter))
        } else {
            None
        };

        // Run fuzz tests if requested
        let (fuzz_result, crash_count) = if self.fuzz {
            let fuzz_engine = FuzzEngine::new(FuzzConfig {
                min_sequences: self.fuzz_count.unwrap_or(10_000) as usize,
                max_duration: Duration::from_secs(self.fuzz_duration),
                ..Default::default()
            });
            let result = fuzz_engine.run(Duration::from_secs(self.fuzz_duration), self.fuzz_count);

            let crashes = result.crashes.len();
            // Report crashes but don't return early - let caller handle exit
            if crashes > 0 {
                self.output
                    .error(&format!("Fuzz testing found {} crash(es)!", crashes));
                for crash in &result.crashes {
                    self.output.error(&format!(
                        "  Crash saved to: {} - {}",
                        crash.file_path, crash.error
                    ));
                }
            }

            (Some(result), crashes)
        } else {
            (None, 0)
        };

        // Build coverage map if requested or for report
        let coverage_report = if self.coverage_report || self.report {
            let tests = runner.discover();
            let mapper = CoverageMapper::new();
            let map = mapper.build(&tests, &uat_results);
            Some(mapper.report(&map))
        } else {
            None
        };

        // Evaluate quality gate if specified
        let gate_result = if let Some(ref gate_name) = self.gate {
            let enforcer = QualityGateEnforcer::new();
            let gate = enforcer
                .load(Some(gate_name))
                .context(format!("Failed to load quality gate '{}'", gate_name))?;
            Some(enforcer.evaluate(&gate, &uat_results))
        } else {
            None
        };

        // Generate and output report if requested
        if self.report {
            self.generate_report(
                &uat_results,
                coverage_report.as_ref(),
                perf_results.as_ref(),
                gate_result.as_ref(),
            )?;
        }

        // Output results
        self.output_results(
            &uat_results,
            coverage_report.as_ref(),
            perf_results.as_ref(),
            fuzz_result.as_ref(),
            gate_result.as_ref(),
        )?;

        // Return results: (passed, failed, gate_passed, crash_count)
        let gate_passed = gate_result.as_ref().is_none_or(|g| g.passed);
        Ok((
            uat_results.passed,
            uat_results.failed,
            gate_passed,
            crash_count,
        ))
    }

    /// Build a filter from the command options.
    fn build_filter(&self) -> UatFilter {
        use crate::uat::Priority;

        let priorities: Vec<Priority> = self
            .priorities
            .iter()
            .filter_map(|p| p.parse().ok())
            .collect();

        UatFilter {
            categories: self.categories.clone(),
            priorities,
            pattern: None,
        }
    }

    /// Generate and write a report to file.
    fn generate_report(
        &self,
        uat_results: &crate::uat::UatResults,
        coverage: Option<&crate::uat::CoverageReport>,
        perf: Option<&crate::uat::PerfResults>,
        gate: Option<&GateResult>,
    ) -> Result<()> {
        let mut report_data = ReportData::new(uat_results.clone()).with_title("KeyRx UAT Report");

        if let Some(cov) = coverage {
            report_data = report_data.with_coverage(cov.clone());
        }

        if let Some(p) = perf {
            report_data = report_data.with_performance(p.clone());
        }

        if let Some(g) = gate {
            report_data = report_data.with_gate_result(g.clone());
        }

        let generator = ReportGenerator::new();
        let content = match self.report_format.to_lowercase().as_str() {
            "md" | "markdown" => generator.generate_markdown(&report_data),
            "json" => generator.generate_json(&report_data),
            _ => generator.generate_html(&report_data),
        };

        // Determine output path
        let output_path = self.report_output.clone().unwrap_or_else(|| {
            let ext = match self.report_format.to_lowercase().as_str() {
                "md" | "markdown" => "md",
                "json" => "json",
                _ => "html",
            };
            PathBuf::from(format!("uat-report.{}", ext))
        });

        fs::write(&output_path, content).context(format!(
            "Failed to write report to {}",
            output_path.display()
        ))?;

        if !self.json {
            println!("Report written to: {}", output_path.display());
        }

        Ok(())
    }

    /// Output results to console.
    fn output_results(
        &self,
        uat_results: &crate::uat::UatResults,
        coverage: Option<&crate::uat::CoverageReport>,
        perf: Option<&crate::uat::PerfResults>,
        fuzz: Option<&crate::uat::FuzzResult>,
        gate: Option<&GateResult>,
    ) -> Result<()> {
        if self.json {
            // Build combined JSON output
            #[derive(serde::Serialize)]
            struct UatOutput<'a> {
                uat_results: &'a crate::uat::UatResults,
                #[serde(skip_serializing_if = "Option::is_none")]
                coverage: Option<&'a crate::uat::CoverageReport>,
                #[serde(skip_serializing_if = "Option::is_none")]
                performance: Option<&'a crate::uat::PerfResults>,
                #[serde(skip_serializing_if = "Option::is_none")]
                fuzz: Option<&'a crate::uat::FuzzResult>,
                #[serde(skip_serializing_if = "Option::is_none")]
                gate_result: Option<&'a GateResult>,
            }

            let output = UatOutput {
                uat_results,
                coverage,
                performance: perf,
                fuzz,
                gate_result: gate,
            };

            self.output.data(&output)?;
        } else {
            self.output_human_results(uat_results, coverage, perf, fuzz, gate);
        }

        Ok(())
    }

    /// Output human-readable results.
    fn output_human_results(
        &self,
        uat_results: &crate::uat::UatResults,
        coverage: Option<&crate::uat::CoverageReport>,
        perf: Option<&crate::uat::PerfResults>,
        fuzz: Option<&crate::uat::FuzzResult>,
        gate: Option<&GateResult>,
    ) {
        // Summary header
        println!("\n{}", "═".repeat(60));
        println!("  UAT Results Summary");
        println!("{}", "═".repeat(60));

        // Test results
        let pass_rate = if uat_results.total > 0 {
            (uat_results.passed as f64 / uat_results.total as f64) * 100.0
        } else {
            100.0
        };

        println!(
            "\n  Tests: {} total, {} passed, {} failed, {} skipped",
            uat_results.total, uat_results.passed, uat_results.failed, uat_results.skipped
        );
        println!(
            "  Pass Rate: {:.1}% ({} µs)",
            pass_rate, uat_results.duration_us
        );

        // Failed tests details
        if uat_results.failed > 0 {
            println!("\n  Failed Tests:");
            for result in &uat_results.results {
                if !result.passed {
                    let priority = match result.test.priority {
                        crate::uat::Priority::P0 => "P0",
                        crate::uat::Priority::P1 => "P1",
                        crate::uat::Priority::P2 => "P2",
                    };
                    println!(
                        "    ✗ {} [{}] ({})",
                        result.test.name, priority, result.test.category
                    );
                    if let Some(ref error) = result.error {
                        println!("      └─ {}", error);
                    }
                }
            }
        }

        // Coverage report
        if let Some(cov) = coverage {
            println!("\n  Coverage:");
            println!(
                "    Requirements: {} total, {} verified, {} at-risk, {} uncovered",
                cov.total, cov.verified, cov.at_risk, cov.uncovered
            );
            println!("    Coverage: {:.1}%", cov.coverage_percentage * 100.0);
        }

        // Performance results
        if let Some(p) = perf {
            println!("\n  Performance:");
            println!(
                "    Tests: {} total, {} passed, {} failed",
                p.total, p.passed, p.failed
            );
            println!(
                "    Latency: p50={}µs, p95={}µs, p99={}µs, max={}µs",
                p.aggregate_p50_us, p.aggregate_p95_us, p.aggregate_p99_us, p.aggregate_max_us
            );
            if !p.all_violations.is_empty() {
                println!("    Violations: {}", p.all_violations.len());
            }
        }

        // Fuzz results
        if let Some(f) = fuzz {
            println!("\n  Fuzz Testing:");
            println!(
                "    Sequences: {} tested in {:.1}s",
                f.sequences_tested, f.duration_secs
            );
            println!("    Unique Paths: {}", f.unique_paths);
            println!("    Crashes: {}", f.crashes.len());
        }

        // Gate result
        if let Some(g) = gate {
            println!("\n  Quality Gate:");
            if g.passed {
                println!("    Status: PASSED ✓");
            } else {
                println!("    Status: FAILED ✗");
                for violation in &g.violations {
                    println!(
                        "    - {}: expected {}, got {}",
                        violation.criterion, violation.expected, violation.actual
                    );
                }
            }
        }

        println!("\n{}", "═".repeat(60));
    }
}

impl Command for UatCommand {
    fn name(&self) -> &str {
        "uat"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        self.execute_uat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(exit_codes::PASS, 0);
        assert_eq!(exit_codes::TEST_FAIL, 1);
        assert_eq!(exit_codes::GATE_FAIL, 2);
        assert_eq!(exit_codes::CRASH, 3);
    }

    #[test]
    fn uat_command_new() {
        let cmd = UatCommand::new(OutputFormat::Human);
        assert!(cmd.categories.is_empty());
        assert!(cmd.priorities.is_empty());
        assert!(!cmd.json);
        assert!(!cmd.fail_fast);
        assert!(!cmd.perf);
        assert!(!cmd.fuzz);
    }

    #[test]
    fn uat_command_builder_methods() {
        let cmd = UatCommand::new(OutputFormat::Human)
            .with_categories(vec!["core".to_string()])
            .with_priorities(vec!["P0".to_string()])
            .with_fail_fast(true)
            .with_perf(true)
            .with_fuzz(true)
            .with_fuzz_duration(30)
            .with_coverage_report(true)
            .with_report(true)
            .with_report_format("md".to_string())
            .with_gate(Some("beta".to_string()));

        assert_eq!(cmd.categories, vec!["core"]);
        assert_eq!(cmd.priorities, vec!["P0"]);
        assert!(cmd.fail_fast);
        assert!(cmd.perf);
        assert!(cmd.fuzz);
        assert_eq!(cmd.fuzz_duration, 30);
        assert!(cmd.coverage_report);
        assert!(cmd.report);
        assert_eq!(cmd.report_format, "md");
        assert_eq!(cmd.gate, Some("beta".to_string()));
    }

    #[test]
    fn build_filter_empty() {
        let cmd = UatCommand::new(OutputFormat::Human);
        let filter = cmd.build_filter();

        assert!(filter.categories.is_empty());
        assert!(filter.priorities.is_empty());
        assert!(filter.pattern.is_none());
    }

    #[test]
    fn build_filter_with_categories() {
        let cmd = UatCommand::new(OutputFormat::Human)
            .with_categories(vec!["core".to_string(), "layers".to_string()]);
        let filter = cmd.build_filter();

        assert_eq!(filter.categories, vec!["core", "layers"]);
    }

    #[test]
    fn build_filter_with_priorities() {
        let cmd = UatCommand::new(OutputFormat::Human)
            .with_priorities(vec!["P0".to_string(), "P1".to_string()]);
        let filter = cmd.build_filter();

        assert_eq!(filter.priorities.len(), 2);
    }

    #[test]
    fn with_json_enables_json_output() {
        let cmd = UatCommand::new(OutputFormat::Human).with_json(true);
        assert!(cmd.json);
    }
}
