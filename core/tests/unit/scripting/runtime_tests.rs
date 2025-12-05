//! Unit tests for scripting::runtime module.

use keyrx_core::engine::{HoldAction, KeyCode, LayerAction, Modifier, RemapAction, TimingConfig};
use keyrx_core::scripting::cache::ScriptCache;
use keyrx_core::scripting::{PendingOp, ResourceEnforcer, ResourceLimits, RhaiRuntime};
use keyrx_core::ScriptRuntime;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn new_runtime_has_empty_registry() {
    let runtime = RhaiRuntime::new().unwrap();
    assert_eq!(runtime.lookup_remap(KeyCode::A), RemapAction::Pass);
}

#[test]
fn load_file_hits_cache_on_second_load() {
    let script_dir = tempdir().unwrap();
    let cache_dir = tempdir().unwrap();
    let script_path = script_dir.path().join("script.rhai");

    fs::write(
        &script_path,
        r#"
remap("A", "B");
"#,
    )
    .unwrap();

    let mut runtime =
        RhaiRuntime::with_cache(ScriptCache::new(cache_dir.path().to_path_buf())).unwrap();

    runtime
        .load_file(script_path.to_str().expect("utf-8 path"))
        .unwrap();
    let first_stats = runtime.script_cache().unwrap().stats();
    assert_eq!(first_stats.misses, 1);
    assert_eq!(first_stats.hits, 0);
    assert_eq!(first_stats.entries, 1);

    runtime
        .load_file(script_path.to_str().expect("utf-8 path"))
        .unwrap();
    let second_stats = runtime.script_cache().unwrap().stats();
    assert_eq!(second_stats.hits, 1);
    assert_eq!(second_stats.entries, 1);
}

#[test]
fn execute_remap_registers_mapping() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime.execute(r#"remap("A", "B");"#).unwrap();
    assert_eq!(
        runtime.lookup_remap(KeyCode::A),
        RemapAction::Remap(KeyCode::B)
    );
}

#[test]
fn execute_block_registers_block() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime.execute(r#"block("CapsLock");"#).unwrap();
    assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
}

#[test]
fn execute_pass_registers_pass() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime.execute(r#"remap("A", "B"); pass("A");"#).unwrap();
    assert_eq!(runtime.lookup_remap(KeyCode::A), RemapAction::Pass);
}

#[test]
fn unknown_key_returns_error() {
    let mut runtime = RhaiRuntime::new().unwrap();
    // Invalid keys should cause script errors
    let result = runtime.execute(r#"remap("InvalidKey", "B");"#);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("InvalidKey"));

    let result = runtime.execute(r#"remap("A", "InvalidKey");"#);
    assert!(result.is_err());

    let result = runtime.execute(r#"block("InvalidKey");"#);
    assert!(result.is_err());

    let result = runtime.execute(r#"pass("InvalidKey");"#);
    assert!(result.is_err());

    // Valid mappings should still work
    runtime.execute(r#"remap("C", "D");"#).unwrap();
    assert_eq!(
        runtime.lookup_remap(KeyCode::C),
        RemapAction::Remap(KeyCode::D)
    );
}

#[test]
fn errors_are_catchable_in_scripts() {
    let mut runtime = RhaiRuntime::new().unwrap();
    // Using try/catch in Rhai scripts should work
    let result = runtime.execute(
        r#"
            let caught = false;
            try {
                remap("InvalidKey", "B");
            } catch {
                caught = true;
            }
            // After catching the error, valid remaps should still work
            remap("A", "B");
            "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        runtime.lookup_remap(KeyCode::A),
        RemapAction::Remap(KeyCode::B)
    );
}

#[test]
fn multiple_remaps_work() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(
            r#"
            remap("A", "B");
            remap("C", "D");
            block("CapsLock");
        "#,
        )
        .unwrap();
    assert_eq!(
        runtime.lookup_remap(KeyCode::A),
        RemapAction::Remap(KeyCode::B)
    );
    assert_eq!(
        runtime.lookup_remap(KeyCode::C),
        RemapAction::Remap(KeyCode::D)
    );
    assert_eq!(runtime.lookup_remap(KeyCode::CapsLock), RemapAction::Block);
}

#[test]
fn execute_tap_hold_registers_binding() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(r#"tap_hold("CapsLock", "Escape", "LeftCtrl");"#)
        .unwrap();

    let binding = runtime.registry().tap_hold(KeyCode::CapsLock).unwrap();
    assert_eq!(binding.tap, KeyCode::Escape);
    assert_eq!(binding.hold, HoldAction::Key(KeyCode::LeftCtrl));
}

#[test]
fn execute_tap_hold_mod_registers_modifier_binding() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(r#"tap_hold_mod("CapsLock", "Escape", 2);"#)
        .unwrap();

    let binding = runtime.registry().tap_hold(KeyCode::CapsLock).unwrap();
    assert_eq!(binding.tap, KeyCode::Escape);
    assert_eq!(binding.hold, HoldAction::Modifier(2));
}

#[test]
fn tap_hold_mod_rejects_out_of_range_modifier() {
    let mut runtime = RhaiRuntime::new().unwrap();
    let result = runtime.execute(r#"tap_hold_mod("CapsLock", "Escape", 999);"#);
    assert!(result.is_err());
    assert!(runtime.registry().tap_hold(KeyCode::CapsLock).is_none());
}

#[test]
fn execute_combo_registers_definition() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime.execute(r#"combo(["A", "B"], "Escape");"#).unwrap();

    let action = runtime.registry().combos().find(&[KeyCode::A, KeyCode::B]);
    assert_eq!(action, Some(&LayerAction::Remap(KeyCode::Escape)));
}

#[test]
fn combo_requires_between_two_and_four_keys() {
    let mut runtime = RhaiRuntime::new().unwrap();
    assert!(runtime.execute(r#"combo(["A"], "Escape");"#).is_err());
    assert!(runtime
        .execute(r#"combo(["A","B","C","D","E"], "Escape");"#)
        .is_err());
}

#[test]
fn combo_rejects_non_string_keys() {
    let mut runtime = RhaiRuntime::new().unwrap();
    let err = runtime.execute(r#"combo([1, "B"], "Escape");"#);
    assert!(err.is_err());
}

#[test]
fn timing_functions_update_config() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(
            r#"
                set_tap_timeout(350);
                set_combo_timeout(75);
                set_hold_delay(10);
                set_eager_tap(true);
                set_permissive_hold(false);
                set_retro_tap(true);
            "#,
        )
        .unwrap();

    let timing = runtime.registry().timing_config();
    assert_eq!(timing.tap_timeout_ms, 350);
    assert_eq!(timing.combo_timeout_ms, 75);
    assert_eq!(timing.hold_delay_ms, 10);
    assert!(timing.eager_tap);
    assert!(!timing.permissive_hold);
    assert!(timing.retro_tap);
}

#[test]
fn timing_functions_validate_ranges() {
    let mut runtime = RhaiRuntime::new().unwrap();
    assert!(runtime.execute(r#"set_tap_timeout(0);"#).is_err());
    assert!(runtime.execute(r#"set_combo_timeout(6000);"#).is_err());
    assert!(runtime.execute(r#"set_hold_delay(-1);"#).is_err());

    let timing = runtime.registry().timing_config();
    assert_eq!(*timing, TimingConfig::default());
}

#[test]
fn layer_functions_apply_and_query() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(
            r#"
                layer_define("nav", true);
                layer_map("nav", "A", "Escape");
                layer_push("nav");
                if !is_layer_active("nav") {
                    throw "nav should be active";
                }
            "#,
        )
        .unwrap();

    let registry = runtime.registry();
    assert!(registry.is_layer_active("nav").unwrap());
    let action = registry.layers().lookup(KeyCode::A);
    assert_eq!(action, Some(&LayerAction::Remap(KeyCode::Escape)));
}

#[test]
fn layer_map_supports_toggle_actions() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(
            r#"
                layer_define("nav", false);
                layer_define("fn", false);
                layer_map("nav", "B", "layer_toggle:fn");
            "#,
        )
        .unwrap();

    runtime.execute(r#"layer_push("nav");"#).unwrap();
    let nav_id = runtime.registry().layer_id("nav").unwrap();
    let fn_id = runtime.registry().layer_id("fn").unwrap();

    assert_eq!(
        runtime.registry().layers().lookup(KeyCode::B),
        Some(&LayerAction::LayerToggle(fn_id))
    );
    assert_eq!(nav_id, 1);
    assert_eq!(fn_id, 2);
}

#[test]
fn define_modifier_and_activate_it() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(
            r#"
                let id = define_modifier("hyper");
                if id != 0 { throw "unexpected modifier id"; }
                modifier_on("hyper");
            "#,
        )
        .unwrap();

    let registry = runtime.registry();
    let id = registry.modifier_id("hyper").unwrap();
    assert_eq!(id, 0);
    assert!(registry.modifier_state().is_active(Modifier::Virtual(id)));
}

#[test]
fn one_shot_marks_modifier_as_active_once() {
    let mut runtime = RhaiRuntime::new().unwrap();
    runtime
        .execute(
            r#"
                define_modifier("hyper");
                one_shot("hyper");
            "#,
        )
        .unwrap();

    let registry = runtime.registry();
    let id = registry.modifier_id("hyper").unwrap();
    let mut snapshot = registry.modifier_state();
    assert!(snapshot.is_active(Modifier::Virtual(id)));
    assert!(snapshot.consume_one_shot(Modifier::Virtual(id)));
    assert!(!snapshot.is_active(Modifier::Virtual(id)));
}

#[test]
fn modifier_functions_require_definition() {
    let mut runtime = RhaiRuntime::new().unwrap();
    assert!(runtime.execute(r#"modifier_on("hyper");"#).is_err());
    assert!(runtime.execute(r#"modifier_off("hyper");"#).is_err());
    assert!(runtime.execute(r#"one_shot("hyper");"#).is_err());
    assert!(runtime.execute(r#"is_modifier_active("hyper");"#).is_err());
}
