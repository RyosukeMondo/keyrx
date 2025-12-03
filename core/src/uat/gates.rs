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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
}
