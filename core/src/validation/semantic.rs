//! Semantic validation for KeyRx scripts.
//!
//! Validates that key names, layer references, and modifier references
//! are valid and exist when used in operations.

use std::collections::HashSet;

use crate::scripting::{LayerMapAction, PendingOp};
use crate::validation::config::ValidationConfig;
use crate::validation::suggestions::suggest_similar_keys;
use crate::validation::types::ValidationError;

/// Semantic validator for script operations.
///
/// Validates that all key names are valid and that layer/modifier
/// references refer to defined entities.
pub struct SemanticValidator<'a> {
    /// Config is stored for use in timing validation (Task 6).
    #[allow(dead_code)]
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
}
