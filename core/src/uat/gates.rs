//! Quality gate configuration and enforcement.
//!
//! This module coordinates quality gate functionality by re-exporting types
//! from `gates_definitions` and evaluation logic from `gates_evaluation`.

use std::fs;
use std::path::PathBuf;

// Re-export all public types
use super::gates_definitions::QualityGatesConfig;
pub use super::gates_definitions::{
    EvaluationContext, GateLoadError, GateResult, GateViolation, QualityGate,
};
use super::gates_evaluation;

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
        gates_evaluation::evaluate(gate, results)
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
        gates_evaluation::evaluate_with_context(gate, results, context)
    }
}

impl Default for QualityGateEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
