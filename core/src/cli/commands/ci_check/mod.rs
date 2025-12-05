//! CI check command for unified test execution.
//!
//! Runs all test types: unit tests, integration tests, UAT tests,
//! regression tests, and performance tests with quality gate enforcement.

mod ci_check_phases;
mod ci_check_summary;

pub use ci_check_summary::{exit_codes, CheckPhaseResult, CiCheckSummary};

use crate::cli::{OutputFormat, OutputWriter};
use crate::uat::{QualityGateEnforcer, UatResults};
use anyhow::{Context, Result};

use ci_check_phases::{
    run_integration_tests, run_performance_tests, run_regression_tests, run_uat_tests,
    run_unit_tests,
};
use ci_check_summary::{output_human_summary, output_phase_result};

/// CI check command for running all tests in CI pipeline.
pub struct CiCheckCommand {
    /// Quality gate name to enforce.
    pub gate: Option<String>,
    /// Output in JSON format.
    pub json: bool,
    /// Skip unit tests.
    pub skip_unit: bool,
    /// Skip integration tests.
    pub skip_integration: bool,
    /// Skip UAT tests.
    pub skip_uat: bool,
    /// Skip regression tests.
    pub skip_regression: bool,
    /// Skip performance tests.
    pub skip_perf: bool,
    /// Output writer.
    pub output: OutputWriter,
}

impl CiCheckCommand {
    /// Create a new CI check command.
    pub fn new(format: OutputFormat) -> Self {
        Self {
            gate: None,
            json: matches!(format, OutputFormat::Json | OutputFormat::Yaml),
            skip_unit: false,
            skip_integration: false,
            skip_uat: false,
            skip_regression: false,
            skip_perf: false,
            output: OutputWriter::new(format),
        }
    }

    /// Set quality gate.
    pub fn with_gate(mut self, gate: Option<String>) -> Self {
        self.gate = gate;
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

    /// Skip unit tests.
    pub fn with_skip_unit(mut self, skip: bool) -> Self {
        self.skip_unit = skip;
        self
    }

    /// Skip integration tests.
    pub fn with_skip_integration(mut self, skip: bool) -> Self {
        self.skip_integration = skip;
        self
    }

    /// Skip UAT tests.
    pub fn with_skip_uat(mut self, skip: bool) -> Self {
        self.skip_uat = skip;
        self
    }

    /// Skip regression tests.
    pub fn with_skip_regression(mut self, skip: bool) -> Self {
        self.skip_regression = skip;
        self
    }

    /// Skip performance tests.
    pub fn with_skip_perf(mut self, skip: bool) -> Self {
        self.skip_perf = skip;
        self
    }

    /// Run the CI check command.
    ///
    /// Returns the exit code:
    /// - 0: All checks passed and gate passed
    /// - 1: Test failure
    /// - 2: Gate failure
    /// - 3: Crash detected
    pub fn run(&self) -> Result<i32> {
        let mut summary = CiCheckSummary::new();

        if !self.json {
            println!("\n{}", "═".repeat(60));
            println!("  KeyRx CI Check");
            println!("{}", "═".repeat(60));
        }

        // Phase 1: Unit tests
        if !self.skip_unit {
            let result = run_unit_tests(self.json);
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Phase 2: Integration tests
        if !self.skip_integration {
            let result = run_integration_tests(self.json);
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Phase 3: UAT tests
        if !self.skip_uat {
            let (result, uat_results) = run_uat_tests(self.json);
            self.output_phase_result(&result);
            summary.add_phase(result);

            // Evaluate quality gate if specified
            if let Some(ref gate_name) = self.gate {
                if let Some(ref results) = uat_results {
                    let gate_result = self.evaluate_gate(gate_name, results)?;
                    summary.set_gate_result(gate_result);
                }
            }
        }

        // Phase 4: Regression tests
        if !self.skip_regression {
            let result = run_regression_tests(self.json);
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Phase 5: Performance tests
        if !self.skip_perf {
            let result = run_performance_tests(self.json);
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Output final summary
        if self.json {
            self.output.data(&summary)?;
        } else {
            output_human_summary(&summary);
        }

        Ok(summary.exit_code)
    }

    /// Evaluate quality gate against UAT results.
    fn evaluate_gate(
        &self,
        gate_name: &str,
        results: &UatResults,
    ) -> Result<crate::uat::GateResult> {
        let enforcer = QualityGateEnforcer::new();
        let gate = enforcer
            .load(Some(gate_name))
            .context(format!("Failed to load quality gate '{}'", gate_name))?;
        Ok(enforcer.evaluate(&gate, results))
    }

    /// Output a single phase result (human-readable).
    fn output_phase_result(&self, result: &CheckPhaseResult) {
        if self.json {
            return;
        }
        output_phase_result(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_check_command_new() {
        let cmd = CiCheckCommand::new(OutputFormat::Human);
        assert!(cmd.gate.is_none());
        assert!(!cmd.json);
        assert!(!cmd.skip_unit);
        assert!(!cmd.skip_integration);
        assert!(!cmd.skip_uat);
        assert!(!cmd.skip_regression);
        assert!(!cmd.skip_perf);
    }

    #[test]
    fn ci_check_command_builder_methods() {
        let cmd = CiCheckCommand::new(OutputFormat::Human)
            .with_gate(Some("beta".to_string()))
            .with_json(true)
            .with_skip_unit(true)
            .with_skip_integration(true)
            .with_skip_uat(true)
            .with_skip_regression(true)
            .with_skip_perf(true);

        assert_eq!(cmd.gate, Some("beta".to_string()));
        assert!(cmd.json);
        assert!(cmd.skip_unit);
        assert!(cmd.skip_integration);
        assert!(cmd.skip_uat);
        assert!(cmd.skip_regression);
        assert!(cmd.skip_perf);
    }
}
