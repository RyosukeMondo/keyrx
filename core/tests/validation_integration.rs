//! Integration tests for the validation pipeline.
//!
//! Tests the complete validation flow including:
//! - ValidationEngine with real scripts
//! - CLI output format and exit codes
//! - FFI roundtrip
//! - JSON output parsing
//! - Config-driven behavior

use keyrx_core::cli::commands::{check_exit_codes, CheckCommand};
use keyrx_core::cli::OutputFormat;
use keyrx_core::validation::config::ValidationConfig;
use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::types::{ValidationOptions, WarningCategory};
use std::io::Write;
use tempfile::NamedTempFile;

// =============================================================================
// ValidationEngine Integration Tests
// =============================================================================

mod validation_engine {
    use super::*;

    #[test]
    fn validates_simple_remap_script() {
        let script = r#"
            remap("CapsLock", "Escape");
            remap("A", "B");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validates_tap_hold_script() {
        let script = r#"
            tap_hold("CapsLock", "Escape", "LeftCtrl");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validates_layer_script() {
        let script = r#"
            define_layer("navigation");
            define_layer("symbols", true);
            layer_push("navigation");
            layer_toggle("symbols");
            layer_pop();
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validates_modifier_script() {
        let script = r#"
            define_modifier("hyper");
            modifier_activate("hyper");
            modifier_deactivate("hyper");
            modifier_one_shot("hyper");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validates_combo_script() {
        let script = r#"
            combo(["A", "S"], "Escape");
            combo(["D", "F"], "block");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validates_layer_map_script() {
        let script = r#"
            define_layer("nav");
            layer_map("nav", "H", "Left");
            layer_map("nav", "J", "Down");
            layer_map("nav", "K", "Up");
            layer_map("nav", "L", "Right");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validates_timing_script() {
        let script = r#"
            tap_timeout(200);
            combo_timeout(50);
            hold_delay(150);
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn detects_invalid_key_name() {
        // Invalid key names cause Rhai runtime errors (E000)
        let script = r#"remap("InvalidKeyName123", "Escape");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(!result.is_valid);
        // Invalid keys in Rhai functions cause runtime errors
        assert!(result.errors.iter().any(|e| e.code == "E000"));
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("Unknown key")));
    }

    #[test]
    fn detects_undefined_layer() {
        let script = r#"layer_push("nonexistent_layer");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "E002"));
    }

    #[test]
    fn detects_undefined_modifier() {
        let script = r#"modifier_activate("undefined_mod");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "E003"));
    }

    #[test]
    fn detects_duplicate_remap_warning() {
        let script = r#"
            remap("A", "B");
            remap("A", "C");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid); // Warnings don't make script invalid
        assert!(result.has_warnings());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.category == WarningCategory::Conflict));
    }

    #[test]
    fn detects_escape_remap_safety_warning() {
        let script = r#"remap("Escape", "A");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.has_warnings());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.category == WarningCategory::Safety));
    }

    #[test]
    fn no_false_positives_on_valid_complex_script() {
        let script = r#"
            // A more complex script with various features
            remap("CapsLock", "Escape");
            tap_hold("Space", "Space", "LeftShift");

            define_layer("nav");
            layer_map("nav", "H", "Left");
            layer_map("nav", "J", "Down");
            layer_map("nav", "K", "Up");
            layer_map("nav", "L", "Right");

            define_modifier("hyper");

            tap_timeout(200);
            combo_timeout(50);

            combo(["A", "S"], "Tab");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn coverage_report_tracks_affected_keys() {
        let script = r#"
            remap("A", "B");
            block("C");
            tap_hold("D", "E", "F");
            combo(["G", "H"], "I");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new().with_coverage());

        assert!(result.is_valid);
        assert!(result.coverage.is_some());

        let coverage = result.coverage.unwrap();
        assert!(coverage.remapped.iter().any(|k| k.name() == "A"));
        assert!(coverage.blocked.iter().any(|k| k.name() == "C"));
        assert!(coverage.tap_hold.iter().any(|k| k.name() == "D"));
        assert!(
            coverage.combo_triggers.iter().any(|k| k.name() == "G")
                || coverage.combo_triggers.iter().any(|k| k.name() == "H")
        );
    }

    #[test]
    fn visual_output_contains_legend() {
        let script = r#"remap("A", "B");"#;

        let engine = ValidationEngine::new();
        let (result, visual) = engine.validate_with_visual(script);

        assert!(result.is_valid);
        assert!(visual.is_some());
        let visual_str = visual.unwrap();
        assert!(visual_str.contains("Legend:"));
    }
}

// =============================================================================
// Config-Driven Validation Tests
// =============================================================================

mod config_driven {
    use super::*;

    #[test]
    fn respects_max_errors_limit() {
        let mut config = ValidationConfig::default();
        config.max_errors = 2;

        let script = r#"
            layer_push("layer1");
            layer_push("layer2");
            layer_push("layer3");
            layer_push("layer4");
        "#;

        let engine = ValidationEngine::with_config(config);
        let result = engine.validate(script, ValidationOptions::new());

        // Should stop at max_errors
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn respects_custom_tap_timeout_range() {
        let mut config = ValidationConfig::default();
        config.tap_timeout_warn_range = (100, 300);

        let engine = ValidationEngine::with_config(config);

        // 200ms is within range - no warning
        let script1 = "tap_timeout(200);";
        let result1 = engine.validate(script1, ValidationOptions::new());
        assert!(!result1.has_warnings());

        // 50ms is below min - warning
        let script2 = "tap_timeout(50);";
        let result2 = engine.validate(script2, ValidationOptions::new());
        assert!(result2.has_warnings());

        // 500ms is above max - warning
        let script3 = "tap_timeout(500);";
        let result3 = engine.validate(script3, ValidationOptions::new());
        assert!(result3.has_warnings());
    }

    #[test]
    fn respects_custom_combo_timeout_range() {
        let mut config = ValidationConfig::default();
        config.combo_timeout_warn_range = (20, 80);

        let engine = ValidationEngine::with_config(config);

        // 50ms is within range - no warning
        let script1 = "combo_timeout(50);";
        let result1 = engine.validate(script1, ValidationOptions::new());
        assert!(!result1.has_warnings());

        // 5ms is below min - warning
        let script2 = "combo_timeout(5);";
        let result2 = engine.validate(script2, ValidationOptions::new());
        assert!(result2.has_warnings());
    }

    #[test]
    fn respects_custom_cycle_depth() {
        let mut config = ValidationConfig::default();
        config.max_cycle_depth = 3;

        // This creates a 2-step cycle which should be detected
        let script = r#"
            remap("A", "B");
            remap("B", "A");
        "#;

        let engine = ValidationEngine::with_config(config);
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.has_warnings());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.category == WarningCategory::Conflict));
    }

    #[test]
    fn loads_config_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("validation.toml");

        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(file, "max_errors = 5").unwrap();
        writeln!(file, "max_suggestions = 3").unwrap();

        let config = ValidationConfig::load_from_path(&config_path).unwrap();
        assert_eq!(config.max_errors, 5);
        assert_eq!(config.max_suggestions, 3);
    }
}

// =============================================================================
// CLI Integration Tests
// =============================================================================

mod cli_integration {
    use super::*;

    fn create_script_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn exit_code_valid_script() {
        let file = create_script_file(r#"remap("CapsLock", "Escape");"#);
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let exit = cmd.run().unwrap();
        assert_eq!(exit, check_exit_codes::VALID);
    }

    #[test]
    fn exit_code_errors() {
        let file = create_script_file(r#"remap("InvalidKey", "Escape");"#);
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human);
        let exit = cmd.run().unwrap();
        assert_eq!(exit, check_exit_codes::ERRORS);
    }

    #[test]
    fn exit_code_strict_with_warnings() {
        let file = create_script_file(
            r#"
            remap("A", "B");
            remap("A", "C");
        "#,
        );
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).strict();
        let exit = cmd.run().unwrap();
        assert_eq!(exit, check_exit_codes::WARNINGS_STRICT);
    }

    #[test]
    fn custom_config_affects_validation() {
        // Create config with very restrictive tap timeout range
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "tap_timeout_warn_range = [100, 200]").unwrap();

        // Script with tap_timeout outside the range
        let script_file = create_script_file("tap_timeout(50);");

        let cmd = CheckCommand::new(script_file.path().to_path_buf(), OutputFormat::Human)
            .with_config(config_file.path().to_path_buf());
        let exit = cmd.run().unwrap();
        // Should still be valid (warnings don't cause failure by default)
        assert_eq!(exit, check_exit_codes::VALID);
    }

    #[test]
    fn custom_config_with_strict_mode() {
        // Create config with restrictive tap timeout range
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "tap_timeout_warn_range = [100, 200]").unwrap();

        // Script with tap_timeout outside the range
        let script_file = create_script_file("tap_timeout(50);");

        let cmd = CheckCommand::new(script_file.path().to_path_buf(), OutputFormat::Human)
            .with_config(config_file.path().to_path_buf())
            .strict();
        let exit = cmd.run().unwrap();
        // Should fail in strict mode with warnings
        assert_eq!(exit, check_exit_codes::WARNINGS_STRICT);
    }

    #[test]
    fn coverage_includes_report() {
        let file = create_script_file(
            r#"
            remap("A", "B");
            block("C");
        "#,
        );
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).with_coverage();
        let exit = cmd.run().unwrap();
        assert_eq!(exit, check_exit_codes::VALID);
    }

    #[test]
    fn visual_includes_keyboard() {
        let file = create_script_file(r#"remap("A", "B");"#);
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).with_visual();
        let exit = cmd.run().unwrap();
        assert_eq!(exit, check_exit_codes::VALID);
    }

    #[test]
    fn no_warnings_suppresses_output() {
        let file = create_script_file(
            r#"
            remap("A", "B");
            remap("A", "C");
        "#,
        );
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).no_warnings();
        let exit = cmd.run().unwrap();
        assert_eq!(exit, check_exit_codes::VALID);
    }

    #[test]
    fn show_config_returns_valid() {
        let file = create_script_file("");
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human).show_config();
        let exit = cmd.run().unwrap();
        assert_eq!(exit, check_exit_codes::VALID);
    }

    #[test]
    fn invalid_config_path_errors() {
        let file = create_script_file(r#"remap("A", "B");"#);
        let cmd = CheckCommand::new(file.path().to_path_buf(), OutputFormat::Human)
            .with_config("/nonexistent/config.toml".into());
        let result = cmd.run();
        assert!(result.is_err());
    }
}

// =============================================================================
// JSON Output Tests
// =============================================================================

mod json_output {
    use super::*;

    #[test]
    fn json_output_is_parseable() {
        let script = r#"remap("CapsLock", "Escape");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["is_valid"], true);
        assert!(parsed["errors"].is_array());
        assert!(parsed["warnings"].is_array());
    }

    #[test]
    fn json_errors_include_code_and_message() {
        let script = r#"layer_push("undefined");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(!parsed["errors"].as_array().unwrap().is_empty());
        let first_error = &parsed["errors"][0];
        assert!(first_error["code"].is_string());
        assert!(first_error["message"].is_string());
    }

    #[test]
    fn json_warnings_include_category() {
        let script = r#"
            remap("A", "B");
            remap("A", "C");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(!parsed["warnings"].as_array().unwrap().is_empty());
        let first_warning = &parsed["warnings"][0];
        assert!(first_warning["category"].is_string());
    }

    #[test]
    fn json_coverage_when_requested() {
        let script = r#"remap("A", "B");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new().with_coverage());

        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["coverage"].is_object());
        assert!(parsed["coverage"]["remapped"].is_array());
        assert!(parsed["coverage"]["blocked"].is_array());
    }

    #[test]
    fn json_roundtrip_preserves_data() {
        let script = r#"
            remap("A", "B");
            block("C");
            layer_push("undefined");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new().with_coverage());

        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify data is preserved through JSON serialization
        assert_eq!(parsed["is_valid"], result.is_valid);
        assert_eq!(
            parsed["errors"].as_array().unwrap().len(),
            result.errors.len()
        );
        assert_eq!(
            parsed["warnings"].as_array().unwrap().len(),
            result.warnings.len()
        );
    }
}

// =============================================================================
// FFI Integration Tests
// =============================================================================

mod ffi_integration {
    #[allow(unused_imports)]
    use super::*;
    use keyrx_core::ffi::{keyrx_free_string, keyrx_validate_script};
    use std::ffi::{CStr, CString};
    use std::ptr;

    fn parse_ffi_response(ptr: *mut std::ffi::c_char) -> (bool, String) {
        assert!(!ptr.is_null());
        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        if let Some(payload) = raw.strip_prefix("ok:") {
            (true, payload.to_string())
        } else if let Some(msg) = raw.strip_prefix("error:") {
            (false, msg.to_string())
        } else {
            (false, raw)
        }
    }

    #[test]
    fn ffi_validates_valid_script() {
        let script = CString::new(r#"remap("CapsLock", "Escape");"#).unwrap();
        let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
        let (ok, payload) = parse_ffi_response(ptr);

        assert!(ok);
        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(result["is_valid"], true);
    }

    #[test]
    fn ffi_detects_errors() {
        let script = CString::new(r#"layer_push("undefined_layer");"#).unwrap();
        let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
        let (ok, payload) = parse_ffi_response(ptr);

        assert!(ok);
        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(result["is_valid"], false);
        assert!(!result["errors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn ffi_handles_null_pointer() {
        let ptr = unsafe { keyrx_validate_script(ptr::null()) };
        let (ok, _) = parse_ffi_response(ptr);
        assert!(!ok);
    }

    #[test]
    fn ffi_json_is_parseable() {
        let script = CString::new(
            r#"
            remap("A", "B");
            block("C");
        "#,
        )
        .unwrap();
        let ptr = unsafe { keyrx_validate_script(script.as_ptr()) };
        let (ok, payload) = parse_ffi_response(ptr);

        assert!(ok);
        let result: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(result["is_valid"], true);
        assert!(result["coverage"].is_object()); // FFI always includes coverage
    }
}

// =============================================================================
// Strict Mode Tests
// =============================================================================

mod strict_mode {
    use super::*;

    #[test]
    fn strict_mode_fails_on_any_warning() {
        let script = r#"
            remap("A", "B");
            remap("A", "C");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new().strict());

        assert!(!result.is_valid);
        assert!(result.has_warnings());
    }

    #[test]
    fn strict_mode_passes_without_warnings() {
        let script = r#"remap("A", "B");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new().strict());

        assert!(result.is_valid);
        assert!(!result.has_warnings());
    }

    #[test]
    fn no_warnings_option_suppresses_warnings() {
        let script = r#"
            remap("A", "B");
            remap("A", "C");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new().no_warnings());

        assert!(result.is_valid);
        assert!(!result.has_warnings());
    }
}

// =============================================================================
// Parse Error Tests
// =============================================================================

mod parse_errors {
    use super::*;

    #[test]
    fn detects_syntax_error() {
        let script = "this is not valid rhai {{{{";

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].code, "E000");
    }

    #[test]
    fn detects_unclosed_string() {
        let script = r#"remap("CapsLock, "Escape");"#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn detects_undefined_function() {
        let script = "nonexistent_function();";

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
}

// =============================================================================
// Real-World Script Tests
// =============================================================================

mod real_world_scripts {
    use super::*;

    #[test]
    fn capslock_to_escape_config() {
        let script = r#"
            // Classic CapsLock to Escape remap
            remap("CapsLock", "Escape");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn vim_style_navigation_layer() {
        let script = r#"
            // Vim-style navigation on a layer
            define_layer("nav");

            // HJKL navigation
            layer_map("nav", "H", "Left");
            layer_map("nav", "J", "Down");
            layer_map("nav", "K", "Up");
            layer_map("nav", "L", "Right");

            // Tap-hold for layer activation
            tap_hold("CapsLock", "Escape", "LeftCtrl");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn home_row_mods_config() {
        let script = r#"
            // Home row mods
            tap_hold("A", "A", "LeftAlt");
            tap_hold("S", "S", "LeftCtrl");
            tap_hold("D", "D", "LeftShift");
            tap_hold("F", "F", "LeftMeta");

            tap_hold("J", "J", "RightMeta");
            tap_hold("K", "K", "RightShift");
            tap_hold("L", "L", "RightCtrl");
            tap_hold("Semicolon", "Semicolon", "RightAlt");

            // Timing adjustments
            tap_timeout(200);
            hold_delay(150);
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn combo_based_shortcuts() {
        let script = r#"
            // Combo-based shortcuts
            combo(["A", "S"], "Tab");
            combo(["S", "D"], "Escape");
            combo(["D", "F"], "Enter");

            // Timing
            combo_timeout(50);
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn multi_layer_config() {
        let script = r#"
            // Multiple layers
            define_layer("nav");
            define_layer("symbols");
            define_layer("numbers");

            // Navigation layer
            layer_map("nav", "H", "Left");
            layer_map("nav", "J", "Down");
            layer_map("nav", "K", "Up");
            layer_map("nav", "L", "Right");

            // Custom modifier for layer switching
            define_modifier("layer_switch");
        "#;

        let engine = ValidationEngine::new();
        let result = engine.validate(script, ValidationOptions::new());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
}
