//! Semantic validation for KeyRx scripts.
//!
//! Validates that key names, layer references, and modifier references
//! are valid and exist when used in operations.

use std::collections::HashSet;

use crate::scripting::{LayerMapAction, PendingOp, TimingUpdate};
use crate::validation::config::ValidationConfig;
use crate::validation::suggestions::suggest_similar_keys;
use crate::validation::types::{ValidationError, ValidationWarning, WarningCategory};

/// Semantic validator for script operations.
///
/// Validates that all key names are valid and that layer/modifier
/// references refer to defined entities.
pub struct SemanticValidator<'a> {
    config: &'a ValidationConfig,
    defined_layers: HashSet<String>,
    defined_modifiers: HashSet<String>,
}

impl<'a> SemanticValidator<'a> {
    /// Create a new semantic validator.
    pub fn new(
        config: &'a ValidationConfig,
        defined_layers: HashSet<String>,
        defined_modifiers: HashSet<String>,
    ) -> Self {
        Self {
            config,
            defined_layers,
            defined_modifiers,
        }
    }

    /// Validate all operations and return errors.
    pub fn validate_operations(&self, ops: &[PendingOp]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // First pass: collect all layer and modifier definitions
        let mut layers = self.defined_layers.clone();
        let mut modifiers = self.defined_modifiers.clone();

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

        // Second pass: validate all operations
        for op in ops {
            self.validate_op(op, &layers, &modifiers, &mut errors);
        }

        errors
    }

    /// Validate timing operations and return warnings.
    pub fn validate_timing(&self, ops: &[PendingOp]) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for op in ops {
            if let PendingOp::SetTiming(update) = op {
                if let Some(warning) = self.check_timing_bounds(update) {
                    warnings.push(warning);
                }
            }
        }

        warnings
    }

    /// Check if a timing value is within configured bounds.
    fn check_timing_bounds(&self, update: &TimingUpdate) -> Option<ValidationWarning> {
        match update {
            TimingUpdate::TapTimeout(ms) => {
                let (min, max) = self.config.tap_timeout_warn_range;
                if *ms < min {
                    Some(ValidationWarning::new(
                        "W001",
                        WarningCategory::Performance,
                        format!(
                            "Tap timeout {}ms is below recommended minimum ({}ms). \
                             Very short timeouts may cause accidental taps.",
                            ms, min
                        ),
                    ))
                } else if *ms > max {
                    Some(ValidationWarning::new(
                        "W001",
                        WarningCategory::Performance,
                        format!(
                            "Tap timeout {}ms exceeds recommended maximum ({}ms). \
                             Very long timeouts may cause sluggish behavior.",
                            ms, max
                        ),
                    ))
                } else {
                    None
                }
            }
            TimingUpdate::ComboTimeout(ms) => {
                let (min, max) = self.config.combo_timeout_warn_range;
                if *ms < min {
                    Some(ValidationWarning::new(
                        "W002",
                        WarningCategory::Performance,
                        format!(
                            "Combo timeout {}ms is below recommended minimum ({}ms). \
                             Very short timeouts may miss combos.",
                            ms, min
                        ),
                    ))
                } else if *ms > max {
                    Some(ValidationWarning::new(
                        "W002",
                        WarningCategory::Performance,
                        format!(
                            "Combo timeout {}ms exceeds recommended maximum ({}ms). \
                             Very long timeouts may cause delayed responses.",
                            ms, max
                        ),
                    ))
                } else {
                    None
                }
            }
            // HoldDelay doesn't have a configured range, skip validation
            TimingUpdate::HoldDelay(_)
            | TimingUpdate::EagerTap(_)
            | TimingUpdate::PermissiveHold(_)
            | TimingUpdate::RetroTap(_) => None,
        }
    }
}

impl SemanticValidator<'_> {
    /// Validate a single operation.
    fn validate_op(
        &self,
        op: &PendingOp,
        layers: &HashSet<String>,
        modifiers: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        match op {
            PendingOp::LayerPush { name } => {
                self.check_layer_exists(name, layers, errors);
            }
            PendingOp::LayerToggle { name } => {
                self.check_layer_exists(name, layers, errors);
            }
            PendingOp::LayerMap { layer, action, .. } => {
                self.check_layer_exists(layer, layers, errors);
                self.check_layer_map_action(action, layers, errors);
            }
            PendingOp::ModifierActivate { name, .. } => {
                self.check_modifier_exists(name, modifiers, errors);
            }
            PendingOp::ModifierDeactivate { name, .. } => {
                self.check_modifier_exists(name, modifiers, errors);
            }
            PendingOp::ModifierOneShot { name, .. } => {
                self.check_modifier_exists(name, modifiers, errors);
            }
            // These ops have already validated KeyCodes at parse time
            PendingOp::Remap { .. }
            | PendingOp::Block { .. }
            | PendingOp::Pass { .. }
            | PendingOp::TapHold { .. }
            | PendingOp::Combo { .. }
            | PendingOp::LayerDefine { .. }
            | PendingOp::DefineModifier { .. }
            | PendingOp::LayoutDefine { .. }
            | PendingOp::LayoutEnable { .. }
            | PendingOp::LayoutDisable { .. }
            | PendingOp::LayoutRemove { .. }
            | PendingOp::LayoutSetPriority { .. }
            | PendingOp::LayerPop
            | PendingOp::SetTiming(_) => {}
        }
    }

    /// Check if a layer reference exists.
    fn check_layer_exists(
        &self,
        name: &str,
        layers: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        if !layers.contains(name) {
            let defined: Vec<String> = layers.iter().cloned().collect();
            errors.push(ValidationError::undefined_layer(name, &defined));
        }
    }

    /// Check if a modifier reference exists.
    fn check_modifier_exists(
        &self,
        name: &str,
        modifiers: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        if !modifiers.contains(name) {
            let defined: Vec<String> = modifiers.iter().cloned().collect();
            errors.push(ValidationError::undefined_modifier(name, &defined));
        }
    }

    /// Check layer map action for layer references.
    fn check_layer_map_action(
        &self,
        action: &LayerMapAction,
        layers: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        match action {
            LayerMapAction::LayerPush(name) => {
                self.check_layer_exists(name, layers, errors);
            }
            LayerMapAction::LayerToggle(name) => {
                self.check_layer_exists(name, layers, errors);
            }
            LayerMapAction::Remap(_)
            | LayerMapAction::Block
            | LayerMapAction::Pass
            | LayerMapAction::TapHold { .. }
            | LayerMapAction::LayerPop => {}
        }
    }
}

/// Validate a key name and return an error with suggestions if invalid.
///
/// This function is useful for validation of raw key names that haven't
/// been parsed into KeyCode yet.
pub fn validate_key_name(key: &str, config: &ValidationConfig) -> Option<ValidationError> {
    use crate::engine::KeyCode;

    if KeyCode::from_name(key).is_some() {
        return None;
    }

    let suggestions = suggest_similar_keys(key, config);
    Some(ValidationError::unknown_key(key, suggestions))
}

/// Convenience function to validate operations with default empty sets.
pub fn validate_operations(
    ops: &[PendingOp],
    layers: &HashSet<String>,
    modifiers: &HashSet<String>,
    config: &ValidationConfig,
) -> Vec<ValidationError> {
    let validator = SemanticValidator::new(config, layers.clone(), modifiers.clone());
    validator.validate_operations(ops)
}

/// Convenience function to validate timing operations.
pub fn validate_timing(ops: &[PendingOp], config: &ValidationConfig) -> Vec<ValidationWarning> {
    let validator = SemanticValidator::new(config, HashSet::new(), HashSet::new());
    validator.validate_timing(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{HoldAction, KeyCode, LayerAction};

    fn test_config() -> ValidationConfig {
        ValidationConfig::default()
    }

    #[test]
    fn valid_key_name_returns_none() {
        let config = test_config();
        assert!(validate_key_name("Escape", &config).is_none());
        assert!(validate_key_name("A", &config).is_none());
        assert!(validate_key_name("LeftCtrl", &config).is_none());
    }

    #[test]
    fn invalid_key_name_returns_error_with_suggestions() {
        let config = test_config();
        let error = validate_key_name("Escpe", &config).unwrap();
        assert_eq!(error.code, "E001");
        assert!(error.message.contains("Escpe"));
        assert!(error.suggestions.contains(&"Escape".to_string()));
    }

    #[test]
    fn undefined_layer_produces_error() {
        let config = test_config();
        let ops = vec![PendingOp::LayerPush {
            name: "undefined_layer".to_string(),
        }];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E002");
        assert!(errors[0].message.contains("undefined_layer"));
    }

    #[test]
    fn defined_layer_produces_no_error() {
        let config = test_config();
        let ops = vec![
            PendingOp::LayerDefine {
                name: "nav".to_string(),
                transparent: false,
            },
            PendingOp::LayerPush {
                name: "nav".to_string(),
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn undefined_modifier_produces_error() {
        let config = test_config();
        let ops = vec![PendingOp::ModifierActivate {
            name: "hyper".to_string(),
            id: 0,
        }];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E003");
        assert!(errors[0].message.contains("hyper"));
    }

    #[test]
    fn defined_modifier_produces_no_error() {
        let config = test_config();
        let ops = vec![
            PendingOp::DefineModifier {
                name: "hyper".to_string(),
                id: 0,
            },
            PendingOp::ModifierActivate {
                name: "hyper".to_string(),
                id: 0,
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn layer_map_with_undefined_layer_produces_error() {
        let config = test_config();
        let ops = vec![PendingOp::LayerMap {
            layer: "undefined".to_string(),
            key: KeyCode::A,
            action: LayerMapAction::Remap(KeyCode::B),
        }];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E002");
    }

    #[test]
    fn layer_map_action_layer_push_validates_target() {
        let config = test_config();
        let mut layers = HashSet::new();
        layers.insert("base".to_string());

        let ops = vec![PendingOp::LayerMap {
            layer: "base".to_string(),
            key: KeyCode::A,
            action: LayerMapAction::LayerPush("nonexistent".to_string()),
        }];

        let errors = validate_operations(&ops, &layers, &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("nonexistent"));
    }

    #[test]
    fn pre_defined_layers_are_recognized() {
        let config = test_config();
        let mut layers = HashSet::new();
        layers.insert("nav".to_string());

        let ops = vec![PendingOp::LayerPush {
            name: "nav".to_string(),
        }];

        let errors = validate_operations(&ops, &layers, &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn pre_defined_modifiers_are_recognized() {
        let config = test_config();
        let mut modifiers = HashSet::new();
        modifiers.insert("hyper".to_string());

        let ops = vec![PendingOp::ModifierActivate {
            name: "hyper".to_string(),
            id: 0,
        }];

        let errors = validate_operations(&ops, &HashSet::new(), &modifiers, &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn remap_ops_dont_produce_errors() {
        let config = test_config();
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::C },
            PendingOp::Pass { key: KeyCode::D },
            PendingOp::TapHold {
                key: KeyCode::E,
                tap: KeyCode::F,
                hold: HoldAction::Key(KeyCode::G),
            },
            PendingOp::Combo {
                keys: vec![KeyCode::H, KeyCode::I],
                action: LayerAction::Remap(KeyCode::J),
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn multiple_errors_collected() {
        let config = test_config();
        let ops = vec![
            PendingOp::LayerPush {
                name: "layer1".to_string(),
            },
            PendingOp::LayerToggle {
                name: "layer2".to_string(),
            },
            PendingOp::ModifierActivate {
                name: "mod1".to_string(),
                id: 0,
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert_eq!(errors.len(), 3);
    }

    // Timing validation tests

    #[test]
    fn tap_timeout_within_range_no_warning() {
        let config = test_config(); // default range: (50, 500)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(200))];

        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());
    }

    #[test]
    fn tap_timeout_below_min_produces_warning() {
        let config = test_config(); // default range: (50, 500)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(10))];

        let warnings = validate_timing(&ops, &config);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W001");
        assert!(warnings[0].message.contains("10ms"));
        assert!(warnings[0].message.contains("below"));
        assert_eq!(warnings[0].category, WarningCategory::Performance);
    }

    #[test]
    fn tap_timeout_above_max_produces_warning() {
        let config = test_config(); // default range: (50, 500)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(1000))];

        let warnings = validate_timing(&ops, &config);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W001");
        assert!(warnings[0].message.contains("1000ms"));
        assert!(warnings[0].message.contains("exceeds"));
    }

    #[test]
    fn combo_timeout_within_range_no_warning() {
        let config = test_config(); // default range: (10, 100)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::ComboTimeout(50))];

        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());
    }

    #[test]
    fn combo_timeout_below_min_produces_warning() {
        let config = test_config(); // default range: (10, 100)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::ComboTimeout(5))];

        let warnings = validate_timing(&ops, &config);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W002");
        assert!(warnings[0].message.contains("5ms"));
        assert!(warnings[0].message.contains("below"));
    }

    #[test]
    fn combo_timeout_above_max_produces_warning() {
        let config = test_config(); // default range: (10, 100)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::ComboTimeout(200))];

        let warnings = validate_timing(&ops, &config);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W002");
        assert!(warnings[0].message.contains("200ms"));
        assert!(warnings[0].message.contains("exceeds"));
    }

    #[test]
    fn hold_delay_does_not_produce_warning() {
        let config = test_config();
        let ops = vec![PendingOp::SetTiming(TimingUpdate::HoldDelay(1000))];

        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());
    }

    #[test]
    fn timing_validation_uses_config_values() {
        // Create custom config with different ranges
        let mut config = ValidationConfig::default();
        config.tap_timeout_warn_range = (100, 200);
        config.combo_timeout_warn_range = (20, 50);

        // 150ms is within custom range (100-200)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(150))];
        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());

        // 80ms is below custom min (100)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(80))];
        let warnings = validate_timing(&ops, &config);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("100ms")); // shows config min

        // 250ms is above custom max (200)
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(250))];
        let warnings = validate_timing(&ops, &config);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("200ms")); // shows config max
    }

    #[test]
    fn multiple_timing_warnings_collected() {
        let config = test_config();
        let ops = vec![
            PendingOp::SetTiming(TimingUpdate::TapTimeout(10)),
            PendingOp::SetTiming(TimingUpdate::ComboTimeout(5)),
        ];

        let warnings = validate_timing(&ops, &config);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn boolean_timing_options_no_warning() {
        let config = test_config();
        let ops = vec![
            PendingOp::SetTiming(TimingUpdate::EagerTap(true)),
            PendingOp::SetTiming(TimingUpdate::PermissiveHold(true)),
            PendingOp::SetTiming(TimingUpdate::RetroTap(true)),
        ];

        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());
    }

    // Additional edge case tests for config-driven behavior and false positive prevention

    #[test]
    fn layer_toggle_with_undefined_layer_produces_error() {
        let config = test_config();
        let ops = vec![PendingOp::LayerToggle {
            name: "missing_layer".to_string(),
        }];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E002");
        assert!(errors[0].message.contains("missing_layer"));
    }

    #[test]
    fn modifier_deactivate_with_undefined_modifier_produces_error() {
        let config = test_config();
        let ops = vec![PendingOp::ModifierDeactivate {
            name: "unknown_mod".to_string(),
            id: 0,
        }];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E003");
    }

    #[test]
    fn modifier_one_shot_with_undefined_modifier_produces_error() {
        let config = test_config();
        let ops = vec![PendingOp::ModifierOneShot {
            name: "oneshot_mod".to_string(),
            id: 0,
        }];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E003");
    }

    #[test]
    fn layer_map_action_layer_toggle_validates_target() {
        let config = test_config();
        let mut layers = HashSet::new();
        layers.insert("base".to_string());

        let ops = vec![PendingOp::LayerMap {
            layer: "base".to_string(),
            key: KeyCode::A,
            action: LayerMapAction::LayerToggle("unknown_layer".to_string()),
        }];

        let errors = validate_operations(&ops, &layers, &HashSet::new(), &config);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("unknown_layer"));
    }

    #[test]
    fn layer_map_non_layer_actions_produce_no_errors() {
        let config = test_config();
        let mut layers = HashSet::new();
        layers.insert("base".to_string());

        let ops = vec![
            PendingOp::LayerMap {
                layer: "base".to_string(),
                key: KeyCode::A,
                action: LayerMapAction::Remap(KeyCode::B),
            },
            PendingOp::LayerMap {
                layer: "base".to_string(),
                key: KeyCode::B,
                action: LayerMapAction::Block,
            },
            PendingOp::LayerMap {
                layer: "base".to_string(),
                key: KeyCode::C,
                action: LayerMapAction::Pass,
            },
            PendingOp::LayerMap {
                layer: "base".to_string(),
                key: KeyCode::D,
                action: LayerMapAction::TapHold {
                    tap: KeyCode::E,
                    hold: HoldAction::Key(KeyCode::F),
                },
            },
            PendingOp::LayerMap {
                layer: "base".to_string(),
                key: KeyCode::E,
                action: LayerMapAction::LayerPop,
            },
        ];

        let errors = validate_operations(&ops, &layers, &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn layer_pop_produces_no_error() {
        let config = test_config();
        let ops = vec![PendingOp::LayerPop];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn set_timing_produces_no_errors_in_semantic_validation() {
        let config = test_config();
        let ops = vec![
            PendingOp::SetTiming(TimingUpdate::TapTimeout(200)),
            PendingOp::SetTiming(TimingUpdate::ComboTimeout(50)),
            PendingOp::SetTiming(TimingUpdate::HoldDelay(300)),
        ];

        // Semantic validation (validate_operations) produces no errors for timing
        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn tap_timeout_at_exact_boundary_no_warning() {
        let config = test_config(); // default range: (50, 500)

        // At exact minimum
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(50))];
        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());

        // At exact maximum
        let ops = vec![PendingOp::SetTiming(TimingUpdate::TapTimeout(500))];
        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());
    }

    #[test]
    fn combo_timeout_at_exact_boundary_no_warning() {
        let config = test_config(); // default range: (10, 100)

        // At exact minimum
        let ops = vec![PendingOp::SetTiming(TimingUpdate::ComboTimeout(10))];
        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());

        // At exact maximum
        let ops = vec![PendingOp::SetTiming(TimingUpdate::ComboTimeout(100))];
        let warnings = validate_timing(&ops, &config);
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_key_name_with_custom_config() {
        let mut config = ValidationConfig::default();
        config.max_suggestions = 2;
        config.similarity_threshold = 2;

        let error = validate_key_name("Escpe", &config);
        assert!(error.is_some());
        let err = error.unwrap();
        assert!(err.suggestions.len() <= 2);
    }

    #[test]
    fn defined_layer_in_multiple_operations_no_errors() {
        let config = test_config();
        let ops = vec![
            PendingOp::LayerDefine {
                name: "nav".to_string(),
                transparent: false,
            },
            PendingOp::LayerPush {
                name: "nav".to_string(),
            },
            PendingOp::LayerToggle {
                name: "nav".to_string(),
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn defined_modifier_in_multiple_operations_no_errors() {
        let config = test_config();
        let ops = vec![
            PendingOp::DefineModifier {
                name: "hyper".to_string(),
                id: 0,
            },
            PendingOp::ModifierActivate {
                name: "hyper".to_string(),
                id: 0,
            },
            PendingOp::ModifierDeactivate {
                name: "hyper".to_string(),
                id: 0,
            },
            PendingOp::ModifierOneShot {
                name: "hyper".to_string(),
                id: 0,
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn layer_defined_after_use_still_valid() {
        // Testing that the two-pass approach correctly handles forward references
        let config = test_config();
        let ops = vec![
            PendingOp::LayerPush {
                name: "future_layer".to_string(),
            },
            PendingOp::LayerDefine {
                name: "future_layer".to_string(),
                transparent: false,
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        // Should have no errors due to two-pass collection
        assert!(errors.is_empty());
    }

    #[test]
    fn modifier_defined_after_use_still_valid() {
        let config = test_config();
        let ops = vec![
            PendingOp::ModifierActivate {
                name: "future_mod".to_string(),
                id: 0,
            },
            PendingOp::DefineModifier {
                name: "future_mod".to_string(),
                id: 0,
            },
        ];

        let errors = validate_operations(&ops, &HashSet::new(), &HashSet::new(), &config);
        // Should have no errors due to two-pass collection
        assert!(errors.is_empty());
    }
}
