//! Validation error definitions.
//!
//! This module defines all errors related to configuration validation,
//! script validation, and conflict detection. Validation errors use the KRX-V4xxx range.

use crate::define_errors;

define_errors! {
    category: Validation,
    base: 4000,

    errors: {
        UNKNOWN_KEY = 1 => {
            message: "Unknown key '{key}' in configuration",
            hint: "Check for typos in the key name. Use 'keyrx list-keys' to see available key names",
            severity: Error,
        },

        UNDEFINED_LAYER = 2 => {
            message: "Reference to undefined layer '{layer}'",
            hint: "Define the layer before referencing it, or remove the reference. Use 'keyrx list-layers' to see defined layers",
            severity: Error,
        },

        UNDEFINED_MODIFIER = 3 => {
            message: "Reference to undefined modifier '{modifier}'",
            hint: "Define the modifier first, or use a standard modifier key like LeftCtrl, LeftShift",
            severity: Error,
        },

        DUPLICATE_REMAP = 4 => {
            message: "Key '{key}' remapped multiple times",
            hint: "Remove duplicate remaps or keep only the intended mapping. Later definitions override earlier ones",
            severity: Warning,
        },

        REMAP_BLOCK_CONFLICT = 5 => {
            message: "Key '{key}' is both remapped and blocked",
            hint: "Decide whether to remap or block the key. Block takes precedence",
            severity: Warning,
        },

        TAP_HOLD_CONFLICT = 6 => {
            message: "Key '{key}' has conflicting tap-hold and remap definitions",
            hint: "Use either tap-hold or remap, not both. The later definition will override",
            severity: Warning,
        },

        COMBO_SHADOWING = 7 => {
            message: "Combo {shorter} shadows combo {longer}",
            hint: "The shorter combo will trigger before the longer combo can complete. Reorder or remove one",
            severity: Warning,
        },

        CIRCULAR_REMAP = 8 => {
            message: "Circular remap detected: {cycle}",
            hint: "Remove one remap from the cycle to break the loop. Circular remaps cause unpredictable behavior",
            severity: Error,
        },

        INVALID_REMAP_TARGET = 9 => {
            message: "Invalid remap target '{target}' for key '{key}'",
            hint: "The target key is not valid. Use 'keyrx list-keys' to see available keys",
            severity: Error,
        },

        SCRIPT_SYNTAX_ERROR = 10 => {
            message: "Script syntax error: {error}",
            hint: "Fix the syntax error in your script. Check line and column numbers in the error",
            severity: Error,
        },

        SCRIPT_COMPILATION_ERROR = 11 => {
            message: "Failed to compile script: {error}",
            hint: "Check the script for type errors or undefined functions. See script documentation",
            severity: Error,
        },

        SCRIPT_UNDEFINED_FUNCTION = 12 => {
            message: "Script references undefined function '{function}'",
            hint: "Define the function before calling it, or check for typos in the function name",
            severity: Error,
        },

        SCRIPT_TYPE_ERROR = 13 => {
            message: "Script type error: expected {expected}, got {actual}",
            hint: "Fix the type mismatch. Check function signatures and return types",
            severity: Error,
        },

        SCRIPT_INVALID_HOOK = 14 => {
            message: "Invalid script hook '{hook}': {reason}",
            hint: "Use valid hook names: on_key, on_layer, on_modifier. Check script documentation",
            severity: Error,
        },

        CONFLICTING_COMBO_DEFINITIONS = 15 => {
            message: "Combo with keys {keys} defined multiple times",
            hint: "Remove duplicate combo definitions or keep only the intended behavior",
            severity: Warning,
        },

        EMPTY_COMBO = 16 => {
            message: "Combo definition has no keys",
            hint: "Add at least one key to the combo, or remove the empty combo definition",
            severity: Error,
        },

        COMBO_TOO_LARGE = 17 => {
            message: "Combo with {count} keys exceeds maximum of {max}",
            hint: "Simplify the combo to use fewer keys. Large combos are hard to trigger",
            severity: Warning,
        },

        DUPLICATE_BLOCK = 18 => {
            message: "Key '{key}' blocked multiple times",
            hint: "Remove duplicate block statements. This is usually redundant",
            severity: Warning,
        },

        INVALID_LAYER_REFERENCE = 19 => {
            message: "Invalid layer reference '{layer}': {reason}",
            hint: "Check the layer name and ensure the layer is defined",
            severity: Error,
        },

        LAYER_DEPTH_EXCEEDED = 20 => {
            message: "Layer nesting depth {depth} exceeds maximum of {max}",
            hint: "Reduce layer nesting. Deep nesting can cause performance issues",
            severity: Warning,
        },

        CONFLICTING_LAYER_BINDINGS = 21 => {
            message: "Key '{key}' has conflicting layer bindings in layer '{layer}'",
            hint: "Remove duplicate layer bindings or keep only the intended behavior",
            severity: Warning,
        },

        INVALID_MODIFIER_DEFINITION = 22 => {
            message: "Invalid modifier definition: {reason}",
            hint: "Check the modifier syntax. Modifiers should use valid key names",
            severity: Error,
        },

        MODIFIER_KEY_CONFLICT = 23 => {
            message: "Key '{key}' is both a modifier and has other mappings",
            hint: "Decide whether the key should be a modifier or have other behavior",
            severity: Warning,
        },

        TIMEOUT_VALUE_INVALID = 24 => {
            message: "Invalid timeout value {value}ms: {reason}",
            hint: "Timeout values should be between 50ms and 5000ms for best results",
            severity: Error,
        },

        INVALID_KEY_SEQUENCE = 25 => {
            message: "Invalid key sequence '{sequence}': {reason}",
            hint: "Check the sequence syntax. Use plus (+) for modifiers and valid key names",
            severity: Error,
        },

        UNSAFE_ESCAPE_BLOCKING = 26 => {
            message: "Escape key is blocked or remapped",
            hint: "Blocking or remapping Escape can cause lockout. Consider using tap-hold instead",
            severity: Warning,
        },

        UNSAFE_CTRL_ALT_DEL_BLOCKING = 27 => {
            message: "Ctrl+Alt+Delete keys are blocked",
            hint: "Blocking these keys prevents emergency exit. Keep at least one emergency exit combo",
            severity: Warning,
        },

        UNSAFE_MODIFIER_BLOCKING = 28 => {
            message: "All {modifier} keys are blocked or remapped",
            hint: "Blocking all instances of a modifier can limit functionality. Keep at least one",
            severity: Warning,
        },

        TOO_MANY_ACTIVE_LAYERS = 29 => {
            message: "Configuration has {count} layers, exceeding recommended maximum of {max}",
            hint: "Simplify layer structure. Too many layers can be hard to manage",
            severity: Warning,
        },

        TOO_MANY_REMAPS = 30 => {
            message: "Configuration has {count} remaps, exceeding recommended maximum of {max}",
            hint: "Consider using layers to organize remaps instead of many individual mappings",
            severity: Warning,
        },

        PERFORMANCE_COMBO_OVERHEAD = 31 => {
            message: "Large number of combos ({count}) may impact performance",
            hint: "Consider using fewer combos or organizing them into layers",
            severity: Info,
        },

        PERFORMANCE_COMPLEX_SCRIPT = 32 => {
            message: "Script complexity ({ops} operations) may impact performance",
            hint: "Simplify the script or optimize frequently-called functions",
            severity: Info,
        },

        MISSING_REQUIRED_HOOK = 33 => {
            message: "Script is missing required hook '{hook}'",
            hint: "Add the required hook function to your script. See documentation for hook signatures",
            severity: Error,
        },

        HOOK_INVALID_SIGNATURE = 34 => {
            message: "Hook '{hook}' has invalid signature: {reason}",
            hint: "Check the hook signature against documentation. Hooks have required parameters and return types",
            severity: Error,
        },

        LAYER_NOT_REACHABLE = 35 => {
            message: "Layer '{layer}' is defined but not reachable",
            hint: "Add a key binding to activate this layer, or remove it if not needed",
            severity: Warning,
        },

        KEY_NOT_COVERED = 36 => {
            message: "Key '{key}' is not covered by any layer or remap",
            hint: "This is informational. The key will pass through unchanged",
            severity: Info,
        },

        CONFLICTING_OPTIONS = 37 => {
            message: "Conflicting options: {option1} and {option2}",
            hint: "These options cannot be used together. Choose one",
            severity: Error,
        },

        DEPRECATED_SYNTAX = 38 => {
            message: "Deprecated syntax: {syntax}",
            hint: "Use {replacement} instead. The old syntax will be removed in a future version",
            severity: Warning,
        },

        INVALID_REGEX_PATTERN = 39 => {
            message: "Invalid regular expression pattern: {pattern}",
            hint: "Fix the regex syntax. Check for unescaped special characters",
            severity: Error,
        },

        VALIDATION_INCOMPLETE = 40 => {
            message: "Validation could not complete: {reason}",
            hint: "This may indicate a bug or corrupt configuration. Try simplifying your config",
            severity: Error,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCategory;
    use crate::keyrx_err;

    #[test]
    fn validation_error_codes_in_range() {
        assert_eq!(UNKNOWN_KEY.code().number(), 4001);
        assert_eq!(CIRCULAR_REMAP.code().number(), 4008);
        assert_eq!(VALIDATION_INCOMPLETE.code().number(), 4040);

        // Verify all are in Validation category range
        assert!(ErrorCategory::Validation.contains(UNKNOWN_KEY.code().number()));
        assert!(ErrorCategory::Validation.contains(VALIDATION_INCOMPLETE.code().number()));
    }

    #[test]
    fn validation_error_categories() {
        assert_eq!(UNKNOWN_KEY.code().category(), ErrorCategory::Validation);
        assert_eq!(
            SCRIPT_SYNTAX_ERROR.code().category(),
            ErrorCategory::Validation
        );
        assert_eq!(COMBO_SHADOWING.code().category(), ErrorCategory::Validation);
    }

    #[test]
    fn validation_error_messages() {
        let err = keyrx_err!(UNKNOWN_KEY, key = "Escpe");
        assert_eq!(err.code(), "KRX-V4001");
        assert!(err.message().contains("Escpe"));
    }

    #[test]
    fn validation_error_hints() {
        assert!(UNKNOWN_KEY.hint().is_some());
        assert!(CIRCULAR_REMAP.hint().unwrap().contains("Remove one"));
        assert!(UNSAFE_ESCAPE_BLOCKING.hint().unwrap().contains("tap-hold"));
    }

    #[test]
    fn validation_error_severities() {
        use crate::errors::ErrorSeverity;

        assert_eq!(UNKNOWN_KEY.severity(), ErrorSeverity::Error);
        assert_eq!(DUPLICATE_REMAP.severity(), ErrorSeverity::Warning);
        assert_eq!(KEY_NOT_COVERED.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn validation_error_formatting() {
        let err = keyrx_err!(LAYER_DEPTH_EXCEEDED, depth = "10", max = "8");
        assert!(err.message().contains("10"));
        assert!(err.message().contains("8"));
        assert!(err.message().contains("depth"));
    }

    #[test]
    fn validation_error_context_substitution() {
        let err = keyrx_err!(SCRIPT_TYPE_ERROR, expected = "bool", actual = "string");
        assert_eq!(err.code(), "KRX-V4013");
        assert!(err.message().contains("bool"));
        assert!(err.message().contains("string"));
    }

    #[test]
    fn conflict_errors() {
        let remap_conflict = keyrx_err!(DUPLICATE_REMAP, key = "A");
        assert!(remap_conflict.message().contains("A"));
        assert!(remap_conflict.message().contains("multiple times"));

        let block_conflict = keyrx_err!(REMAP_BLOCK_CONFLICT, key = "Escape");
        assert!(block_conflict.message().contains("Escape"));
        assert!(block_conflict.message().contains("remapped and blocked"));
    }

    #[test]
    fn script_errors() {
        let syntax_err = keyrx_err!(SCRIPT_SYNTAX_ERROR, error = "unexpected token '}'");
        assert!(syntax_err.message().contains("unexpected token"));

        let undef_fn = keyrx_err!(SCRIPT_UNDEFINED_FUNCTION, function = "on_custom_event");
        assert!(undef_fn.message().contains("on_custom_event"));
    }

    #[test]
    fn layer_errors() {
        let undef_layer = keyrx_err!(UNDEFINED_LAYER, layer = "nav");
        assert!(undef_layer.message().contains("nav"));

        let depth_err = keyrx_err!(LAYER_DEPTH_EXCEEDED, depth = "12", max = "8");
        assert!(depth_err.message().contains("12"));
    }

    #[test]
    fn combo_errors() {
        let shadowing = keyrx_err!(COMBO_SHADOWING, shorter = "[A+S]", longer = "[A+S+D]");
        assert!(shadowing.message().contains("[A+S]"));
        assert!(shadowing.message().contains("shadows"));

        let empty = keyrx_err!(EMPTY_COMBO);
        assert!(empty.message().contains("no keys"));
    }

    #[test]
    fn safety_warnings() {
        let escape_warn = keyrx_err!(UNSAFE_ESCAPE_BLOCKING);
        assert!(escape_warn.message().contains("Escape"));

        let mod_warn = keyrx_err!(UNSAFE_MODIFIER_BLOCKING, modifier = "Control");
        assert!(mod_warn.message().contains("Control"));
    }

    #[test]
    fn performance_hints() {
        let combo_perf = keyrx_err!(PERFORMANCE_COMBO_OVERHEAD, count = "150");
        assert!(combo_perf.message().contains("150"));
        use crate::errors::ErrorSeverity;
        assert_eq!(PERFORMANCE_COMBO_OVERHEAD.severity(), ErrorSeverity::Info);

        let script_perf = keyrx_err!(PERFORMANCE_COMPLEX_SCRIPT, ops = "500");
        assert!(script_perf.message().contains("500"));
        assert_eq!(PERFORMANCE_COMPLEX_SCRIPT.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn circular_remap_error() {
        let err = keyrx_err!(CIRCULAR_REMAP, cycle = "A → B → C → A");
        assert!(err.message().contains("A → B → C → A"));
        assert!(err.message().contains("Circular remap"));
    }

    #[test]
    fn deprecated_syntax_warning() {
        let err = keyrx_err!(
            DEPRECATED_SYNTAX,
            syntax = "old_function()",
            replacement = "new_function()"
        );
        assert!(err.message().contains("old_function()"));
        use crate::errors::ErrorSeverity;
        assert_eq!(DEPRECATED_SYNTAX.severity(), ErrorSeverity::Warning);
    }
}
