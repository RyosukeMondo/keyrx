//! Function capability categorization for the script sandbox.
//!
//! This module categorizes all script-exposed functions by their security tier.
//! Each function is assigned a ScriptCapability tier based on its security impact.
//!
//! This is a factory function that builds a CapabilityRegistry with all functions
//! registered with HashMap-based O(1) lookup.

use super::capability::ScriptCapability;
use super::registry::{CapabilityRegistry, FunctionCapability};

/// Build a complete capability registry with all script functions categorized.
///
/// # Tier Assignment Rationale
///
/// ## Safe Tier (No side effects, bounded execution)
/// - None currently - all functions modify engine state
///
/// ## Standard Tier (May affect engine state, keyboard operations)
/// Total: 24 functions (1 debug + 6 remapping + 6 layer + 5 modifier + 6 timing)
///
/// - `print_debug`: Logging only, no state modification, bounded
/// - `remap`: Core keyboard remapping, modifies engine state
/// - `block`: Blocks key events, modifies engine state
/// - `pass`: Passes key events, modifies engine state
/// - `tap_hold`: Tap-hold behavior, modifies engine state
/// - `tap_hold_mod`: Tap-hold with modifiers, modifies engine state
/// - `combo`: Key combinations, modifies engine state
/// - `layer_define`: Defines layers, modifies engine state
/// - `layer_map`: Maps layer keys, modifies engine state
/// - `layer_push`: Layer stack operation, modifies engine state
/// - `layer_pop`: Layer stack operation, modifies engine state
/// - `layer_toggle`: Layer stack operation, modifies engine state
/// - `is_layer_active`: Read-only query, no modification but reads state
/// - `define_modifier`: Defines virtual modifiers, modifies engine state
/// - `modifier_on`: Activates modifiers, modifies engine state
/// - `modifier_off`: Deactivates modifiers, modifies engine state
/// - `one_shot`: One-shot modifier behavior, modifies engine state
/// - `is_modifier_active`: Read-only query, no modification but reads state
/// - `set_tap_timeout`: Timing configuration, modifies engine state
/// - `set_combo_timeout`: Timing configuration, modifies engine state
/// - `set_hold_delay`: Timing configuration, modifies engine state
/// - `set_eager_tap`: Timing configuration, modifies engine state
/// - `set_permissive_hold`: Timing configuration, modifies engine state
/// - `set_retro_tap`: Timing configuration, modifies engine state
///
/// ## Advanced Tier (System interaction, requires trust)
/// - None currently - no system interaction functions yet
///
/// ## Internal Tier (Not exposed to user scripts)
/// - None currently - all functions are user-facing
///
/// # Security Considerations
///
/// All current functions are Standard tier because they:
/// 1. Modify keyboard remapping state (the core purpose of KeyRx)
/// 2. Do not access system resources beyond keyboard events
/// 3. Have bounded execution time
/// 4. Cannot cause unbounded resource consumption
/// 5. Are essential for keyboard customization
///
/// Future functions that interact with clipboard, filesystem, network, or
/// execute external commands would require Advanced tier.
///
/// # Performance
///
/// The registry uses HashMap for O(1) lookup by function name and by KeyCode.
/// This meets the requirement in Requirement 4: Registry Optimization.
pub fn build_function_registry() -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::with_capacity(24);

    // Debug functions - Standard (logging only, no dangerous side effects)
    registry.register(FunctionCapability::new(
        "print_debug",
        ScriptCapability::Standard,
        "Print debug message to log",
    ));

    // Remapping functions - Standard (core keyboard functionality)
    registry.register(FunctionCapability::new(
        "remap",
        ScriptCapability::Standard,
        "Remap a key to another key",
    ));
    registry.register(FunctionCapability::new(
        "block",
        ScriptCapability::Standard,
        "Block a key event",
    ));
    registry.register(FunctionCapability::new(
        "pass",
        ScriptCapability::Standard,
        "Pass a key event through unchanged",
    ));
    registry.register(FunctionCapability::new(
        "tap_hold",
        ScriptCapability::Standard,
        "Register tap-hold behavior for a key",
    ));
    registry.register(FunctionCapability::new(
        "tap_hold_mod",
        ScriptCapability::Standard,
        "Register tap-hold behavior with modifier",
    ));
    registry.register(FunctionCapability::new(
        "combo",
        ScriptCapability::Standard,
        "Register a key combination",
    ));

    // Layer functions - Standard (core keyboard functionality)
    registry.register(FunctionCapability::new(
        "layer_define",
        ScriptCapability::Standard,
        "Define a new layer",
    ));
    registry.register(FunctionCapability::new(
        "layer_map",
        ScriptCapability::Standard,
        "Map a key in a layer",
    ));
    registry.register(FunctionCapability::new(
        "layer_push",
        ScriptCapability::Standard,
        "Push a layer onto the stack",
    ));
    registry.register(FunctionCapability::new(
        "layer_pop",
        ScriptCapability::Standard,
        "Pop the top layer from the stack",
    ));
    registry.register(FunctionCapability::new(
        "layer_toggle",
        ScriptCapability::Standard,
        "Toggle a layer on/off",
    ));
    registry.register(FunctionCapability::new(
        "is_layer_active",
        ScriptCapability::Standard,
        "Check if a layer is active",
    ));

    // Modifier functions - Standard (core keyboard functionality)
    registry.register(FunctionCapability::new(
        "define_modifier",
        ScriptCapability::Standard,
        "Define a virtual modifier",
    ));
    registry.register(FunctionCapability::new(
        "modifier_on",
        ScriptCapability::Standard,
        "Activate a modifier",
    ));
    registry.register(FunctionCapability::new(
        "modifier_off",
        ScriptCapability::Standard,
        "Deactivate a modifier",
    ));
    registry.register(FunctionCapability::new(
        "one_shot",
        ScriptCapability::Standard,
        "Arm a one-shot modifier",
    ));
    registry.register(FunctionCapability::new(
        "is_modifier_active",
        ScriptCapability::Standard,
        "Check if a modifier is active",
    ));

    // Timing functions - Standard (configuration, affects timing behavior)
    registry.register(FunctionCapability::new(
        "set_tap_timeout",
        ScriptCapability::Standard,
        "Set tap timeout duration",
    ));
    registry.register(FunctionCapability::new(
        "set_combo_timeout",
        ScriptCapability::Standard,
        "Set combo timeout duration",
    ));
    registry.register(FunctionCapability::new(
        "set_hold_delay",
        ScriptCapability::Standard,
        "Set hold delay duration",
    ));
    registry.register(FunctionCapability::new(
        "set_eager_tap",
        ScriptCapability::Standard,
        "Configure eager tap mode",
    ));
    registry.register(FunctionCapability::new(
        "set_permissive_hold",
        ScriptCapability::Standard,
        "Configure permissive hold mode",
    ));
    registry.register(FunctionCapability::new(
        "set_retro_tap",
        ScriptCapability::Standard,
        "Configure retro tap mode",
    ));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::sandbox::capability::ScriptMode;

    #[test]
    fn test_all_functions_categorized() {
        let registry = build_function_registry();

        // Debug
        assert!(registry.get("print_debug").is_some());

        // Remapping
        assert!(registry.get("remap").is_some());
        assert!(registry.get("block").is_some());
        assert!(registry.get("pass").is_some());
        assert!(registry.get("tap_hold").is_some());
        assert!(registry.get("tap_hold_mod").is_some());
        assert!(registry.get("combo").is_some());

        // Layers
        assert!(registry.get("layer_define").is_some());
        assert!(registry.get("layer_map").is_some());
        assert!(registry.get("layer_push").is_some());
        assert!(registry.get("layer_pop").is_some());
        assert!(registry.get("layer_toggle").is_some());
        assert!(registry.get("is_layer_active").is_some());

        // Modifiers
        assert!(registry.get("define_modifier").is_some());
        assert!(registry.get("modifier_on").is_some());
        assert!(registry.get("modifier_off").is_some());
        assert!(registry.get("one_shot").is_some());
        assert!(registry.get("is_modifier_active").is_some());

        // Timing
        assert!(registry.get("set_tap_timeout").is_some());
        assert!(registry.get("set_combo_timeout").is_some());
        assert!(registry.get("set_hold_delay").is_some());
        assert!(registry.get("set_eager_tap").is_some());
        assert!(registry.get("set_permissive_hold").is_some());
        assert!(registry.get("set_retro_tap").is_some());
    }

    #[test]
    fn test_standard_tier_functions() {
        let registry = build_function_registry();
        let standard_funcs = registry.by_tier(ScriptCapability::Standard);

        // All current functions should be Standard tier
        // 1 debug + 6 remapping + 6 layer + 5 modifier + 6 timing = 24 total
        assert_eq!(standard_funcs.len(), 24);
        let names: Vec<_> = standard_funcs.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"remap"));
        assert!(names.contains(&"layer_define"));
        assert!(names.contains(&"define_modifier"));
    }

    #[test]
    fn test_functions_allowed_in_standard_mode() {
        let registry = build_function_registry();

        // All current functions should be allowed in Standard mode
        assert!(registry.is_allowed("remap", ScriptMode::Standard));
        assert!(registry.is_allowed("layer_define", ScriptMode::Standard));
        assert!(registry.is_allowed("define_modifier", ScriptMode::Standard));
        assert!(registry.is_allowed("print_debug", ScriptMode::Standard));
    }

    #[test]
    fn test_functions_not_allowed_in_safe_mode() {
        let registry = build_function_registry();

        // Standard functions should not be allowed in Safe mode
        assert!(!registry.is_allowed("remap", ScriptMode::Safe));
        assert!(!registry.is_allowed("layer_define", ScriptMode::Safe));
        assert!(!registry.is_allowed("define_modifier", ScriptMode::Safe));
    }

    #[test]
    fn test_functions_allowed_in_full_mode() {
        let registry = build_function_registry();

        // All Standard functions should be allowed in Full mode
        assert!(registry.is_allowed("remap", ScriptMode::Full));
        assert!(registry.is_allowed("layer_define", ScriptMode::Full));
        assert!(registry.is_allowed("define_modifier", ScriptMode::Full));
    }

    #[test]
    fn test_unknown_function() {
        let registry = build_function_registry();
        assert!(registry.get("nonexistent_function").is_none());
        assert!(!registry.is_allowed("nonexistent_function", ScriptMode::Full));
    }

    #[test]
    fn test_registry_size() {
        let registry = build_function_registry();
        assert_eq!(registry.len(), 24);
        assert!(!registry.is_empty());
    }
}
