//! Validation engine orchestrator.
//!
//! Orchestrates all validation passes and aggregates results.
//! This is the main entry point for script validation.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use rhai::Engine;

use crate::scripting::{LayerMapAction, PendingOp, TimingUpdate};

use super::config::ValidationConfig;
use super::conflicts::{detect_circular_remaps, detect_combo_shadowing, detect_remap_conflicts};
use super::coverage::{analyze_coverage, render_ascii_keyboard};
use super::safety::analyze_safety;
use super::semantic::SemanticValidator;
use super::types::{
    SourceLocation, ValidationError, ValidationOptions, ValidationResult, ValidationWarning,
};

/// Operation with associated metadata for validation.
#[derive(Debug, Clone)]
pub struct LocatedOp {
    /// The operation itself.
    pub op: PendingOp,
    /// Operation index (order in script).
    pub index: usize,
}

impl LocatedOp {
    /// Create a new located operation.
    pub fn new(op: PendingOp, index: usize) -> Self {
        Self { op, index }
    }
}

/// Script context containing parsed metadata.
#[derive(Debug, Clone, Default)]
pub struct ScriptContext {
    /// Defined layer names.
    pub layers: HashSet<String>,
    /// Defined modifier names.
    pub modifiers: HashSet<String>,
    /// Script lines for source location context.
    pub lines: Vec<String>,
}

impl ScriptContext {
    /// Create a new script context from script source.
    pub fn from_script(script: &str) -> Self {
        Self {
            layers: HashSet::new(),
            modifiers: HashSet::new(),
            lines: script.lines().map(String::from).collect(),
        }
    }

    /// Get a line from the script (1-indexed).
    pub fn get_line(&self, line_num: usize) -> Option<&str> {
        if line_num > 0 && line_num <= self.lines.len() {
            Some(&self.lines[line_num - 1])
        } else {
            None
        }
    }

    /// Create a source location with context from this script.
    pub fn source_location(&self, line: usize, column: Option<usize>) -> SourceLocation {
        let mut loc = SourceLocation::new(line);
        if let Some(col) = column {
            loc = loc.with_column(col);
        }
        if let Some(context) = self.get_line(line) {
            loc = loc.with_context(context.trim());
        }
        loc
    }
}

/// Thread-safe pending operations storage for validation.
type PendingOps = Arc<Mutex<Vec<PendingOp>>>;

/// Validation engine that orchestrates all validation passes.
///
/// Parses scripts, collects pending operations, and runs semantic,
/// conflict, safety, and coverage validation.
pub struct ValidationEngine {
    config: ValidationConfig,
}

/// Result of parsing a script, containing operations and context.
#[derive(Debug, Clone)]
pub struct ParsedScript {
    /// Collected operations.
    pub ops: Vec<PendingOp>,
    /// Script context with definitions.
    pub context: ScriptContext,
}

impl ValidationEngine {
    /// Create a new validation engine with default config.
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::load(),
        }
    }

    /// Create a validation engine with a specific config.
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
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
        let mut warnings = Vec::new();
        warnings.extend(detect_remap_conflicts(ops));
        warnings.extend(detect_combo_shadowing(ops));
        warnings.extend(detect_circular_remaps(ops, &self.config));
        warnings
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Collect layer and modifier definitions from operations.
pub fn collect_definitions(ops: &[PendingOp]) -> (HashSet<String>, HashSet<String>) {
    let mut layers = HashSet::new();
    let mut modifiers = HashSet::new();

    for op in ops {
        match op {
            PendingOp::LayerDefine { name, .. } => {
                layers.insert(name.clone());
            }
            PendingOp::DefineModifier { name, .. } => {
                modifiers.insert(name.clone());
            }
            _ => {}
        }
    }

    (layers, modifiers)
}

/// Populate script context with layer and modifier definitions from operations.
fn populate_context_from_ops(ops: &[PendingOp], context: &mut ScriptContext) {
    for op in ops {
        match op {
            PendingOp::LayerDefine { name, .. } => {
                context.layers.insert(name.clone());
            }
            PendingOp::DefineModifier { name, .. } => {
                context.modifiers.insert(name.clone());
            }
            _ => {}
        }
    }
}

/// Find the approximate line number for an operation by searching for patterns.
///
/// This is a best-effort heuristic since Rhai doesn't provide position info
/// during function execution. Returns None if no match is found.
pub fn find_operation_line(script: &str, op: &PendingOp) -> Option<usize> {
    let pattern = match op {
        PendingOp::Remap { from, to } => {
            format!("remap(\"{}\", \"{}\")", from.name(), to.name())
        }
        PendingOp::Block { key } => {
            format!("block(\"{}\")", key.name())
        }
        PendingOp::Pass { key } => {
            format!("pass(\"{}\")", key.name())
        }
        PendingOp::TapHold { key, tap, .. } => {
            format!("tap_hold(\"{}\", \"{}\"", key.name(), tap.name())
        }
        PendingOp::LayerDefine { name, .. } => {
            format!("define_layer(\"{}\"", name)
        }
        PendingOp::LayerPush { name } => {
            format!("layer_push(\"{}\")", name)
        }
        PendingOp::LayerToggle { name } => {
            format!("layer_toggle(\"{}\")", name)
        }
        PendingOp::LayerMap { layer, key, .. } => {
            format!("layer_map(\"{}\", \"{}\"", layer, key.name())
        }
        PendingOp::DefineModifier { name, .. } => {
            format!("define_modifier(\"{}\")", name)
        }
        PendingOp::ModifierActivate { name, .. } => {
            format!("modifier_activate(\"{}\")", name)
        }
        PendingOp::ModifierDeactivate { name, .. } => {
            format!("modifier_deactivate(\"{}\")", name)
        }
        PendingOp::ModifierOneShot { name, .. } => {
            format!("modifier_one_shot(\"{}\")", name)
        }
        PendingOp::SetTiming(timing) => match timing {
            TimingUpdate::TapTimeout(ms) => format!("tap_timeout({})", ms),
            TimingUpdate::ComboTimeout(ms) => format!("combo_timeout({})", ms),
            TimingUpdate::HoldDelay(ms) => format!("hold_delay({})", ms),
            _ => return None,
        },
        PendingOp::Combo { .. } | PendingOp::LayerPop => return None,
    };

    for (idx, line) in script.lines().enumerate() {
        if line.contains(&pattern) {
            return Some(idx + 1); // 1-indexed
        }
    }
    None
}

/// Create a Rhai engine configured for validation (no actual registry).
fn create_validation_engine(pending_ops: &PendingOps) -> Engine {
    use crate::engine::{HoldAction, KeyCode, LayerAction};
    use crate::scripting::helpers::parse_key_or_error;
    use rhai::EvalAltResult;
    use std::sync::Arc;

    let mut engine = Engine::new();

    // Sandbox settings
    engine.set_max_expr_depths(64, 64);
    engine.set_max_operations(100_000);

    let ops = pending_ops.clone();

    // remap(from, to)
    let ops_remap = Arc::clone(&ops);
    engine.register_fn(
        "remap",
        move |from: &str, to: &str| -> Result<(), Box<EvalAltResult>> {
            let from_key = parse_key_or_error(from, "remap")?;
            let to_key = parse_key_or_error(to, "remap")?;
            if let Ok(mut guard) = ops_remap.lock() {
                guard.push(PendingOp::Remap {
                    from: from_key,
                    to: to_key,
                });
            }
            Ok(())
        },
    );

    // block(key)
    let ops_block = Arc::clone(&ops);
    engine.register_fn(
        "block",
        move |key: &str| -> Result<(), Box<EvalAltResult>> {
            let key_code = parse_key_or_error(key, "block")?;
            if let Ok(mut guard) = ops_block.lock() {
                guard.push(PendingOp::Block { key: key_code });
            }
            Ok(())
        },
    );

    // pass(key)
    let ops_pass = Arc::clone(&ops);
    engine.register_fn("pass", move |key: &str| -> Result<(), Box<EvalAltResult>> {
        let key_code = parse_key_or_error(key, "pass")?;
        if let Ok(mut guard) = ops_pass.lock() {
            guard.push(PendingOp::Pass { key: key_code });
        }
        Ok(())
    });

    // tap_hold(key, tap, hold)
    let ops_tap_hold = Arc::clone(&ops);
    engine.register_fn(
        "tap_hold",
        move |key: &str, tap: &str, hold: &str| -> Result<(), Box<EvalAltResult>> {
            let key_code = parse_key_or_error(key, "tap_hold")?;
            let tap_key = parse_key_or_error(tap, "tap_hold")?;
            let hold_key = parse_key_or_error(hold, "tap_hold")?;
            if let Ok(mut guard) = ops_tap_hold.lock() {
                guard.push(PendingOp::TapHold {
                    key: key_code,
                    tap: tap_key,
                    hold: HoldAction::Key(hold_key),
                });
            }
            Ok(())
        },
    );

    // define_layer(name)
    let ops_layer = Arc::clone(&ops);
    engine.register_fn("define_layer", move |name: &str| {
        if let Ok(mut guard) = ops_layer.lock() {
            guard.push(PendingOp::LayerDefine {
                name: name.to_string(),
                transparent: false,
            });
        }
    });

    // define_layer(name, transparent)
    let ops_layer_t = Arc::clone(&ops);
    engine.register_fn("define_layer", move |name: &str, transparent: bool| {
        if let Ok(mut guard) = ops_layer_t.lock() {
            guard.push(PendingOp::LayerDefine {
                name: name.to_string(),
                transparent,
            });
        }
    });

    // layer_push(name)
    let ops_push = Arc::clone(&ops);
    engine.register_fn("layer_push", move |name: &str| {
        if let Ok(mut guard) = ops_push.lock() {
            guard.push(PendingOp::LayerPush {
                name: name.to_string(),
            });
        }
    });

    // layer_toggle(name)
    let ops_toggle = Arc::clone(&ops);
    engine.register_fn("layer_toggle", move |name: &str| {
        if let Ok(mut guard) = ops_toggle.lock() {
            guard.push(PendingOp::LayerToggle {
                name: name.to_string(),
            });
        }
    });

    // layer_pop()
    let ops_pop = Arc::clone(&ops);
    engine.register_fn("layer_pop", move || {
        if let Ok(mut guard) = ops_pop.lock() {
            guard.push(PendingOp::LayerPop);
        }
    });

    // define_modifier(name)
    let ops_mod = Arc::clone(&ops);
    engine.register_fn("define_modifier", move |name: &str| {
        if let Ok(mut guard) = ops_mod.lock() {
            guard.push(PendingOp::DefineModifier {
                name: name.to_string(),
                id: 0, // ID assigned at runtime
            });
        }
    });

    // modifier_activate(name)
    let ops_act = Arc::clone(&ops);
    engine.register_fn("modifier_activate", move |name: &str| {
        if let Ok(mut guard) = ops_act.lock() {
            guard.push(PendingOp::ModifierActivate {
                name: name.to_string(),
                id: 0,
            });
        }
    });

    // modifier_deactivate(name)
    let ops_deact = Arc::clone(&ops);
    engine.register_fn("modifier_deactivate", move |name: &str| {
        if let Ok(mut guard) = ops_deact.lock() {
            guard.push(PendingOp::ModifierDeactivate {
                name: name.to_string(),
                id: 0,
            });
        }
    });

    // modifier_one_shot(name)
    let ops_os = Arc::clone(&ops);
    engine.register_fn("modifier_one_shot", move |name: &str| {
        if let Ok(mut guard) = ops_os.lock() {
            guard.push(PendingOp::ModifierOneShot {
                name: name.to_string(),
                id: 0,
            });
        }
    });

    // tap_timeout(ms)
    let ops_tap_to = Arc::clone(&ops);
    engine.register_fn("tap_timeout", move |ms: i64| {
        if let Ok(mut guard) = ops_tap_to.lock() {
            guard.push(PendingOp::SetTiming(TimingUpdate::TapTimeout(ms as u32)));
        }
    });

    // combo_timeout(ms)
    let ops_combo_to = Arc::clone(&ops);
    engine.register_fn("combo_timeout", move |ms: i64| {
        if let Ok(mut guard) = ops_combo_to.lock() {
            guard.push(PendingOp::SetTiming(TimingUpdate::ComboTimeout(ms as u32)));
        }
    });

    // hold_delay(ms)
    let ops_hold = Arc::clone(&ops);
    engine.register_fn("hold_delay", move |ms: i64| {
        if let Ok(mut guard) = ops_hold.lock() {
            guard.push(PendingOp::SetTiming(TimingUpdate::HoldDelay(ms as u32)));
        }
    });

    // combo(keys, action)
    let ops_combo = Arc::clone(&ops);
    engine.register_fn("combo", move |keys: rhai::Array, action: &str| {
        let mut key_codes = Vec::new();
        for key in keys {
            if let Ok(s) = key.clone().into_string() {
                if let Some(kc) = KeyCode::from_name(&s) {
                    key_codes.push(kc);
                }
            }
        }
        let layer_action = if action == "block" {
            LayerAction::Block
        } else if let Some(kc) = KeyCode::from_name(action) {
            LayerAction::Remap(kc)
        } else {
            return;
        };

        if let Ok(mut guard) = ops_combo.lock() {
            guard.push(PendingOp::Combo {
                keys: key_codes,
                action: layer_action,
            });
        }
    });

    // layer_map(layer, key, action)
    let ops_map = ops;
    engine.register_fn("layer_map", move |layer: &str, key: &str, action: &str| {
        let key_code = match KeyCode::from_name(key) {
            Some(k) => k,
            None => return,
        };
        let map_action = if action == "block" {
            LayerMapAction::Block
        } else if action == "pass" {
            LayerMapAction::Pass
        } else if let Some(kc) = KeyCode::from_name(action) {
            LayerMapAction::Remap(kc)
        } else {
            return;
        };

        if let Ok(mut guard) = ops_map.lock() {
            guard.push(PendingOp::LayerMap {
                layer: layer.to_string(),
                key: key_code,
                action: map_action,
            });
        }
    });

    engine
}

/// Convert a Rhai parse error to a ValidationError.
fn parse_error_to_validation_error(err: Box<rhai::EvalAltResult>) -> ValidationError {
    let (line, col) = match err.position() {
        rhai::Position::NONE => (0, None),
        pos => (pos.line().unwrap_or(0), pos.position()),
    };

    let mut error = ValidationError::new("E000", format!("Parse error: {}", err));
    if line > 0 {
        let mut loc = SourceLocation::new(line);
        if let Some(c) = col {
            loc = loc.with_column(c);
        }
        error = error.with_location(loc);
    }
    error
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn script_context_provides_line_access() {
        let script = "line1\nline2\nline3";
        let context = ScriptContext::from_script(script);
        assert_eq!(context.get_line(1), Some("line1"));
        assert_eq!(context.get_line(2), Some("line2"));
        assert_eq!(context.get_line(3), Some("line3"));
        assert_eq!(context.get_line(0), None);
        assert_eq!(context.get_line(4), None);
    }

    #[test]
    fn script_context_creates_source_location() {
        let script = "remap(\"A\", \"B\");\nblock(\"C\");";
        let context = ScriptContext::from_script(script);
        let loc = context.source_location(1, Some(5));
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, Some(5));
        assert_eq!(loc.context, Some("remap(\"A\", \"B\");".into()));
    }

    #[test]
    fn find_operation_line_locates_remap() {
        let script = r#"
            // Comment
            remap("CapsLock", "Escape");
            block("Insert");
        "#;
        let op = PendingOp::Remap {
            from: crate::engine::KeyCode::CapsLock,
            to: crate::engine::KeyCode::Escape,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(3));
    }

    #[test]
    fn find_operation_line_locates_block() {
        let script = r#"
            remap("A", "B");
            block("Insert");
        "#;
        let op = PendingOp::Block {
            key: crate::engine::KeyCode::Insert,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(3));
    }

    #[test]
    fn find_operation_line_locates_layer_define() {
        let script = r#"
            define_layer("navigation");
            layer_push("navigation");
        "#;
        let op = PendingOp::LayerDefine {
            name: "navigation".to_string(),
            transparent: false,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(2));
    }

    #[test]
    fn find_operation_line_locates_modifier_ops() {
        let script = r#"
            define_modifier("hyper");
            modifier_activate("hyper");
        "#;
        let op = PendingOp::ModifierActivate {
            name: "hyper".to_string(),
            id: 0,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, Some(3));
    }

    #[test]
    fn find_operation_line_returns_none_for_no_match() {
        let script = "remap(\"A\", \"B\");";
        let op = PendingOp::Block {
            key: crate::engine::KeyCode::C,
        };
        let line = find_operation_line(script, &op);
        assert_eq!(line, None);
    }

    #[test]
    fn located_op_stores_index() {
        let op = PendingOp::Block {
            key: crate::engine::KeyCode::A,
        };
        let located = LocatedOp::new(op.clone(), 5);
        assert_eq!(located.index, 5);
        match located.op {
            PendingOp::Block { key } => assert_eq!(key, crate::engine::KeyCode::A),
            _ => panic!("wrong op type"),
        }
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
