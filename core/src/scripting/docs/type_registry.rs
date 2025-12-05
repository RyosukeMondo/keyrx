//! Type documentation registration for Rhai API types.
//!
//! This module registers documentation for all types used in the Rhai API.
//! While KeyRx doesn't expose custom Rhai types with methods/properties,
//! it uses built-in types (String, int, bool, Array) to represent domain concepts.
//! This module documents these conceptual types for user understanding.

use super::registry::register_type;
use super::types::TypeDoc;

/// Register all type documentation with the global registry.
///
/// This should be called during initialization to populate the type documentation.
pub fn register_all_types() {
    register_key_code_type();
    register_layer_name_type();
    register_modifier_name_type();
    register_modifier_id_type();
    register_layer_action_type();
}

/// KeyCode represents a keyboard key identifier, passed as a String.
fn key_code_type_doc() -> TypeDoc {
    TypeDoc {
        name: "KeyCode".to_string(),
        description: "Represents a keyboard key identifier. Keys can be specified by their \
                      physical key name (e.g., 'A', 'Space', 'Enter') or by special key names \
                      (e.g., 'CapsLock', 'Esc', 'AudioMute'). Key names are case-insensitive."
            .to_string(),
        module: "keys".to_string(),
        methods: vec![],
        properties: vec![],
        constructors: vec![],
        since: Some("0.1.0".to_string()),
        examples: vec![
            r#"// Standard alphanumeric keys
remap("A", "B");
remap("1", "2");"#
                .to_string(),
            r#"// Special keys
remap("CapsLock", "Esc");
block("F1");"#
                .to_string(),
            r#"// Modifier keys
tap_hold("Tab", "Tab", "Ctrl");
remap("Space", "Shift");"#
                .to_string(),
            r#"// Media keys
remap("F13", "AudioMute");
remap("F14", "AudioVolumeDown");"#
                .to_string(),
        ],
    }
}

/// Register KeyCode type documentation.
fn register_key_code_type() {
    register_type(key_code_type_doc());
}

/// LayerName represents a layer identifier, passed as a String.
fn layer_name_type_doc() -> TypeDoc {
    TypeDoc {
        name: "LayerName".to_string(),
        description: "Represents a layer identifier used to organize and activate different \
                      key mappings. Layer names are case-insensitive strings that uniquely \
                      identify a layer. Common examples include 'Nav', 'Symbols', 'Numbers', etc."
            .to_string(),
        module: "layers".to_string(),
        methods: vec![],
        properties: vec![],
        constructors: vec![],
        since: Some("0.1.0".to_string()),
        examples: vec![
            r#"// Define layers
layer_define("Nav", true);
layer_define("Symbols", false);"#
                .to_string(),
            r#"// Activate layers
layer_push("Nav");
layer_toggle("Symbols");"#
                .to_string(),
            r#"// Map keys on layers
layer_map("Nav", "H", "Left");
layer_map("Nav", "J", "Down");"#
                .to_string(),
        ],
    }
}

/// Register LayerName type documentation.
fn register_layer_name_type() {
    register_type(layer_name_type_doc());
}

/// ModifierName represents a virtual modifier identifier, passed as a String.
fn modifier_name_type_doc() -> TypeDoc {
    TypeDoc {
        name: "ModifierName".to_string(),
        description: "Represents a virtual modifier identifier. Virtual modifiers are custom \
                      modifier keys you define and control in your scripts. Names are \
                      case-insensitive strings. Common examples include 'Hyper', 'Meh', etc."
            .to_string(),
        module: "modifiers".to_string(),
        methods: vec![],
        properties: vec![],
        constructors: vec![],
        since: Some("0.1.0".to_string()),
        examples: vec![
            r#"// Define virtual modifiers
let hyper = define_modifier("Hyper");
let meh = define_modifier("Meh");"#
                .to_string(),
            r#"// Activate/deactivate modifiers
modifier_on("Hyper");
modifier_off("Hyper");"#
                .to_string(),
            r#"// Use in tap-hold bindings
tap_hold_mod("CapsLock", "Esc", hyper);"#
                .to_string(),
        ],
    }
}

/// Register ModifierName type documentation.
fn register_modifier_name_type() {
    register_type(modifier_name_type_doc());
}

/// ModifierId represents a unique identifier for a virtual modifier, as an integer (0-255).
fn modifier_id_type_doc() -> TypeDoc {
    TypeDoc {
        name: "ModifierId".to_string(),
        description: "Represents a unique identifier for a virtual modifier. This is an integer \
                      value (0-255) returned by define_modifier() and used with tap_hold_mod(). \
                      You should store this value and reuse it for consistency."
            .to_string(),
        module: "modifiers".to_string(),
        methods: vec![],
        properties: vec![],
        constructors: vec![],
        since: Some("0.1.0".to_string()),
        examples: vec![
            r#"// Get modifier ID from define_modifier
let hyper = define_modifier("Hyper");
// hyper is now an integer (e.g., 0, 1, 2, etc.)"#
                .to_string(),
            r#"// Use modifier ID with tap_hold_mod
let hyper = define_modifier("Hyper");
tap_hold_mod("CapsLock", "Esc", hyper);"#
                .to_string(),
        ],
    }
}

/// Register ModifierId type documentation.
fn register_modifier_id_type() {
    register_type(modifier_id_type_doc());
}

/// LayerAction represents an action string used in layer mappings.
fn layer_action_type_doc() -> TypeDoc {
    TypeDoc {
        name: "LayerAction".to_string(),
        description: "Represents an action that can be performed on a layer mapping. Layer \
                      actions are specified as strings and can be either simple key remaps or \
                      layer stack operations (push, pop, toggle). The format supports prefixes \
                      like 'push:', 'pop:', and 'toggle:' for layer operations."
            .to_string(),
        module: "layers".to_string(),
        methods: vec![],
        properties: vec![],
        constructors: vec![],
        since: Some("0.1.0".to_string()),
        examples: vec![
            r#"// Simple key remap action
layer_map("Nav", "H", "Left");
layer_map("Nav", "J", "Down");"#
                .to_string(),
            r#"// Push layer action
layer_map("Nav", "Space", "push:Symbols");"#
                .to_string(),
            r#"// Toggle layer action
layer_map("Base", "F1", "toggle:Nav");"#
                .to_string(),
            r#"// Pop layer action (less common)
layer_map("Symbols", "Esc", "pop:");"#
                .to_string(),
        ],
    }
}

/// Register LayerAction type documentation.
fn register_layer_action_type() {
    register_type(layer_action_type_doc());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::registry;
    use crate::scripting::docs::search::{search, SearchOptions};

    #[test]
    fn test_register_all_types() {
        // Initialize registry first
        registry::initialize();

        // Register all types
        register_all_types();

        // Verify types are registered
        assert!(registry::get_type("KeyCode").is_some());
        assert!(registry::get_type("LayerName").is_some());
        assert!(registry::get_type("ModifierName").is_some());
        assert!(registry::get_type("ModifierId").is_some());
        assert!(registry::get_type("LayerAction").is_some());
    }

    #[test]
    fn test_keycode_type_has_examples() {
        registry::initialize();
        register_key_code_type();
        let keycode = registry::get_type("KeyCode").unwrap();
        assert!(!keycode.examples.is_empty());
        assert!(keycode.module == "keys");
    }

    #[test]
    fn test_layer_name_type_searchable() {
        registry::initialize();
        register_layer_name_type();
        let results = search("layer", SearchOptions::default());
        assert!(results.iter().any(|r| r.name == "LayerName"));
    }

    #[test]
    fn test_modifier_types_in_correct_module() {
        registry::initialize();
        register_modifier_name_type();
        register_modifier_id_type();

        let modifier_name = registry::get_type("ModifierName").unwrap();
        let modifier_id = registry::get_type("ModifierId").unwrap();

        assert_eq!(modifier_name.module, "modifiers");
        assert_eq!(modifier_id.module, "modifiers");
    }

    #[test]
    fn test_layer_action_type_has_comprehensive_examples() {
        registry::initialize();
        register_layer_action_type();
        let action = registry::get_type("LayerAction").unwrap();

        // Should have examples for remap, push, toggle, pop
        assert!(action.examples.len() >= 4);
        assert!(action
            .examples
            .iter()
            .any(|ex| ex.contains("push:") || ex.contains("Simple")));
        assert!(action.examples.iter().any(|ex| ex.contains("toggle:")));
    }

    #[test]
    fn test_all_types_have_since_version() {
        for doc in [
            key_code_type_doc(),
            layer_name_type_doc(),
            modifier_name_type_doc(),
            modifier_id_type_doc(),
            layer_action_type_doc(),
        ] {
            assert!(doc.since.is_some(), "{} should have since", doc.name);
        }
    }
}
