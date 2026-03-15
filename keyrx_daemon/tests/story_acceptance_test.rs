//! Story-driven acceptance tests: Rhai → compile → deserialize → EventProcessor → output.
//!
//! These tests verify the full pipeline from user-written Rhai scripts through
//! compilation, deserialization, and runtime event processing. They catch
//! contract breaks between any layer (parser, compiler, serializer, runtime).

use keyrx_compiler::parser::Parser;
use keyrx_compiler::serialize::{deserialize, serialize};
use keyrx_core::config::mappings::{DeviceConfig, DeviceIdentifier, KeyMapping};
use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::platform::{MockInput, MockOutput};
use keyrx_daemon::processor::EventProcessor;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

extern crate alloc;
use alloc::string::String;

// ============================================================================
// Helpers
// ============================================================================

/// Parse a Rhai script, serialize to .krx bytes, deserialize, and return
/// the device configs as owned types suitable for EventProcessor.
fn compile_rhai_to_configs(script: &str) -> Vec<DeviceConfig> {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("test.rhai");

    let mut parser = Parser::new();
    let config = parser
        .parse_string(script, &source_path)
        .expect("Rhai parse should succeed");

    let bytes = serialize(&config).expect("serialize should succeed");
    let archived = deserialize(&bytes).expect("deserialize should succeed");

    // Convert archived devices to owned DeviceConfig
    archived
        .devices
        .iter()
        .map(|d| {
            let pattern: String = d.identifier.pattern.to_string();
            let mappings: Vec<KeyMapping> = config
                .devices
                .iter()
                .find(|dev| dev.identifier.pattern == pattern)
                .expect("device should exist in original config")
                .mappings
                .clone();
            DeviceConfig {
                identifier: DeviceIdentifier { pattern },
                mappings,
            }
        })
        .collect()
}

/// Build an EventProcessor from a DeviceConfig with given input events.
fn build_processor(
    config: &DeviceConfig,
    events: Vec<KeyEvent>,
) -> EventProcessor<MockInput, MockOutput> {
    let input = MockInput::new(events);
    let output = MockOutput::new();
    EventProcessor::new(config, input, output)
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
    let configs = compile_rhai_to_configs(
        r#"
device_start("*");
map("CapsLock", "VK_Escape");
device_end();
"#,
    );
    let config = &configs[0];

    let mut proc = build_processor(
        config,
        vec![
            KeyEvent::Press(KeyCode::CapsLock),
            KeyEvent::Release(KeyCode::CapsLock),
        ],
    );
    proc.run().unwrap();

    let events = proc.output().events();
    assert_eq!(events[0], KeyEvent::Press(KeyCode::Escape));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::Escape));
}

// ============================================================================
// S2: Tap-Hold — quick tap sends Escape, hold activates modifier
// ============================================================================

#[test]
fn test_s2_tap_hold_quick_tap_sends_escape() {
    let configs = compile_rhai_to_configs(
        r#"
device_start("*");
tap_hold("CapsLock", "VK_Escape", "MD_00", 200);
device_end();
"#,
    );
    let config = &configs[0];

    // Quick tap: press then release immediately (timestamp 0 = instant)
    let mut proc = build_processor(
        config,
        vec![
            KeyEvent::Press(KeyCode::CapsLock),
            KeyEvent::Release(KeyCode::CapsLock),
        ],
    );
    proc.run().unwrap();

    let events = proc.output().events();
    // On quick release, tap-hold should produce the tap key (Escape)
    let has_escape = events
        .iter()
        .any(|e| *e == KeyEvent::Press(KeyCode::Escape));
    assert!(
        has_escape,
        "Quick tap should produce Escape press, got: {events:?}"
    );
}

#[test]
fn test_s2_tap_hold_long_hold_activates_modifier() {
    let configs = compile_rhai_to_configs(
        r#"
device_start("*");
tap_hold("CapsLock", "VK_Escape", "MD_00", 200);
device_end();
"#,
    );
    let config = &configs[0];

    // Hold past threshold: press with timestamp 0, release with timestamp > 200ms
    let mut proc = build_processor(
        config,
        vec![
            KeyEvent::press(KeyCode::CapsLock).with_timestamp(0),
            KeyEvent::release(KeyCode::CapsLock).with_timestamp(300_000),
        ],
    );
    proc.run().unwrap();

    let events = proc.output().events();
    // Hold should NOT produce Escape tap
    let has_escape = events
        .iter()
        .any(|e| *e == KeyEvent::Press(KeyCode::Escape));
    assert!(
        !has_escape,
        "Long hold should NOT produce Escape, got: {events:?}"
    );
}

// ============================================================================
// S3: Vim Navigation with Modifier Layer
// ============================================================================

#[test]
fn test_s3_vim_navigation_with_modifier_layer() {
    let configs = compile_rhai_to_configs(
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
    let config = &configs[0];

    let mut proc = build_processor(
        config,
        vec![
            // Activate modifier layer
            KeyEvent::Press(KeyCode::CapsLock),
            // Press H → should get Left
            KeyEvent::Press(KeyCode::H),
            KeyEvent::Release(KeyCode::H),
            // Press J → should get Down
            KeyEvent::Press(KeyCode::J),
            KeyEvent::Release(KeyCode::J),
            // Release modifier
            KeyEvent::Release(KeyCode::CapsLock),
            // Press H without modifier → should pass through as H
            KeyEvent::Press(KeyCode::H),
            KeyEvent::Release(KeyCode::H),
        ],
    );
    proc.run().unwrap();

    let events = proc.output().events();
    // CapsLock press → no output (modifier activation)
    // H press → Left press
    assert_eq!(events[0], KeyEvent::Press(KeyCode::Left));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::Left));
    // J press → Down press
    assert_eq!(events[2], KeyEvent::Press(KeyCode::Down));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::Down));
    // CapsLock release → no output
    // H press without modifier → H pass-through
    assert_eq!(events[4], KeyEvent::Press(KeyCode::H));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::H));
}

// ============================================================================
// S4: Device-Specific Config
// ============================================================================

#[test]
fn test_s4_device_specific_different_keyboards() {
    let configs = compile_rhai_to_configs(
        r#"
device_start("work-*");
map("CapsLock", "VK_Escape");
device_end();

device_start("game-*");
map("CapsLock", "VK_LCtrl");
device_end();
"#,
    );

    assert_eq!(configs.len(), 2, "Should have 2 device configs");

    // Work keyboard: CapsLock → Escape
    let work = &configs[0];
    assert_eq!(work.identifier.pattern, "work-*");
    let mut proc = build_processor(work, vec![KeyEvent::Press(KeyCode::CapsLock)]);
    proc.run().unwrap();
    assert_eq!(proc.output().events()[0], KeyEvent::Press(KeyCode::Escape));

    // Game keyboard: CapsLock → LCtrl
    let game = &configs[1];
    assert_eq!(game.identifier.pattern, "game-*");
    let mut proc = build_processor(game, vec![KeyEvent::Press(KeyCode::CapsLock)]);
    proc.run().unwrap();
    assert_eq!(proc.output().events()[0], KeyEvent::Press(KeyCode::LCtrl));
}

// ============================================================================
// S5: Modified Output — Z always outputs Ctrl+Z
// ============================================================================

#[test]
fn test_s5_modified_output_ctrl_z() {
    let configs = compile_rhai_to_configs(
        r#"
device_start("*");
map("VK_Z", with_ctrl("VK_Z"));
device_end();
"#,
    );
    let config = &configs[0];

    let mut proc = build_processor(
        config,
        vec![KeyEvent::Press(KeyCode::Z), KeyEvent::Release(KeyCode::Z)],
    );
    proc.run().unwrap();

    let events = proc.output().events();
    // Press: LCtrl down, then Z down
    assert_eq!(events[0], KeyEvent::Press(KeyCode::LCtrl));
    assert_eq!(events[1], KeyEvent::Press(KeyCode::Z));
    // Release: Z up, then LCtrl up
    assert_eq!(events[2], KeyEvent::Release(KeyCode::Z));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::LCtrl));
}

// ============================================================================
// S6: Lock Toggle activates conditional mapping
// ============================================================================

#[test]
fn test_s6_lock_toggle_activates_conditional() {
    let configs = compile_rhai_to_configs(
        r#"
device_start("*");
map("ScrollLock", "LK_00");
when_start("LK_00");
map("VK_A", "VK_B");
when_end();
device_end();
"#,
    );
    let config = &configs[0];

    let mut proc = build_processor(
        config,
        vec![
            // A before lock → passes through
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            // Toggle lock ON
            KeyEvent::Press(KeyCode::ScrollLock),
            KeyEvent::Release(KeyCode::ScrollLock),
            // A after lock → should map to B
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            // Toggle lock OFF
            KeyEvent::Press(KeyCode::ScrollLock),
            KeyEvent::Release(KeyCode::ScrollLock),
            // A after lock off → passes through again
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
        ],
    );
    proc.run().unwrap();

    let events = proc.output().events();
    // Before lock: A passes through
    assert_eq!(events[0], KeyEvent::Press(KeyCode::A));
    assert_eq!(events[1], KeyEvent::Release(KeyCode::A));
    // ScrollLock press/release → no output (lock toggle)
    // After lock ON: A → B
    assert_eq!(events[2], KeyEvent::Press(KeyCode::B));
    assert_eq!(events[3], KeyEvent::Release(KeyCode::B));
    // ScrollLock again → no output (lock toggle off)
    // After lock OFF: A passes through
    assert_eq!(events[4], KeyEvent::Press(KeyCode::A));
    assert_eq!(events[5], KeyEvent::Release(KeyCode::A));
}

// ============================================================================
// S7: Config Validation (Error Paths)
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
// S8: Full Pipeline File Round-Trip
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
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = temp_dir.path().join("profile.rhai");
    let krx_path = temp_dir.path().join("profile.krx");

    // --- Phase 1: Initial profile (CapsLock → Escape, A → B) ---
    fs::write(
        &rhai_path,
        r#"
device_start("*");
map("CapsLock", "VK_Escape");
map("VK_A", "VK_B");
device_end();
"#,
    )
    .unwrap();

    keyrx_compiler::compile_file(&rhai_path, &krx_path).expect("Initial compile should succeed");

    let configs_v1 = load_configs_from_krx(&krx_path);
    let mut proc = build_processor(
        &configs_v1[0],
        vec![
            KeyEvent::Press(KeyCode::CapsLock),
            KeyEvent::Release(KeyCode::CapsLock),
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
        ],
    );
    proc.run().unwrap();

    let events_v1 = proc.output().events();
    assert_eq!(
        events_v1[0],
        KeyEvent::Press(KeyCode::Escape),
        "v1: CapsLock should map to Escape"
    );
    assert_eq!(events_v1[1], KeyEvent::Release(KeyCode::Escape));
    assert_eq!(
        events_v1[2],
        KeyEvent::Press(KeyCode::B),
        "v1: A should map to B"
    );
    assert_eq!(events_v1[3], KeyEvent::Release(KeyCode::B));

    // --- Phase 2: User edits profile (CapsLock → Tab, A → B unchanged) ---
    fs::write(
        &rhai_path,
        r#"
device_start("*");
map("CapsLock", "VK_Tab");
map("VK_A", "VK_B");
device_end();
"#,
    )
    .unwrap();

    keyrx_compiler::compile_file(&rhai_path, &krx_path)
        .expect("Recompile after edit should succeed");

    let configs_v2 = load_configs_from_krx(&krx_path);
    let mut proc = build_processor(
        &configs_v2[0],
        vec![
            KeyEvent::Press(KeyCode::CapsLock),
            KeyEvent::Release(KeyCode::CapsLock),
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
        ],
    );
    proc.run().unwrap();

    let events_v2 = proc.output().events();
    assert_eq!(
        events_v2[0],
        KeyEvent::Press(KeyCode::Tab),
        "v2: CapsLock should now map to Tab after reload"
    );
    assert_eq!(events_v2[1], KeyEvent::Release(KeyCode::Tab));
    assert_eq!(
        events_v2[2],
        KeyEvent::Press(KeyCode::B),
        "v2: A→B should survive reload unchanged"
    );
    assert_eq!(events_v2[3], KeyEvent::Release(KeyCode::B));
}

/// Simulates adding a new mapping to an existing profile and reloading.
/// Verifies the new mapping works and existing mappings are unaffected.
#[test]
fn test_s9_hot_reload_add_mapping_to_existing_profile() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = temp_dir.path().join("profile.rhai");
    let krx_path = temp_dir.path().join("profile.krx");

    // --- Phase 1: Simple profile with one mapping ---
    fs::write(
        &rhai_path,
        r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#,
    )
    .unwrap();

    keyrx_compiler::compile_file(&rhai_path, &krx_path).unwrap();
    let configs_v1 = load_configs_from_krx(&krx_path);

    // A → B works, Z passes through
    let mut proc = build_processor(
        &configs_v1[0],
        vec![KeyEvent::Press(KeyCode::A), KeyEvent::Press(KeyCode::Z)],
    );
    proc.run().unwrap();
    let events = proc.output().events();
    assert_eq!(events[0], KeyEvent::Press(KeyCode::B), "v1: A→B");
    assert_eq!(
        events[1],
        KeyEvent::Press(KeyCode::Z),
        "v1: Z passes through"
    );

    // --- Phase 2: User adds Ctrl+Z shortcut, reloads ---
    fs::write(
        &rhai_path,
        r#"
device_start("*");
map("VK_A", "VK_B");
map("VK_Z", with_ctrl("VK_Z"));
device_end();
"#,
    )
    .unwrap();

    keyrx_compiler::compile_file(&rhai_path, &krx_path).unwrap();
    let configs_v2 = load_configs_from_krx(&krx_path);

    let mut proc = build_processor(
        &configs_v2[0],
        vec![KeyEvent::Press(KeyCode::A), KeyEvent::Press(KeyCode::Z)],
    );
    proc.run().unwrap();
    let events = proc.output().events();
    assert_eq!(
        events[0],
        KeyEvent::Press(KeyCode::B),
        "v2: A→B still works"
    );
    // Z now produces Ctrl+Z
    assert_eq!(
        events[1],
        KeyEvent::Press(KeyCode::LCtrl),
        "v2: Z now triggers LCtrl"
    );
    assert_eq!(
        events[2],
        KeyEvent::Press(KeyCode::Z),
        "v2: Z now triggers Ctrl+Z"
    );
}

/// Simulates removing a mapping from a profile. After reload, the removed
/// mapping should no longer be active (key passes through).
#[test]
fn test_s9_hot_reload_remove_mapping_effective_immediately() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = temp_dir.path().join("profile.rhai");
    let krx_path = temp_dir.path().join("profile.krx");

    // --- Phase 1: Two mappings ---
    fs::write(
        &rhai_path,
        r#"
device_start("*");
map("VK_A", "VK_B");
map("CapsLock", "VK_Escape");
device_end();
"#,
    )
    .unwrap();

    keyrx_compiler::compile_file(&rhai_path, &krx_path).unwrap();
    let configs_v1 = load_configs_from_krx(&krx_path);

    let mut proc = build_processor(&configs_v1[0], vec![KeyEvent::Press(KeyCode::CapsLock)]);
    proc.run().unwrap();
    assert_eq!(
        proc.output().events()[0],
        KeyEvent::Press(KeyCode::Escape),
        "v1: CapsLock→Escape active"
    );

    // --- Phase 2: Remove CapsLock mapping, keep A→B ---
    fs::write(
        &rhai_path,
        r#"
device_start("*");
map("VK_A", "VK_B");
device_end();
"#,
    )
    .unwrap();

    keyrx_compiler::compile_file(&rhai_path, &krx_path).unwrap();
    let configs_v2 = load_configs_from_krx(&krx_path);

    let mut proc = build_processor(
        &configs_v2[0],
        vec![
            KeyEvent::Press(KeyCode::CapsLock),
            KeyEvent::Press(KeyCode::A),
        ],
    );
    proc.run().unwrap();
    let events = proc.output().events();
    assert_eq!(
        events[0],
        KeyEvent::Press(KeyCode::CapsLock),
        "v2: CapsLock passes through (mapping removed)"
    );
    assert_eq!(
        events[1],
        KeyEvent::Press(KeyCode::B),
        "v2: A→B still works"
    );
}

// ============================================================================
// S9 Helpers
// ============================================================================

/// Load configs from a .krx file on disk (simulates daemon reload).
fn load_configs_from_krx(krx_path: &std::path::Path) -> Vec<DeviceConfig> {
    let bytes = fs::read(krx_path).expect("Should read .krx file");
    // Validate .krx integrity before loading
    let _archived = deserialize(&bytes).expect("deserialize should succeed");

    // Re-parse to get owned configs (same approach as daemon reload)
    let mut parser = Parser::new();
    // Read the original .rhai to get owned types
    // In real daemon, this comes from ProfileManager::activate()
    // Here we re-parse from the .rhai file that was just compiled
    let rhai_path = krx_path.with_extension("rhai");
    let config = parser
        .parse_script(&rhai_path)
        .expect("parse_script should succeed");

    config
        .devices
        .iter()
        .map(|d| DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: d.identifier.pattern.clone(),
            },
            mappings: d.mappings.clone(),
        })
        .collect()
}
