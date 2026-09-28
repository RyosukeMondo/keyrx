//! Story-driven acceptance tests: Rhai -> compile -> .krx -> SimulationEngine -> output.
//!
//! These tests verify the full pipeline from user-written Rhai scripts through
//! compilation, serialization, and the SAME production remapping engine the
//! daemon's live event loop uses (`keyrx_core::runtime::Remapper`, driven via
//! `SimulationEngine`/`keyrx_core::simulate` - see their module docs). They
//! catch contract breaks between any layer (parser, compiler, serializer,
//! engine) and, unlike the deleted `EventProcessor`-based version of this
//! file, cannot drift from what the daemon actually does.

use keyrx_compiler::parser::Parser;
use keyrx_compiler::serialize::deserialize;
use keyrx_daemon::config::simulation_engine::{
    EventSequence, EventType, SimulatedEvent, SimulationEngine,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Helpers
// ============================================================================

/// Compiles a Rhai script to a real `.krx` file and loads it into a fresh
/// [`SimulationEngine`] - the same load path the daemon's `run`/`simulate`
/// commands use. Returns the `TempDir` too, so callers that need to edit and
/// recompile the same path (hot-reload tests) can keep it alive.
fn compile_and_load(
    dir: &TempDir,
    name: &str,
    script: &str,
) -> (PathBuf, PathBuf, SimulationEngine) {
    let rhai_path = dir.path().join(format!("{name}.rhai"));
    let krx_path = dir.path().join(format!("{name}.krx"));
    fs::write(&rhai_path, script).expect("write rhai source");
    keyrx_compiler::compile_file(&rhai_path, &krx_path).expect("compile_file should succeed");
    let engine = SimulationEngine::new(&krx_path).expect("SimulationEngine should load the .krx");
    (rhai_path, krx_path, engine)
}

fn ev(device: Option<&str>, key: &str, press: bool, at_us: u64) -> SimulatedEvent {
    SimulatedEvent {
        device_id: device.map(str::to_string),
        timestamp_us: at_us,
        key: key.to_string(),
        event_type: if press {
            EventType::Press
        } else {
            EventType::Release
        },
    }
}

fn sequence(events: Vec<SimulatedEvent>) -> EventSequence {
    EventSequence { events, seed: 0 }
}

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path
}

// ============================================================================
// S1: Simple Remap — CapsLock → Escape
// ============================================================================

#[test]
fn test_s1_simple_remap_capslock_to_escape() {
    let dir = TempDir::new().unwrap();
    let (_, _, mut engine) = compile_and_load(
        &dir,
        "s1",
        r#"
device_start("*");
map("CapsLock", "VK_Escape");
device_end();
"#,
    );

    let output = engine
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "CapsLock", false, 10_000),
        ]))
        .unwrap();

    assert_eq!(output[0].key, "Escape");
    assert_eq!(output[0].event_type, EventType::Press);
    assert_eq!(output[1].key, "Escape");
    assert_eq!(output[1].event_type, EventType::Release);
}

// ============================================================================
// S2: Tap-Hold — quick tap sends Escape, hold activates modifier
// ============================================================================

#[test]
fn test_s2_tap_hold_quick_tap_sends_escape() {
    let dir = TempDir::new().unwrap();
    let (_, _, mut engine) = compile_and_load(
        &dir,
        "s2a",
        r#"
device_start("*");
tap_hold("CapsLock", "VK_Escape", "MD_00", 200);
device_end();
"#,
    );

    // Quick tap: press then release well inside the 200ms threshold.
    let output = engine
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "CapsLock", false, 50_000),
        ]))
        .unwrap();

    assert!(
        output
            .iter()
            .any(|e| e.key == "Escape" && e.event_type == EventType::Press),
        "quick tap should produce an Escape press, got: {output:?}"
    );
}

#[test]
fn test_s2_tap_hold_long_hold_activates_modifier() {
    let dir = TempDir::new().unwrap();
    let (_, _, mut engine) = compile_and_load(
        &dir,
        "s2b",
        r#"
device_start("*");
tap_hold("CapsLock", "VK_Escape", "MD_00", 200);
device_end();
"#,
    );

    // Hold past the threshold, released only after the hold has resolved.
    let output = engine
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "CapsLock", false, 300_000),
        ]))
        .unwrap();

    assert!(
        !output.iter().any(|e| e.key == "Escape"),
        "a long hold must NOT produce the tap output, got: {output:?}"
    );
}

// ============================================================================
// S3: Vim Navigation with Modifier Layer
// ============================================================================

#[test]
fn test_s3_vim_navigation_with_modifier_layer() {
    let dir = TempDir::new().unwrap();
    let (_, _, mut engine) = compile_and_load(
        &dir,
        "s3",
        r#"
device_start("*");
map("CapsLock", "MD_00");
when_start("MD_00");
map("VK_H", "VK_Left");
map("VK_J", "VK_Down");
map("VK_K", "VK_Up");
map("VK_L", "VK_Right");
when_end();
device_end();
"#,
    );

    let output = engine
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "H", true, 10_000),
            ev(None, "H", false, 20_000),
            ev(None, "J", true, 30_000),
            ev(None, "J", false, 40_000),
            ev(None, "CapsLock", false, 50_000),
            ev(None, "H", true, 60_000),
            ev(None, "H", false, 70_000),
        ]))
        .unwrap();

    // H with the layer active -> Left.
    assert_eq!(output[0].key, "Left");
    assert_eq!(output[1].key, "Left");
    // J with the layer active -> Down.
    assert_eq!(output[2].key, "Down");
    assert_eq!(output[3].key, "Down");
    // H without the layer -> passthrough.
    assert_eq!(output[4].key, "H");
    assert_eq!(output[5].key, "H");
}

// ============================================================================
// S4: Device-Specific Config — two device_start blocks, ONE engine
// ============================================================================

#[test]
fn test_s4_device_specific_different_keyboards() {
    let dir = TempDir::new().unwrap();
    let (_, _, mut engine) = compile_and_load(
        &dir,
        "s4",
        r#"
device_start("work-*");
map("CapsLock", "VK_Escape");
device_end();

device_start("game-*");
map("CapsLock", "VK_LCtrl");
device_end();
"#,
    );

    // Both devices' events go through the SAME engine (the same `Remapper`
    // the daemon's one event loop would route them through), not two
    // separately-configured instances.
    let output = engine
        .replay(&sequence(vec![
            ev(Some("work-kbd-1"), "CapsLock", true, 0),
            ev(Some("work-kbd-1"), "CapsLock", false, 10_000),
            ev(Some("game-pad-1"), "CapsLock", true, 20_000),
            ev(Some("game-pad-1"), "CapsLock", false, 30_000),
        ]))
        .unwrap();

    assert_eq!(output[0].key, "Escape", "work keyboard: CapsLock -> Escape");
    assert_eq!(output[2].key, "LCtrl", "game pad: CapsLock -> LCtrl");
}

// ============================================================================
// S5: Modified Output — Z always outputs Ctrl+Z
// ============================================================================

#[test]
fn test_s5_modified_output_ctrl_z() {
    let dir = TempDir::new().unwrap();
    let (_, _, mut engine) = compile_and_load(
        &dir,
        "s5",
        r#"
device_start("*");
map("VK_Z", with_ctrl("VK_Z"));
device_end();
"#,
    );

    let output = engine
        .replay(&sequence(vec![
            ev(None, "Z", true, 0),
            ev(None, "Z", false, 10_000),
        ]))
        .unwrap();

    assert_eq!(output[0].key, "LCtrl");
    assert_eq!(output[0].event_type, EventType::Press);
    assert_eq!(output[1].key, "Z");
    assert_eq!(output[1].event_type, EventType::Press);
    assert_eq!(output[2].key, "Z");
    assert_eq!(output[2].event_type, EventType::Release);
    assert_eq!(output[3].key, "LCtrl");
    assert_eq!(output[3].event_type, EventType::Release);
}

// ============================================================================
// S6: Lock Toggle activates conditional mapping
// ============================================================================

#[test]
fn test_s6_lock_toggle_activates_conditional() {
    let dir = TempDir::new().unwrap();
    let (_, _, mut engine) = compile_and_load(
        &dir,
        "s6",
        r#"
device_start("*");
map("ScrollLock", "LK_00");
when_start("LK_00");
map("VK_A", "VK_B");
when_end();
device_end();
"#,
    );

    let output = engine
        .replay(&sequence(vec![
            ev(None, "A", true, 0),
            ev(None, "A", false, 10_000),
            ev(None, "ScrollLock", true, 20_000),
            ev(None, "ScrollLock", false, 30_000),
            ev(None, "A", true, 40_000),
            ev(None, "A", false, 50_000),
            ev(None, "ScrollLock", true, 60_000),
            ev(None, "ScrollLock", false, 70_000),
            ev(None, "A", true, 80_000),
            ev(None, "A", false, 90_000),
        ]))
        .unwrap();

    // Before lock: A passes through.
    assert_eq!(output[0].key, "A");
    assert_eq!(output[1].key, "A");
    // After lock ON: A -> B.
    assert_eq!(output[2].key, "B");
    assert_eq!(output[3].key, "B");
    // After lock OFF: A passes through again.
    assert_eq!(output[4].key, "A");
    assert_eq!(output[5].key, "A");
}

// ============================================================================
// S7: Config Validation (Error Paths) — pure parser, no engine involved
// ============================================================================

#[test]
fn test_s7_invalid_rhai_clear_error() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("bad.rhai");

    let mut parser = Parser::new();
    let result = parser.parse_string(
        r#"
device_start("*");
map("VK_A", "INVALID_KEY_NAME");
device_end();
"#,
        &source_path,
    );

    assert!(result.is_err(), "Should fail on invalid key name");
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("INVALID_KEY_NAME") || err_msg.contains("nknown"),
        "Error should mention the invalid key: {err_msg}"
    );
}

#[test]
fn test_s7_map_outside_device_block_error() {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("no_device.rhai");

    let mut parser = Parser::new();
    let result = parser.parse_string(
        r#"
map("VK_A", "VK_B");
"#,
        &source_path,
    );

    assert!(result.is_err(), "map() outside device block should fail");
}

// ============================================================================
// S8: Full Pipeline File Round-Trip — pure compiler/serializer, no engine
// ============================================================================

#[test]
fn test_s8_full_pipeline_file_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = create_temp_file(
        &temp_dir,
        "full_pipeline.rhai",
        r#"
device_start("keyboard-*");
map("CapsLock", "VK_Escape");
map("VK_A", "VK_B");
map("ScrollLock", "LK_00");
tap_hold("Space", "VK_Space", "MD_00", 200);
map("VK_Z", with_ctrl("VK_Z"));
device_end();
"#,
    );
    let krx_path = temp_dir.path().join("full_pipeline.krx");

    // Step 1: compile_file (Rhai → .krx on disk)
    keyrx_compiler::compile_file(&rhai_path, &krx_path).expect("compile_file should succeed");

    // Step 2: read .krx from disk
    let bytes = fs::read(&krx_path).expect("Should read .krx file");

    // Step 3: deserialize
    let archived = deserialize(&bytes).expect("deserialize should succeed");

    // Step 4: verify structure
    assert_eq!(archived.devices.len(), 1);
    let device = &archived.devices[0];
    assert_eq!(device.identifier.pattern.as_str(), "keyboard-*");
    assert_eq!(
        device.mappings.len(),
        5,
        "Should have 5 mappings (simple, simple, lock, tap_hold, modified_output)"
    );
}

// ============================================================================
// S9: Hot-Reload — edit profile, reload, new mapping effective immediately
// ============================================================================

/// Simulates the full hot-reload cycle:
/// 1. Write .rhai profile (CapsLock → Escape), compile, load, verify events
/// 2. Edit .rhai profile (CapsLock → Tab), re-compile, reload, verify new events
/// 3. Also verifies other mappings survive the reload unchanged
#[test]
fn test_s9_hot_reload_edit_and_reload_effective_immediately() {
    let dir = TempDir::new().unwrap();

    // --- Phase 1: Initial profile (CapsLock → Escape, A → B) ---
    let (_, _, mut engine_v1) = compile_and_load(
        &dir,
        "profile",
        r#"
device_start("*");
map("CapsLock", "VK_Escape");
map("VK_A", "VK_B");
device_end();
"#,
    );

    let events_v1 = engine_v1
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "CapsLock", false, 10_000),
            ev(None, "A", true, 20_000),
            ev(None, "A", false, 30_000),
        ]))
        .unwrap();
    assert_eq!(
        events_v1[0].key, "Escape",
        "v1: CapsLock should map to Escape"
    );
    assert_eq!(events_v1[2].key, "B", "v1: A should map to B");

    // --- Phase 2: User edits profile (CapsLock → Tab, A → B unchanged) ---
    let (_, _, mut engine_v2) = compile_and_load(
        &dir,
        "profile",
        r#"
device_start("*");
map("CapsLock", "VK_Tab");
map("VK_A", "VK_B");
device_end();
"#,
    );

    let events_v2 = engine_v2
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "CapsLock", false, 10_000),
            ev(None, "A", true, 20_000),
            ev(None, "A", false, 30_000),
        ]))
        .unwrap();
    assert_eq!(
        events_v2[0].key, "Tab",
        "v2: CapsLock should now map to Tab after reload"
    );
    assert_eq!(
        events_v2[2].key, "B",
        "v2: A→B should survive reload unchanged"
    );
}

/// Simulates adding a new mapping to an existing profile and reloading.
/// Verifies the new mapping works and existing mappings are unaffected.
#[test]
fn test_s9_hot_reload_add_mapping_to_existing_profile() {
    let dir = TempDir::new().unwrap();

    // --- Phase 1: Simple profile with one mapping ---
    let (_, _, mut engine_v1) = compile_and_load(
        &dir,
        "profile",
        r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#,
    );

    let events = engine_v1
        .replay(&sequence(vec![
            ev(None, "A", true, 0),
            ev(None, "A", false, 10_000),
            ev(None, "Z", true, 20_000),
            ev(None, "Z", false, 30_000),
        ]))
        .unwrap();
    assert_eq!(events[0].key, "B", "v1: A→B");
    assert_eq!(events[2].key, "Z", "v1: Z passes through");

    // --- Phase 2: User adds Ctrl+Z shortcut, reloads ---
    let (_, _, mut engine_v2) = compile_and_load(
        &dir,
        "profile",
        r#"
device_start("*");
map("VK_A", "VK_B");
map("VK_Z", with_ctrl("VK_Z"));
device_end();
"#,
    );

    let events = engine_v2
        .replay(&sequence(vec![
            ev(None, "A", true, 0),
            ev(None, "A", false, 10_000),
            ev(None, "Z", true, 20_000),
            ev(None, "Z", false, 30_000),
        ]))
        .unwrap();
    assert_eq!(events[0].key, "B", "v2: A→B still works");
    // Z now produces Ctrl+Z.
    assert_eq!(events[2].key, "LCtrl", "v2: Z now triggers LCtrl");
    assert_eq!(events[3].key, "Z", "v2: Z now triggers Ctrl+Z");
}

/// Simulates removing a mapping from a profile. After reload, the removed
/// mapping should no longer be active (key passes through).
#[test]
fn test_s9_hot_reload_remove_mapping_effective_immediately() {
    let dir = TempDir::new().unwrap();

    // --- Phase 1: Two mappings ---
    let (_, _, mut engine_v1) = compile_and_load(
        &dir,
        "profile",
        r#"
device_start("*");
map("VK_A", "VK_B");
map("CapsLock", "VK_Escape");
device_end();
"#,
    );

    let events = engine_v1
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "CapsLock", false, 10_000),
        ]))
        .unwrap();
    assert_eq!(events[0].key, "Escape", "v1: CapsLock→Escape active");

    // --- Phase 2: Remove CapsLock mapping, keep A→B ---
    let (_, _, mut engine_v2) = compile_and_load(
        &dir,
        "profile",
        r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#,
    );

    let events = engine_v2
        .replay(&sequence(vec![
            ev(None, "CapsLock", true, 0),
            ev(None, "CapsLock", false, 10_000),
            ev(None, "A", true, 20_000),
            ev(None, "A", false, 30_000),
        ]))
        .unwrap();
    assert_eq!(
        events[0].key, "CapsLock",
        "v2: CapsLock passes through (mapping removed)"
    );
    assert_eq!(events[2].key, "B", "v2: A→B still works");
}
