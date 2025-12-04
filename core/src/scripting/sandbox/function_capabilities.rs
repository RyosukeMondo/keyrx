//! Function capability categorization for the script sandbox.
//!
//! This module categorizes all script-exposed functions by their security tier.
//! Each function is assigned a ScriptCapability tier based on its security impact.

use super::capability::ScriptCapability;
use std::collections::HashMap;

/// Capability categorization for all script functions.
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
pub struct FunctionCapabilities {
    capabilities: HashMap<&'static str, ScriptCapability>,
}

impl FunctionCapabilities {
    /// Create a new function capabilities registry with all functions categorized.
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();

        // Debug functions - Standard (logging only, no dangerous side effects)
        capabilities.insert("print_debug", ScriptCapability::Standard);

        // Remapping functions - Standard (core keyboard functionality)
        capabilities.insert("remap", ScriptCapability::Standard);
        capabilities.insert("block", ScriptCapability::Standard);
        capabilities.insert("pass", ScriptCapability::Standard);
        capabilities.insert("tap_hold", ScriptCapability::Standard);
        capabilities.insert("tap_hold_mod", ScriptCapability::Standard);
        capabilities.insert("combo", ScriptCapability::Standard);

        // Layer functions - Standard (core keyboard functionality)
        capabilities.insert("layer_define", ScriptCapability::Standard);
        capabilities.insert("layer_map", ScriptCapability::Standard);
        capabilities.insert("layer_push", ScriptCapability::Standard);
        capabilities.insert("layer_pop", ScriptCapability::Standard);
        capabilities.insert("layer_toggle", ScriptCapability::Standard);
        capabilities.insert("is_layer_active", ScriptCapability::Standard);

        // Modifier functions - Standard (core keyboard functionality)
        capabilities.insert("define_modifier", ScriptCapability::Standard);
        capabilities.insert("modifier_on", ScriptCapability::Standard);
        capabilities.insert("modifier_off", ScriptCapability::Standard);
        capabilities.insert("one_shot", ScriptCapability::Standard);
        capabilities.insert("is_modifier_active", ScriptCapability::Standard);

        // Timing functions - Standard (configuration, affects timing behavior)
        capabilities.insert("set_tap_timeout", ScriptCapability::Standard);
        capabilities.insert("set_combo_timeout", ScriptCapability::Standard);
        capabilities.insert("set_hold_delay", ScriptCapability::Standard);
        capabilities.insert("set_eager_tap", ScriptCapability::Standard);
        capabilities.insert("set_permissive_hold", ScriptCapability::Standard);
        capabilities.insert("set_retro_tap", ScriptCapability::Standard);

        Self { capabilities }
    }

    /// Get the capability tier for a function.
    ///
    /// Returns `None` if the function is not registered.
    pub fn get(&self, function_name: &str) -> Option<ScriptCapability> {
        self.capabilities.get(function_name).copied()
    }

    /// Get all function names categorized by capability tier.
    pub fn by_capability(&self, capability: ScriptCapability) -> Vec<&'static str> {
        self.capabilities
            .iter()
            .filter_map(|(name, cap)| {
                if *cap == capability {
                    Some(*name)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all registered functions with their capability tiers.
    pub fn all(&self) -> &HashMap<&'static str, ScriptCapability> {
        &self.capabilities
    }

    /// Check if a function is allowed in the given mode.
    pub fn is_allowed(&self, function_name: &str, mode: super::capability::ScriptMode) -> bool {
        self.get(function_name)
            .map(|cap| cap.is_allowed_in(mode))
            .unwrap_or(false)
    }
}

impl Default for FunctionCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::sandbox::capability::ScriptMode;

    #[test]
    fn test_all_functions_categorized() {
        let caps = FunctionCapabilities::new();

        // Debug
        assert!(caps.get("print_debug").is_some());

        // Remapping
        assert!(caps.get("remap").is_some());
        assert!(caps.get("block").is_some());
        assert!(caps.get("pass").is_some());
        assert!(caps.get("tap_hold").is_some());
        assert!(caps.get("tap_hold_mod").is_some());
        assert!(caps.get("combo").is_some());

        // Layers
        assert!(caps.get("layer_define").is_some());
        assert!(caps.get("layer_map").is_some());
        assert!(caps.get("layer_push").is_some());
        assert!(caps.get("layer_pop").is_some());
        assert!(caps.get("layer_toggle").is_some());
        assert!(caps.get("is_layer_active").is_some());

        // Modifiers
        assert!(caps.get("define_modifier").is_some());
        assert!(caps.get("modifier_on").is_some());
        assert!(caps.get("modifier_off").is_some());
        assert!(caps.get("one_shot").is_some());
        assert!(caps.get("is_modifier_active").is_some());

        // Timing
        assert!(caps.get("set_tap_timeout").is_some());
        assert!(caps.get("set_combo_timeout").is_some());
        assert!(caps.get("set_hold_delay").is_some());
        assert!(caps.get("set_eager_tap").is_some());
        assert!(caps.get("set_permissive_hold").is_some());
        assert!(caps.get("set_retro_tap").is_some());
    }

    #[test]
    fn test_standard_tier_functions() {
        let caps = FunctionCapabilities::new();
        let standard_funcs = caps.by_capability(ScriptCapability::Standard);

        // All current functions should be Standard tier
        // 1 debug + 6 remapping + 6 layer + 5 modifier + 6 timing = 24 total
        assert_eq!(standard_funcs.len(), 24);
        assert!(standard_funcs.contains(&"remap"));
        assert!(standard_funcs.contains(&"layer_define"));
        assert!(standard_funcs.contains(&"define_modifier"));
    }

    #[test]
    fn test_functions_allowed_in_standard_mode() {
        let caps = FunctionCapabilities::new();

        // All current functions should be allowed in Standard mode
        assert!(caps.is_allowed("remap", ScriptMode::Standard));
        assert!(caps.is_allowed("layer_define", ScriptMode::Standard));
        assert!(caps.is_allowed("define_modifier", ScriptMode::Standard));
        assert!(caps.is_allowed("print_debug", ScriptMode::Standard));
    }

    #[test]
    fn test_functions_not_allowed_in_safe_mode() {
        let caps = FunctionCapabilities::new();

        // Standard functions should not be allowed in Safe mode
        assert!(!caps.is_allowed("remap", ScriptMode::Safe));
        assert!(!caps.is_allowed("layer_define", ScriptMode::Safe));
        assert!(!caps.is_allowed("define_modifier", ScriptMode::Safe));
    }

    #[test]
    fn test_functions_allowed_in_full_mode() {
        let caps = FunctionCapabilities::new();

        // All Standard functions should be allowed in Full mode
        assert!(caps.is_allowed("remap", ScriptMode::Full));
        assert!(caps.is_allowed("layer_define", ScriptMode::Full));
        assert!(caps.is_allowed("define_modifier", ScriptMode::Full));
    }

    #[test]
    fn test_unknown_function() {
        let caps = FunctionCapabilities::new();
        assert!(caps.get("nonexistent_function").is_none());
        assert!(!caps.is_allowed("nonexistent_function", ScriptMode::Full));
    }
}
