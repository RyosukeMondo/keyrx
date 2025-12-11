//! Validation engine orchestrator.
//!
//! Orchestrates all validation passes and aggregates results.
//! This is the main entry point for script validation.
//!
//! # Submodules
//!
//! - **context**: Script context and operation location types
//! - **rhai_engine**: Rhai engine configuration for validation

mod context;
mod rhai_engine;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::scripting::PendingOp;

use super::config::ValidationConfig;
use super::coverage::{analyze_coverage, render_ascii_keyboard};
use super::detectors::conflicts::ConflictDetector;
use super::detectors::cycles::CycleDetector;
use super::detectors::shadowing::ShadowingDetector;
use super::detectors::DetectorContext;
use super::orchestrator::DetectorOrchestrator;
use super::safety::analyze_safety;
use super::semantic::SemanticValidator;
use super::types::{ValidationError, ValidationOptions, ValidationResult, ValidationWarning};

use context::populate_context_from_ops;
pub use context::{
    collect_definitions, find_operation_line, LocatedOp, ParsedScript, ScriptContext,
};
use rhai_engine::{create_validation_engine, parse_error_to_validation_error, PendingOps};

/// Validation engine that orchestrates all validation passes.
///
/// Parses scripts, collects pending operations, and runs semantic,
/// conflict, safety, and coverage validation.
pub struct ValidationEngine {
    config: ValidationConfig,
    orchestrator: DetectorOrchestrator,
}

impl ValidationEngine {
    /// Create a new validation engine with default config.
    pub fn new() -> Self {
        Self::with_config(ValidationConfig::load())
    }

    /// Create a validation engine with a specific config.
    pub fn with_config(config: ValidationConfig) -> Self {
        let mut orchestrator = DetectorOrchestrator::new();

        // Register all detectors in order
        orchestrator.register(Box::new(ConflictDetector::new()));
        orchestrator.register(Box::new(ShadowingDetector::new()));
        orchestrator.register(Box::new(CycleDetector));

        Self {
            config,
            orchestrator,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Validate a script and return the result.
    ///
    /// This is the main validation entry point. It:
    /// 1. Parses the script with Rhai
    /// 2. Collects pending operations
    /// 3. Runs all validators
    /// 4. Aggregates and returns results
    pub fn validate(&self, script: &str, options: ValidationOptions) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Step 1: Parse script and collect operations
        let ops = match self.parse_script(script) {
            Ok(ops) => ops,
            Err(parse_error) => {
                result.add_error(parse_error);
                return result;
            }
        };

        // Step 2: Collect layer and modifier definitions
        let (layers, modifiers) = collect_definitions(&ops);

        // Step 3: Run semantic validation
        let semantic_errors = self.run_semantic_validation(&ops, &layers, &modifiers);
        for error in semantic_errors {
            if result.errors.len() >= self.config.max_errors {
                break;
            }
            result.add_error(error);
        }

        // Step 4: Run timing validation (produces warnings)
        if !options.no_warnings {
            let timing_warnings = self.run_timing_validation(&ops);
            for warning in timing_warnings {
                result.add_warning(warning);
            }
        }

        // Step 5: Run conflict detection (produces warnings)
        if !options.no_warnings {
            let conflict_warnings = self.run_conflict_detection(&ops);
            for warning in conflict_warnings {
                result.add_warning(warning);
            }
        }

        // Step 6: Run safety analysis (produces warnings)
        if !options.no_warnings {
            let safety_warnings = analyze_safety(&ops, &self.config);
            for warning in safety_warnings {
                result.add_warning(warning);
            }
        }

        // Step 7: Generate coverage report if requested
        if options.include_coverage || options.include_visual {
            let coverage = analyze_coverage(&ops);
            result = result.with_coverage(coverage);
        }

        // Step 8: In strict mode, treat warnings as errors
        if options.strict && result.has_warnings() {
            result.is_valid = false;
        }

        result
    }

    /// Validate a script and return the ASCII keyboard visualization.
    pub fn validate_with_visual(&self, script: &str) -> (ValidationResult, Option<String>) {
        let options = ValidationOptions::new().with_coverage().with_visual();
        let result = self.validate(script, options);

        let visual = result.coverage.as_ref().map(render_ascii_keyboard);

        (result, visual)
    }

    /// Parse a script and collect pending operations.
    fn parse_script(&self, script: &str) -> Result<Vec<PendingOp>, ValidationError> {
        let parsed = self.parse_script_with_context(script)?;
        Ok(parsed.ops)
    }

    /// Parse a script and return operations with full context.
    ///
    /// This method extracts layer and modifier definitions during parsing
    /// and provides access to the script context for source location tracking.
    pub fn parse_script_with_context(&self, script: &str) -> Result<ParsedScript, ValidationError> {
        let pending_ops: PendingOps = Arc::new(Mutex::new(Vec::new()));
        let engine = create_validation_engine(&pending_ops);

        // Run the script to collect operations
        if let Err(err) = engine.run(script) {
            return Err(parse_error_to_validation_error(err));
        }

        // Extract collected operations
        let ops = pending_ops.lock().map_err(|_| {
            ValidationError::new("E000", "Internal error: failed to lock pending operations")
        })?;

        // Build context from script and operations
        let mut context = ScriptContext::from_script(script);
        populate_context_from_ops(&ops, &mut context);

        Ok(ParsedScript {
            ops: ops.clone(),
            context,
        })
    }

    /// Run semantic validation on operations.
    fn run_semantic_validation(
        &self,
        ops: &[PendingOp],
        layers: &HashSet<String>,
        modifiers: &HashSet<String>,
    ) -> Vec<ValidationError> {
        let validator = SemanticValidator::new(&self.config, layers.clone(), modifiers.clone());
        validator.validate_operations(ops)
    }

    /// Run timing validation on operations.
    fn run_timing_validation(&self, ops: &[PendingOp]) -> Vec<ValidationWarning> {
        let validator = SemanticValidator::new(&self.config, HashSet::new(), HashSet::new());
        validator.validate_timing(ops)
    }

    /// Run conflict detection on operations.
    fn run_conflict_detection(&self, ops: &[PendingOp]) -> Vec<ValidationWarning> {
        // Create detector context
        let ctx = DetectorContext::new(self.config.clone());

        // Run all detectors through the orchestrator
        let report = self.orchestrator.run(ops, &ctx);

        // Convert ValidationIssues to ValidationWarnings for backward compatibility
        report
            .issues
            .iter()
            .map(|issue| issue.to_warning())
            .collect()
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

    #[test]
    fn validates_valid_script() {
        let engine = ValidationEngine::new();
        let script = r#"
            remap("CapsLock", "Escape");
            remap("A", "B");
        "#;
        let result = engine.validate(script, ValidationOptions::new());
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn detects_parse_error() {
        let engine = ValidationEngine::new();
        let script = "this is not valid rhai {{{{";
        let result = engine.validate(script, ValidationOptions::new());
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].code, "E000");
    }

    #[test]
    fn detects_undefined_layer() {
        let engine = ValidationEngine::new();
        let script = r#"layer_push("undefined_layer");"#;
        let result = engine.validate(script, ValidationOptions::new());
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "E002"));
    }

    #[test]
    fn accepts_defined_layer() {
        let engine = ValidationEngine::new();
        let script = r#"
            define_layer("nav");
            layer_push("nav");
        "#;
        let result = engine.validate(script, ValidationOptions::new());
        assert!(result.is_valid);
    }

    #[test]
    fn detects_undefined_modifier() {
        let engine = ValidationEngine::new();
        let script = r#"modifier_activate("hyper");"#;
        let result = engine.validate(script, ValidationOptions::new());
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "E003"));
    }

    #[test]
    fn accepts_defined_modifier() {
        let engine = ValidationEngine::new();
        let script = r#"
            define_modifier("hyper");
            modifier_activate("hyper");
        "#;
        let result = engine.validate(script, ValidationOptions::new());
        assert!(result.is_valid);
    }

    #[test]
    fn generates_conflict_warnings() {
        let engine = ValidationEngine::new();
        let script = r#"
            remap("A", "B");
            remap("A", "C");
        "#;
        let result = engine.validate(script, ValidationOptions::new());
        assert!(result.is_valid); // Warnings don't make invalid
        assert!(result.has_warnings());
    }

    #[test]
    fn strict_mode_treats_warnings_as_errors() {
        let engine = ValidationEngine::new();
        let script = r#"
            remap("A", "B");
            remap("A", "C");
        "#;
        let result = engine.validate(script, ValidationOptions::new().strict());
        assert!(!result.is_valid);
    }

    #[test]
    fn no_warnings_suppresses_warnings() {
        let engine = ValidationEngine::new();
        let script = r#"
            remap("A", "B");
            remap("A", "C");
        "#;
        let result = engine.validate(script, ValidationOptions::new().no_warnings());
        assert!(result.is_valid);
        assert!(!result.has_warnings());
    }

    #[test]
    fn includes_coverage_when_requested() {
        let engine = ValidationEngine::new();
        let script = r#"
            remap("CapsLock", "Escape");
            block("Insert");
        "#;
        let result = engine.validate(script, ValidationOptions::new().with_coverage());
        assert!(result.coverage.is_some());
        let coverage = result.coverage.unwrap();
        assert!(coverage.remapped.iter().any(|k| k.name() == "CapsLock"));
        assert!(coverage.blocked.iter().any(|k| k.name() == "Insert"));
    }

    #[test]
    fn validate_with_visual_returns_visualization() {
        let engine = ValidationEngine::new();
        let script = r#"remap("A", "B");"#;
        let (result, visual) = engine.validate_with_visual(script);
        assert!(result.is_valid);
        assert!(visual.is_some());
        assert!(visual.unwrap().contains("Legend:"));
    }

    #[test]
    fn respects_max_errors_config() {
        let mut config = ValidationConfig::default();
        config.max_errors = 1;
        let engine = ValidationEngine::with_config(config);

        let script = r#"
            layer_push("layer1");
            layer_push("layer2");
            layer_push("layer3");
        "#;
        let result = engine.validate(script, ValidationOptions::new());
        // Should only have 1 error due to max_errors limit
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn uses_custom_config() {
        let mut config = ValidationConfig::default();
        config.tap_timeout_warn_range = (100, 200);
        let engine = ValidationEngine::with_config(config);

        // 150ms is within our custom range, should not warn
        let script = "tap_timeout(150);";
        let result = engine.validate(script, ValidationOptions::new());
        assert!(!result.has_warnings());

        // 50ms is below our custom min, should warn
        let script = "tap_timeout(50);";
        let result = engine.validate(script, ValidationOptions::new());
        assert!(result.has_warnings());
    }

    // Tests for operation collection and context

    #[test]
    fn parse_script_with_context_extracts_layers() {
        let engine = ValidationEngine::new();
        let script = r#"
            define_layer("nav");
            define_layer("symbols");
            layer_push("nav");
        "#;
        let parsed = engine.parse_script_with_context(script).unwrap();
        assert!(parsed.context.layers.contains("nav"));
        assert!(parsed.context.layers.contains("symbols"));
        assert_eq!(parsed.context.layers.len(), 2);
    }

    #[test]
    fn parse_script_with_context_extracts_modifiers() {
        let engine = ValidationEngine::new();
        let script = r#"
            define_modifier("hyper");
            define_modifier("meh");
            modifier_activate("hyper");
        "#;
        let parsed = engine.parse_script_with_context(script).unwrap();
        assert!(parsed.context.modifiers.contains("hyper"));
        assert!(parsed.context.modifiers.contains("meh"));
        assert_eq!(parsed.context.modifiers.len(), 2);
    }

    #[test]
    fn parsed_script_contains_ops_and_context() {
        let engine = ValidationEngine::new();
        let script = r#"
            define_layer("test");
            define_modifier("mod1");
            remap("A", "B");
        "#;
        let parsed = engine.parse_script_with_context(script).unwrap();

        // Should have 3 operations
        assert_eq!(parsed.ops.len(), 3);

        // Context should have collected definitions
        assert!(parsed.context.layers.contains("test"));
        assert!(parsed.context.modifiers.contains("mod1"));

        // Script lines should be available
        assert_eq!(parsed.context.lines.len(), 5); // includes empty lines
    }
}
