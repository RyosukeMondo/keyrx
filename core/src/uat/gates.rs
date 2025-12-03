//! Quality gate configuration and enforcement.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Quality gate configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    /// Minimum pass rate (0.0-1.0).
    #[serde(default = "default_pass_rate")]
    pub pass_rate: f64,
    /// Maximum open P0 issues.
    #[serde(default)]
    pub p0_open: usize,
    /// Maximum open P1 issues.
    #[serde(default = "default_p1_open")]
    pub p1_open: usize,
    /// Maximum latency in microseconds.
    #[serde(default = "default_max_latency")]
    pub max_latency_us: u64,
    /// Minimum coverage percentage (0.0-1.0).
    #[serde(default = "default_coverage")]
    pub coverage_min: f64,
}

fn default_pass_rate() -> f64 {
    0.95
}

fn default_p1_open() -> usize {
    2
}

fn default_max_latency() -> u64 {
    1000
}

fn default_coverage() -> f64 {
    0.80
}

impl Default for QualityGate {
    fn default() -> Self {
        Self {
            pass_rate: default_pass_rate(),
            p0_open: 0,
            p1_open: default_p1_open(),
            max_latency_us: default_max_latency(),
            coverage_min: default_coverage(),
        }
    }
}

impl QualityGate {
    /// Create a new quality gate with all thresholds specified.
    pub fn new(
        pass_rate: f64,
        p0_open: usize,
        p1_open: usize,
        max_latency_us: u64,
        coverage_min: f64,
    ) -> Self {
        Self {
            pass_rate,
            p0_open,
            p1_open,
            max_latency_us,
            coverage_min,
        }
    }

    /// Create an alpha (relaxed) quality gate.
    pub fn alpha() -> Self {
        Self {
            pass_rate: 0.80,
            p0_open: 0,
            p1_open: 5,
            max_latency_us: 2000,
            coverage_min: 0.60,
        }
    }

    /// Create a beta quality gate.
    pub fn beta() -> Self {
        Self {
            pass_rate: 0.90,
            p0_open: 0,
            p1_open: 2,
            max_latency_us: 1000,
            coverage_min: 0.75,
        }
    }

    /// Create an RC (release candidate) quality gate.
    pub fn rc() -> Self {
        Self {
            pass_rate: 0.98,
            p0_open: 0,
            p1_open: 0,
            max_latency_us: 500,
            coverage_min: 0.85,
        }
    }

    /// Create a GA (general availability) quality gate.
    pub fn ga() -> Self {
        Self {
            pass_rate: 1.0,
            p0_open: 0,
            p1_open: 0,
            max_latency_us: 500,
            coverage_min: 0.90,
        }
    }
}

/// Result of quality gate evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct GateResult {
    /// Whether the gate passed.
    pub passed: bool,
    /// List of violations.
    pub violations: Vec<GateViolation>,
}

impl GateResult {
    /// Create a passing gate result.
    pub fn pass() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
        }
    }

    /// Create a failing gate result with the given violations.
    pub fn fail(violations: Vec<GateViolation>) -> Self {
        Self {
            passed: false,
            violations,
        }
    }
}

/// A quality gate violation.
#[derive(Debug, Clone, Serialize)]
pub struct GateViolation {
    /// Name of the violated criterion.
    pub criterion: String,
    /// Expected value.
    pub expected: String,
    /// Actual value.
    pub actual: String,
}

impl GateViolation {
    /// Create a new gate violation.
    pub fn new(
        criterion: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            criterion: criterion.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

/// Configuration file format for quality gates.
#[derive(Debug, Deserialize)]
struct QualityGatesConfig {
    #[serde(flatten)]
    gates: HashMap<String, QualityGate>,
}

/// Enforcer for quality gate configuration loading.
#[derive(Debug)]
pub struct QualityGateEnforcer {
    /// Path to the quality gates configuration file.
    config_path: PathBuf,
}

impl QualityGateEnforcer {
    /// Create a new enforcer with the default config path (`.keyrx/quality-gates.toml`).
    pub fn new() -> Self {
        Self {
            config_path: PathBuf::from(".keyrx/quality-gates.toml"),
        }
    }

    /// Create a new enforcer with a custom config path.
    pub fn with_config_path(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    /// Load a quality gate by name from the configuration file.
    ///
    /// If no name is provided, loads the "default" gate.
    /// If the config file doesn't exist, returns built-in defaults.
    ///
    /// # Arguments
    /// * `gate_name` - Optional gate name (e.g., "alpha", "beta", "rc", "ga")
    ///
    /// # Returns
    /// The quality gate configuration.
    pub fn load(&self, gate_name: Option<&str>) -> Result<QualityGate, GateLoadError> {
        let name = gate_name.unwrap_or("default");

        // Try to load from config file
        if self.config_path.exists() {
            return self.load_from_file(name);
        }

        // Fall back to built-in gates
        tracing::debug!(
            service = "keyrx",
            event = "gate_config_not_found",
            component = "quality_gate",
            path = %self.config_path.display(),
            "Quality gate config not found, using built-in defaults"
        );

        self.get_builtin_gate(name)
    }

    /// Load a gate from the configuration file.
    fn load_from_file(&self, name: &str) -> Result<QualityGate, GateLoadError> {
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| GateLoadError::FileRead(self.config_path.clone(), e.to_string()))?;

        let config: QualityGatesConfig =
            toml::from_str(&content).map_err(|e| GateLoadError::ParseError(e.to_string()))?;

        config
            .gates
            .get(name)
            .cloned()
            .ok_or_else(|| GateLoadError::GateNotFound(name.to_string()))
    }

    /// Get a built-in gate by name.
    fn get_builtin_gate(&self, name: &str) -> Result<QualityGate, GateLoadError> {
        match name {
            "default" => Ok(QualityGate::default()),
            "alpha" => Ok(QualityGate::alpha()),
            "beta" => Ok(QualityGate::beta()),
            "rc" => Ok(QualityGate::rc()),
            "ga" => Ok(QualityGate::ga()),
            _ => Err(GateLoadError::GateNotFound(name.to_string())),
        }
    }

    /// List all available gate names from the config file and built-ins.
    pub fn list_gates(&self) -> Vec<String> {
        let mut gates = vec![
            "default".to_string(),
            "alpha".to_string(),
            "beta".to_string(),
            "rc".to_string(),
            "ga".to_string(),
        ];

        // Add gates from config file
        if self.config_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_path) {
                if let Ok(config) = toml::from_str::<QualityGatesConfig>(&content) {
                    for name in config.gates.keys() {
                        if !gates.contains(name) {
                            gates.push(name.clone());
                        }
                    }
                }
            }
        }

        gates.sort();
        gates
    }
}

impl Default for QualityGateEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluation context containing additional metrics for gate evaluation.
#[derive(Debug, Default, Clone)]
pub struct EvaluationContext {
    /// Coverage percentage (0.0-1.0). If None, coverage check is skipped.
    pub coverage: Option<f64>,
    /// Maximum observed latency in microseconds.
    pub max_latency_us: Option<u64>,
}

impl QualityGateEnforcer {
    /// Evaluate UAT results against a quality gate.
    ///
    /// Checks all gate criteria and returns a result with any violations.
    ///
    /// # Arguments
    /// * `gate` - Quality gate configuration to evaluate against
    /// * `results` - UAT test results
    ///
    /// # Returns
    /// A `GateResult` indicating pass/fail and listing any violations.
    pub fn evaluate(
        &self,
        gate: &QualityGate,
        results: &crate::uat::runner::UatResults,
    ) -> GateResult {
        self.evaluate_with_context(gate, results, &EvaluationContext::default())
    }

    /// Evaluate UAT results against a quality gate with additional context.
    ///
    /// # Arguments
    /// * `gate` - Quality gate configuration to evaluate against
    /// * `results` - UAT test results
    /// * `context` - Additional evaluation context (coverage, latency metrics)
    ///
    /// # Returns
    /// A `GateResult` indicating pass/fail and listing any violations.
    pub fn evaluate_with_context(
        &self,
        gate: &QualityGate,
        results: &crate::uat::runner::UatResults,
        context: &EvaluationContext,
    ) -> GateResult {
        use crate::uat::runner::Priority;

        let mut violations = Vec::new();

        // Check pass rate
        let pass_rate = if results.total > 0 {
            results.passed as f64 / results.total as f64
        } else {
            1.0 // No tests means 100% pass rate
        };

        if pass_rate < gate.pass_rate {
            violations.push(GateViolation::new(
                "pass_rate",
                format!("≥{:.1}%", gate.pass_rate * 100.0),
                format!("{:.1}%", pass_rate * 100.0),
            ));
        }

        // Count P0 failures
        let p0_failures = results
            .results
            .iter()
            .filter(|r| !r.passed && r.test.priority == Priority::P0)
            .count();

        if p0_failures > gate.p0_open {
            violations.push(GateViolation::new(
                "p0_open",
                format!("≤{}", gate.p0_open),
                p0_failures.to_string(),
            ));
        }

        // Count P1 failures
        let p1_failures = results
            .results
            .iter()
            .filter(|r| !r.passed && r.test.priority == Priority::P1)
            .count();

        if p1_failures > gate.p1_open {
            violations.push(GateViolation::new(
                "p1_open",
                format!("≤{}", gate.p1_open),
                p1_failures.to_string(),
            ));
        }

        // Check max latency from results
        let max_latency_from_results = results.results.iter().map(|r| r.duration_us).max();
        let max_latency = context
            .max_latency_us
            .or(max_latency_from_results)
            .unwrap_or(0);

        if max_latency > gate.max_latency_us {
            violations.push(GateViolation::new(
                "max_latency_us",
                format!("≤{}µs", gate.max_latency_us),
                format!("{}µs", max_latency),
            ));
        }

        // Check coverage (only if provided in context)
        if let Some(coverage) = context.coverage {
            if coverage < gate.coverage_min {
                violations.push(GateViolation::new(
                    "coverage_min",
                    format!("≥{:.1}%", gate.coverage_min * 100.0),
                    format!("{:.1}%", coverage * 100.0),
                ));
            }
        }

        if violations.is_empty() {
            tracing::info!(
                service = "keyrx",
                event = "gate_evaluation_passed",
                component = "quality_gate",
                pass_rate = %format!("{:.1}%", pass_rate * 100.0),
                p0_failures = p0_failures,
                p1_failures = p1_failures,
                "Quality gate passed"
            );
            GateResult::pass()
        } else {
            tracing::warn!(
                service = "keyrx",
                event = "gate_evaluation_failed",
                component = "quality_gate",
                violation_count = violations.len(),
                "Quality gate failed with {} violation(s)",
                violations.len()
            );
            GateResult::fail(violations)
        }
    }
}

/// Error loading a quality gate.
#[derive(Debug)]
pub enum GateLoadError {
    /// Failed to read config file.
    FileRead(PathBuf, String),
    /// Failed to parse config file.
    ParseError(String),
    /// Gate not found.
    GateNotFound(String),
}

impl std::fmt::Display for GateLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileRead(path, err) => {
                write!(
                    f,
                    "Failed to read config file '{}': {}",
                    path.display(),
                    err
                )
            }
            Self::ParseError(err) => write!(f, "Failed to parse config file: {}", err),
            Self::GateNotFound(name) => write!(
                f,
                "Gate '{}' not found. Available: default, alpha, beta, rc, ga",
                name
            ),
        }
    }
}

impl std::error::Error for GateLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn quality_gate_default_values() {
        let gate = QualityGate::default();
        assert!((gate.pass_rate - 0.95).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 2);
        assert_eq!(gate.max_latency_us, 1000);
        assert!((gate.coverage_min - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_alpha_preset() {
        let gate = QualityGate::alpha();
        assert!((gate.pass_rate - 0.80).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 5);
        assert_eq!(gate.max_latency_us, 2000);
        assert!((gate.coverage_min - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_beta_preset() {
        let gate = QualityGate::beta();
        assert!((gate.pass_rate - 0.90).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 2);
        assert_eq!(gate.max_latency_us, 1000);
        assert!((gate.coverage_min - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_rc_preset() {
        let gate = QualityGate::rc();
        assert!((gate.pass_rate - 0.98).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 0);
        assert_eq!(gate.max_latency_us, 500);
        assert!((gate.coverage_min - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_ga_preset() {
        let gate = QualityGate::ga();
        assert!((gate.pass_rate - 1.0).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 0);
        assert_eq!(gate.max_latency_us, 500);
        assert!((gate.coverage_min - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_new_custom() {
        let gate = QualityGate::new(0.99, 1, 3, 750, 0.85);
        assert!((gate.pass_rate - 0.99).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 1);
        assert_eq!(gate.p1_open, 3);
        assert_eq!(gate.max_latency_us, 750);
        assert!((gate.coverage_min - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn gate_result_pass() {
        let result = GateResult::pass();
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn gate_result_fail() {
        let violations = vec![
            GateViolation::new("pass_rate", "95%", "90%"),
            GateViolation::new("p0_open", "0", "2"),
        ];
        let result = GateResult::fail(violations);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn gate_violation_new() {
        let violation = GateViolation::new("pass_rate", "≥95%", "90%");
        assert_eq!(violation.criterion, "pass_rate");
        assert_eq!(violation.expected, "≥95%");
        assert_eq!(violation.actual, "90%");
    }

    #[test]
    fn enforcer_default_path() {
        let enforcer = QualityGateEnforcer::new();
        assert_eq!(
            enforcer.config_path,
            PathBuf::from(".keyrx/quality-gates.toml")
        );
    }

    #[test]
    fn enforcer_custom_path() {
        let enforcer = QualityGateEnforcer::with_config_path("/custom/path.toml");
        assert_eq!(enforcer.config_path, PathBuf::from("/custom/path.toml"));
    }

    #[test]
    fn enforcer_load_builtin_default() {
        let enforcer = QualityGateEnforcer::with_config_path("/nonexistent/path.toml");
        let gate = enforcer.load(None).unwrap();
        assert!((gate.pass_rate - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn enforcer_load_builtin_alpha() {
        let enforcer = QualityGateEnforcer::with_config_path("/nonexistent/path.toml");
        let gate = enforcer.load(Some("alpha")).unwrap();
        assert!((gate.pass_rate - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn enforcer_load_builtin_beta() {
        let enforcer = QualityGateEnforcer::with_config_path("/nonexistent/path.toml");
        let gate = enforcer.load(Some("beta")).unwrap();
        assert!((gate.pass_rate - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn enforcer_load_builtin_rc() {
        let enforcer = QualityGateEnforcer::with_config_path("/nonexistent/path.toml");
        let gate = enforcer.load(Some("rc")).unwrap();
        assert!((gate.pass_rate - 0.98).abs() < f64::EPSILON);
    }

    #[test]
    fn enforcer_load_builtin_ga() {
        let enforcer = QualityGateEnforcer::with_config_path("/nonexistent/path.toml");
        let gate = enforcer.load(Some("ga")).unwrap();
        assert!((gate.pass_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn enforcer_load_unknown_gate_fails() {
        let enforcer = QualityGateEnforcer::with_config_path("/nonexistent/path.toml");
        let result = enforcer.load(Some("nonexistent"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, GateLoadError::GateNotFound(_)));
    }

    #[test]
    fn enforcer_load_from_toml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("quality-gates.toml");

        let config = r#"
[custom]
pass_rate = 0.92
p0_open = 0
p1_open = 3
max_latency_us = 800
coverage_min = 0.70

[strict]
pass_rate = 0.99
p0_open = 0
p1_open = 0
max_latency_us = 250
coverage_min = 0.95
"#;
        fs::write(&config_path, config).unwrap();

        let enforcer = QualityGateEnforcer::with_config_path(&config_path);

        // Load custom gate
        let custom = enforcer.load(Some("custom")).unwrap();
        assert!((custom.pass_rate - 0.92).abs() < f64::EPSILON);
        assert_eq!(custom.p1_open, 3);
        assert_eq!(custom.max_latency_us, 800);
        assert!((custom.coverage_min - 0.70).abs() < f64::EPSILON);

        // Load strict gate
        let strict = enforcer.load(Some("strict")).unwrap();
        assert!((strict.pass_rate - 0.99).abs() < f64::EPSILON);
        assert_eq!(strict.p1_open, 0);
        assert_eq!(strict.max_latency_us, 250);
        assert!((strict.coverage_min - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn enforcer_load_with_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("quality-gates.toml");

        // Partial config - should use defaults for missing fields
        let config = r#"
[minimal]
pass_rate = 0.85
"#;
        fs::write(&config_path, config).unwrap();

        let enforcer = QualityGateEnforcer::with_config_path(&config_path);
        let gate = enforcer.load(Some("minimal")).unwrap();

        assert!((gate.pass_rate - 0.85).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0); // default
        assert_eq!(gate.p1_open, 2); // default
        assert_eq!(gate.max_latency_us, 1000); // default
        assert!((gate.coverage_min - 0.80).abs() < f64::EPSILON); // default
    }

    #[test]
    fn enforcer_list_gates_builtin() {
        let enforcer = QualityGateEnforcer::with_config_path("/nonexistent/path.toml");
        let gates = enforcer.list_gates();

        assert!(gates.contains(&"default".to_string()));
        assert!(gates.contains(&"alpha".to_string()));
        assert!(gates.contains(&"beta".to_string()));
        assert!(gates.contains(&"rc".to_string()));
        assert!(gates.contains(&"ga".to_string()));
    }

    #[test]
    fn enforcer_list_gates_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("quality-gates.toml");

        let config = r#"
[custom]
pass_rate = 0.90

[special]
pass_rate = 0.85
"#;
        fs::write(&config_path, config).unwrap();

        let enforcer = QualityGateEnforcer::with_config_path(&config_path);
        let gates = enforcer.list_gates();

        // Should include both built-in and custom gates
        assert!(gates.contains(&"default".to_string()));
        assert!(gates.contains(&"custom".to_string()));
        assert!(gates.contains(&"special".to_string()));
    }

    #[test]
    fn gate_load_error_display() {
        let err = GateLoadError::FileRead(PathBuf::from("/test.toml"), "not found".to_string());
        assert!(err.to_string().contains("/test.toml"));
        assert!(err.to_string().contains("not found"));

        let err = GateLoadError::ParseError("invalid TOML".to_string());
        assert!(err.to_string().contains("invalid TOML"));

        let err = GateLoadError::GateNotFound("unknown".to_string());
        assert!(err.to_string().contains("unknown"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn toml_parse_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("quality-gates.toml");

        // Invalid TOML
        fs::write(&config_path, "this is not valid [toml").unwrap();

        let enforcer = QualityGateEnforcer::with_config_path(&config_path);
        let result = enforcer.load(Some("any"));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GateLoadError::ParseError(_)));
    }

    // Helper function to create test results
    fn create_test_results(
        total: usize,
        passed: usize,
        p0_failures: usize,
        p1_failures: usize,
        max_duration_us: u64,
    ) -> crate::uat::runner::UatResults {
        use crate::uat::runner::{Priority, UatResult, UatResults, UatTest};

        let mut results = Vec::new();
        let mut remaining_passed = passed;
        let mut remaining_p0 = p0_failures;
        let mut remaining_p1 = p1_failures;

        for i in 0..total {
            let (priority, is_passed) = if remaining_p0 > 0 {
                remaining_p0 -= 1;
                (Priority::P0, false)
            } else if remaining_p1 > 0 {
                remaining_p1 -= 1;
                (Priority::P1, false)
            } else if remaining_passed > 0 {
                remaining_passed -= 1;
                (Priority::P2, true)
            } else {
                (Priority::P2, false)
            };

            results.push(UatResult {
                test: UatTest {
                    name: format!("test_{}", i),
                    file: "test.rhai".to_string(),
                    category: "default".to_string(),
                    priority,
                    requirements: vec![],
                    latency_threshold: None,
                },
                passed: is_passed,
                duration_us: if i == 0 { max_duration_us } else { 100 },
                error: if is_passed {
                    None
                } else {
                    Some("Test failed".to_string())
                },
            });
        }

        UatResults {
            total,
            passed,
            failed: total - passed,
            skipped: 0,
            duration_us: max_duration_us + (total as u64 - 1) * 100,
            results,
        }
    }

    #[test]
    fn evaluate_passes_when_all_criteria_met() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default(); // 95% pass rate, 0 P0, 2 P1, 1000µs, 80% coverage

        // 100 tests, 96 passed, 0 P0 failures, 2 P1 failures, 500µs max latency
        let results = create_test_results(100, 96, 0, 2, 500);

        let result = enforcer.evaluate(&gate, &results);

        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn evaluate_fails_on_low_pass_rate() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default(); // 95% pass rate

        // 100 tests, only 90 passed (90%)
        let results = create_test_results(100, 90, 0, 0, 500);

        let result = enforcer.evaluate(&gate, &results);

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].criterion, "pass_rate");
        assert!(result.violations[0].expected.contains("95.0%"));
        assert!(result.violations[0].actual.contains("90.0%"));
    }

    #[test]
    fn evaluate_fails_on_p0_failures() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default(); // 0 P0 open

        // 100 tests, 98 passed, 1 P0 failure
        let results = create_test_results(100, 98, 1, 0, 500);

        let result = enforcer.evaluate(&gate, &results);

        assert!(!result.passed);
        let p0_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "p0_open")
            .unwrap();
        assert_eq!(p0_violation.actual, "1");
    }

    #[test]
    fn evaluate_fails_on_too_many_p1_failures() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default(); // 2 P1 open max

        // 100 tests, 96 passed, 0 P0 failures, 3 P1 failures (exceeds limit)
        let results = create_test_results(100, 96, 0, 3, 500);

        let result = enforcer.evaluate(&gate, &results);

        assert!(!result.passed);
        let p1_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "p1_open")
            .unwrap();
        assert_eq!(p1_violation.actual, "3");
    }

    #[test]
    fn evaluate_fails_on_high_latency() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default(); // 1000µs max

        // 100 tests, all pass, but max latency is 1500µs
        let results = create_test_results(100, 100, 0, 0, 1500);

        let result = enforcer.evaluate(&gate, &results);

        assert!(!result.passed);
        let latency_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "max_latency_us")
            .unwrap();
        assert!(latency_violation.actual.contains("1500"));
    }

    #[test]
    fn evaluate_with_context_checks_coverage() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default(); // 80% coverage min

        let results = create_test_results(100, 100, 0, 0, 500);
        let context = EvaluationContext {
            coverage: Some(0.70), // Only 70% coverage
            max_latency_us: None,
        };

        let result = enforcer.evaluate_with_context(&gate, &results, &context);

        assert!(!result.passed);
        let coverage_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "coverage_min")
            .unwrap();
        assert!(coverage_violation.expected.contains("80.0%"));
        assert!(coverage_violation.actual.contains("70.0%"));
    }

    #[test]
    fn evaluate_skips_coverage_when_not_provided() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default();

        let results = create_test_results(100, 100, 0, 0, 500);
        // No coverage in context
        let context = EvaluationContext::default();

        let result = enforcer.evaluate_with_context(&gate, &results, &context);

        assert!(result.passed);
        assert!(!result
            .violations
            .iter()
            .any(|v| v.criterion == "coverage_min"));
    }

    #[test]
    fn evaluate_with_context_uses_provided_latency() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default(); // 1000µs max

        let results = create_test_results(100, 100, 0, 0, 500); // Results show 500µs
        let context = EvaluationContext {
            coverage: None,
            max_latency_us: Some(2000), // But context says 2000µs
        };

        let result = enforcer.evaluate_with_context(&gate, &results, &context);

        assert!(!result.passed);
        let latency_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "max_latency_us")
            .unwrap();
        assert!(latency_violation.actual.contains("2000"));
    }

    #[test]
    fn evaluate_passes_with_no_tests() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::default();

        let results = crate::uat::runner::UatResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_us: 0,
            results: vec![],
        };

        let result = enforcer.evaluate(&gate, &results);

        // No tests means 100% pass rate, 0 failures
        assert!(result.passed);
    }

    #[test]
    fn evaluate_collects_multiple_violations() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::ga(); // Strictest: 100% pass, 0 P0/P1, 500µs, 90% coverage

        // Many violations: low pass rate, P0 failure, P1 failure, high latency
        let results = create_test_results(100, 80, 2, 3, 1000);
        let context = EvaluationContext {
            coverage: Some(0.50),
            max_latency_us: None,
        };

        let result = enforcer.evaluate_with_context(&gate, &results, &context);

        assert!(!result.passed);
        // Should have violations for: pass_rate, p0_open, p1_open, max_latency_us, coverage_min
        assert!(result.violations.len() >= 4); // At least these should fail
        assert!(result.violations.iter().any(|v| v.criterion == "pass_rate"));
        assert!(result.violations.iter().any(|v| v.criterion == "p0_open"));
        assert!(result.violations.iter().any(|v| v.criterion == "p1_open"));
        assert!(result
            .violations
            .iter()
            .any(|v| v.criterion == "coverage_min"));
    }

    #[test]
    fn evaluate_with_alpha_gate_is_more_lenient() {
        let enforcer = QualityGateEnforcer::new();
        let gate = QualityGate::alpha(); // 80% pass rate, 5 P1 open, 2000µs

        // Would fail default gate but passes alpha
        let results = create_test_results(100, 85, 0, 4, 1500);

        let result = enforcer.evaluate(&gate, &results);

        assert!(result.passed);
    }
}
