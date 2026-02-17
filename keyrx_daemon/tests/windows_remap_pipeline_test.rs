//! Integration tests for the full Windows remap pipeline
//!
//! Tests the complete event flow: input → remap → inject output
//! using a capturing mock platform. Exercises the same code path as
//! the real daemon (process_one_event) with the default profile config.
//!
//! Run with: cargo test -p keyrx_daemon --test windows_remap_pipeline

use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::daemon::event_loop::process_one_event;
use keyrx_daemon::daemon::remapping_state::RemappingState;
use keyrx_daemon::platform::{DeviceInfo, Platform, PlatformError, PlatformResult};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Mock platform that feeds events from a queue and captures injected output
struct CapturingPlatform {
    input_queue: VecDeque<KeyEvent>,
    injected: Arc<Mutex<Vec<KeyEvent>>>,
}

impl CapturingPlatform {
    fn new(events: Vec<KeyEvent>) -> (Self, Arc<Mutex<Vec<KeyEvent>>>) {
        let injected = Arc::new(Mutex::new(Vec::new()));
        let platform = Self {
            input_queue: VecDeque::from(events),
            injected: injected.clone(),
        };
        (platform, injected)
    }
}

impl Platform for CapturingPlatform {
    fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }

    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        self.input_queue
            .pop_front()
            .ok_or_else(|| PlatformError::DeviceNotFound("No more events".to_string()))
    }

    fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()> {
        self.injected.lock().unwrap().push(event);
        Ok(())
    }

    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        Ok(vec![])
    }

    fn shutdown(&mut self) -> PlatformResult<()> {
        Ok(())
    }
}

/// Build the default profile DeviceConfig (matches default.rhai)
#[allow(clippy::vec_init_then_push)]
fn default_profile_config() -> DeviceConfig {
    let mut m: Vec<KeyMapping> = Vec::new();

    // TapHold modifiers
    m.push(KeyMapping::tap_hold(KeyCode::B, KeyCode::Enter, 0, 200));
    m.push(KeyMapping::tap_hold(KeyCode::V, KeyCode::Delete, 1, 200));
    m.push(KeyMapping::tap_hold(KeyCode::M, KeyCode::Backspace, 2, 200));
    m.push(KeyMapping::tap_hold(KeyCode::X, KeyCode::Delete, 3, 200));
    m.push(KeyMapping::tap_hold(KeyCode::Num1, KeyCode::Num1, 4, 200));
    m.push(KeyMapping::tap_hold(KeyCode::LCtrl, KeyCode::Space, 5, 200));
    m.push(KeyMapping::tap_hold(KeyCode::C, KeyCode::Delete, 6, 200));
    m.push(KeyMapping::tap_hold(KeyCode::Tab, KeyCode::Space, 7, 200));
    m.push(KeyMapping::tap_hold(KeyCode::Q, KeyCode::Minus, 8, 200));
    m.push(KeyMapping::tap_hold(KeyCode::A, KeyCode::Tab, 9, 200));
    m.push(KeyMapping::tap_hold(KeyCode::N, KeyCode::N, 10, 200));

    // MD_00 layer (B hold)
    m.push(KeyMapping::conditional(
        Condition::ModifierActive(0),
        vec![
            BaseKeyMapping::Simple {
                from: KeyCode::Num2,
                to: KeyCode::Left,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num3,
                to: KeyCode::Right,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num4,
                to: KeyCode::Down,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Num5,
                to: KeyCode::Up,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::W,
                to: KeyCode::Num1,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::E,
                to: KeyCode::Num2,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::R,
                to: KeyCode::Num3,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::T,
                to: KeyCode::Num4,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Y,
                to: KeyCode::Num5,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::U,
                to: KeyCode::Num6,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::I,
                to: KeyCode::Num7,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::O,
                to: KeyCode::Num8,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::P,
                to: KeyCode::Num9,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::LeftBracket,
                to: KeyCode::Num0,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::S,
                to: KeyCode::F1,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::D,
                to: KeyCode::F2,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::F,
                to: KeyCode::F3,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::G,
                to: KeyCode::F4,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::F5,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::F6,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::K,
                to: KeyCode::F7,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::L,
                to: KeyCode::F8,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Semicolon,
                to: KeyCode::F9,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Quote,
                to: KeyCode::F10,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::Space,
                to: KeyCode::F12,
            },
        ],
    ));

    // Modified output: 2 -> Shift+7
    m.push(KeyMapping::modified_output(
        KeyCode::Num2,
        KeyCode::Num7,
        true,
        false,
        false,
        false,
    ));

    // Simple remaps: punctuation
    m.push(KeyMapping::simple(KeyCode::LeftBracket, KeyCode::S));
    m.push(KeyMapping::simple(KeyCode::RightBracket, KeyCode::Minus));
    m.push(KeyMapping::simple(KeyCode::Quote, KeyCode::Z));
    m.push(KeyMapping::simple(KeyCode::Equal, KeyCode::Slash));

    // Letter row (Dvorak-like)
    m.push(KeyMapping::simple(KeyCode::D, KeyCode::Q));
    m.push(KeyMapping::simple(KeyCode::E, KeyCode::O));
    m.push(KeyMapping::simple(KeyCode::F, KeyCode::J));
    m.push(KeyMapping::simple(KeyCode::G, KeyCode::K));
    m.push(KeyMapping::simple(KeyCode::H, KeyCode::X));
    m.push(KeyMapping::simple(KeyCode::I, KeyCode::H));
    m.push(KeyMapping::simple(KeyCode::J, KeyCode::B));
    m.push(KeyMapping::simple(KeyCode::K, KeyCode::M));
    m.push(KeyMapping::simple(KeyCode::L, KeyCode::W));
    m.push(KeyMapping::simple(KeyCode::O, KeyCode::T));
    m.push(KeyMapping::simple(KeyCode::P, KeyCode::N));
    m.push(KeyMapping::simple(KeyCode::R, KeyCode::E));
    m.push(KeyMapping::simple(KeyCode::S, KeyCode::Semicolon));
    m.push(KeyMapping::simple(KeyCode::T, KeyCode::U));
    m.push(KeyMapping::simple(KeyCode::U, KeyCode::D));
    m.push(KeyMapping::simple(KeyCode::W, KeyCode::A));
    m.push(KeyMapping::simple(KeyCode::Y, KeyCode::I));

    // Number row
    m.push(KeyMapping::simple(KeyCode::Num3, KeyCode::Comma));
    m.push(KeyMapping::simple(KeyCode::Num4, KeyCode::Period));
    m.push(KeyMapping::simple(KeyCode::Num5, KeyCode::P));
    m.push(KeyMapping::simple(KeyCode::Num6, KeyCode::Y));
    m.push(KeyMapping::simple(KeyCode::Num7, KeyCode::F));
    m.push(KeyMapping::simple(KeyCode::Num8, KeyCode::G));
    m.push(KeyMapping::simple(KeyCode::Num9, KeyCode::C));
    m.push(KeyMapping::simple(KeyCode::Num0, KeyCode::R));

    // Function keys
    m.push(KeyMapping::simple(KeyCode::F1, KeyCode::LMeta));
    m.push(KeyMapping::simple(KeyCode::F2, KeyCode::Escape));
    m.push(KeyMapping::simple(KeyCode::F3, KeyCode::LCtrl));
    m.push(KeyMapping::simple(KeyCode::F4, KeyCode::LAlt));
    m.push(KeyMapping::simple(KeyCode::F5, KeyCode::Backspace));
    m.push(KeyMapping::simple(KeyCode::F6, KeyCode::Delete));
    m.push(KeyMapping::simple(KeyCode::F8, KeyCode::Tab));
    m.push(KeyMapping::simple(KeyCode::F9, KeyCode::Tab));
    m.push(KeyMapping::simple(KeyCode::F10, KeyCode::Tab));
    m.push(KeyMapping::simple(KeyCode::F11, KeyCode::Tab));
    m.push(KeyMapping::simple(KeyCode::F12, KeyCode::Tab));

    // Special keys
    m.push(KeyMapping::simple(KeyCode::Escape, KeyCode::Num5));
    m.push(KeyMapping::simple(KeyCode::Backspace, KeyCode::Delete));
    m.push(KeyMapping::simple(KeyCode::Delete, KeyCode::Num4));
    m.push(KeyMapping::simple(KeyCode::Enter, KeyCode::Yen));

    // Modifier keys
    m.push(KeyMapping::simple(KeyCode::LAlt, KeyCode::LCtrl));

    // Punctuation
    m.push(KeyMapping::simple(KeyCode::Minus, KeyCode::L));
    m.push(KeyMapping::simple(KeyCode::Semicolon, KeyCode::V));
    m.push(KeyMapping::simple(KeyCode::Comma, KeyCode::F9));
    m.push(KeyMapping::simple(KeyCode::Period, KeyCode::F10));
    m.push(KeyMapping::simple(KeyCode::Slash, KeyCode::F11));
    m.push(KeyMapping::simple(KeyCode::Ro, KeyCode::F12));
    m.push(KeyMapping::simple(KeyCode::Zenkaku, KeyCode::Escape));

    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: String::from("*"),
        },
        mappings: m,
    }
}

/// Helper: run a press+release through the full pipeline, return injected events
fn run_press_release(key: KeyCode, remap: &mut RemappingState) -> Vec<KeyEvent> {
    let events = vec![KeyEvent::Press(key), KeyEvent::Release(key)];
    let (platform, injected) = CapturingPlatform::new(events);
    let mut boxed: Box<dyn Platform> = Box::new(platform);

    while let Ok(true) = process_one_event(&mut boxed, None, Some(remap), None) {}

    let result = injected.lock().unwrap().clone();
    result
}

/// Helper: run events through pipeline, return injected output
fn run_events(events: Vec<KeyEvent>, remap: &mut RemappingState) -> Vec<KeyEvent> {
    let (platform, injected) = CapturingPlatform::new(events);
    let mut boxed: Box<dyn Platform> = Box::new(platform);

    while let Ok(true) = process_one_event(&mut boxed, None, Some(remap), None) {}

    let result = injected.lock().unwrap().clone();
    result
}

/// Assert a simple remap through the full pipeline
fn assert_pipeline_remap(from: KeyCode, to: KeyCode, remap: &mut RemappingState, label: &str) {
    let output = run_press_release(from, remap);
    let press_keys: Vec<KeyCode> = output
        .iter()
        .filter(|e| e.is_press())
        .map(|e| e.keycode())
        .collect();
    assert!(
        press_keys.contains(&to),
        "{}: press {:?} should inject {:?}, got press keycodes: {:?}",
        label,
        from,
        to,
        press_keys
    );
}

// ============================================================
// Pipeline tests: full process_one_event with default profile
// ============================================================

#[test]
fn test_pipeline_all_letter_remaps() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let cases = [
        (KeyCode::D, KeyCode::Q, "D→Q"),
        (KeyCode::E, KeyCode::O, "E→O"),
        (KeyCode::F, KeyCode::J, "F→J"),
        (KeyCode::G, KeyCode::K, "G→K"),
        (KeyCode::H, KeyCode::X, "H→X"),
        (KeyCode::I, KeyCode::H, "I→H"),
        (KeyCode::J, KeyCode::B, "J→B"),
        (KeyCode::K, KeyCode::M, "K→M"),
        (KeyCode::L, KeyCode::W, "L→W"),
        (KeyCode::O, KeyCode::T, "O→T"),
        (KeyCode::P, KeyCode::N, "P→N"),
        (KeyCode::R, KeyCode::E, "R→E"),
        (KeyCode::S, KeyCode::Semicolon, "S→;"),
        (KeyCode::T, KeyCode::U, "T→U"),
        (KeyCode::U, KeyCode::D, "U→D"),
        (KeyCode::W, KeyCode::A, "W→A"),
        (KeyCode::Y, KeyCode::I, "Y→I"),
    ];

    for (from, to, label) in &cases {
        assert_pipeline_remap(*from, *to, &mut remap, label);
    }
}

#[test]
fn test_pipeline_number_row_remaps() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let cases = [
        (KeyCode::Num3, KeyCode::Comma, "3→,"),
        (KeyCode::Num4, KeyCode::Period, "4→."),
        (KeyCode::Num5, KeyCode::P, "5→P"),
        (KeyCode::Num6, KeyCode::Y, "6→Y"),
        (KeyCode::Num7, KeyCode::F, "7→F"),
        (KeyCode::Num8, KeyCode::G, "8→G"),
        (KeyCode::Num9, KeyCode::C, "9→C"),
        (KeyCode::Num0, KeyCode::R, "0→R"),
    ];

    for (from, to, label) in &cases {
        assert_pipeline_remap(*from, *to, &mut remap, label);
    }
}

#[test]
fn test_pipeline_function_key_remaps() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let cases = [
        (KeyCode::F1, KeyCode::LMeta, "F1→Win"),
        (KeyCode::F2, KeyCode::Escape, "F2→Esc"),
        (KeyCode::F3, KeyCode::LCtrl, "F3→LCtrl"),
        (KeyCode::F4, KeyCode::LAlt, "F4→LAlt"),
        (KeyCode::F5, KeyCode::Backspace, "F5→BS"),
        (KeyCode::F6, KeyCode::Delete, "F6→Del"),
        (KeyCode::F8, KeyCode::Tab, "F8→Tab"),
        (KeyCode::F9, KeyCode::Tab, "F9→Tab"),
        (KeyCode::F10, KeyCode::Tab, "F10→Tab"),
        (KeyCode::F11, KeyCode::Tab, "F11→Tab"),
        (KeyCode::F12, KeyCode::Tab, "F12→Tab"),
    ];

    for (from, to, label) in &cases {
        assert_pipeline_remap(*from, *to, &mut remap, label);
    }
}

#[test]
fn test_pipeline_special_keys() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    assert_pipeline_remap(KeyCode::Escape, KeyCode::Num5, &mut remap, "Esc→5");
    assert_pipeline_remap(KeyCode::Backspace, KeyCode::Delete, &mut remap, "BS→Del");
    assert_pipeline_remap(KeyCode::Delete, KeyCode::Num4, &mut remap, "Del→4");
    assert_pipeline_remap(KeyCode::Enter, KeyCode::Yen, &mut remap, "Enter→Yen");
    assert_pipeline_remap(KeyCode::LAlt, KeyCode::LCtrl, &mut remap, "LAlt→LCtrl");
}

#[test]
fn test_pipeline_punctuation_remaps() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    assert_pipeline_remap(KeyCode::LeftBracket, KeyCode::S, &mut remap, "[→S");
    assert_pipeline_remap(KeyCode::RightBracket, KeyCode::Minus, &mut remap, "]→-");
    assert_pipeline_remap(KeyCode::Quote, KeyCode::Z, &mut remap, "'→Z");
    assert_pipeline_remap(KeyCode::Equal, KeyCode::Slash, &mut remap, "=→/");
    assert_pipeline_remap(KeyCode::Minus, KeyCode::L, &mut remap, "-→L");
    assert_pipeline_remap(KeyCode::Semicolon, KeyCode::V, &mut remap, ";→V");
    assert_pipeline_remap(KeyCode::Comma, KeyCode::F9, &mut remap, ",→F9");
    assert_pipeline_remap(KeyCode::Period, KeyCode::F10, &mut remap, ".→F10");
    assert_pipeline_remap(KeyCode::Slash, KeyCode::F11, &mut remap, "/→F11");
}

#[test]
fn test_pipeline_taphold_tap_b_enter() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Quick tap B → should inject Enter
    let output = run_press_release(KeyCode::B, &mut remap);
    let keys: Vec<KeyCode> = output.iter().map(|e| e.keycode()).collect();
    assert!(
        keys.contains(&KeyCode::Enter),
        "TapHold B tap should inject Enter, got: {:?}",
        keys
    );
}

#[test]
fn test_pipeline_taphold_tap_v_delete() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let output = run_press_release(KeyCode::V, &mut remap);
    let keys: Vec<KeyCode> = output.iter().map(|e| e.keycode()).collect();
    assert!(
        keys.contains(&KeyCode::Delete),
        "TapHold V tap should inject Delete, got: {:?}",
        keys
    );
}

#[test]
fn test_pipeline_taphold_tap_m_backspace() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let output = run_press_release(KeyCode::M, &mut remap);
    let keys: Vec<KeyCode> = output.iter().map(|e| e.keycode()).collect();
    assert!(
        keys.contains(&KeyCode::Backspace),
        "TapHold M tap should inject Backspace, got: {:?}",
        keys
    );
}

#[test]
fn test_pipeline_taphold_tap_a_tab() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let output = run_press_release(KeyCode::A, &mut remap);
    let keys: Vec<KeyCode> = output.iter().map(|e| e.keycode()).collect();
    assert!(
        keys.contains(&KeyCode::Tab),
        "TapHold A tap should inject Tab, got: {:?}",
        keys
    );
}

#[test]
fn test_pipeline_taphold_tap_q_minus() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let output = run_press_release(KeyCode::Q, &mut remap);
    let keys: Vec<KeyCode> = output.iter().map(|e| e.keycode()).collect();
    assert!(
        keys.contains(&KeyCode::Minus),
        "TapHold Q tap should inject Minus, got: {:?}",
        keys
    );
}

#[test]
fn test_pipeline_taphold_tap_lctrl_space() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    let output = run_press_release(KeyCode::LCtrl, &mut remap);
    let keys: Vec<KeyCode> = output.iter().map(|e| e.keycode()).collect();
    assert!(
        keys.contains(&KeyCode::Space),
        "TapHold LCtrl tap should inject Space, got: {:?}",
        keys
    );
}

#[test]
fn test_pipeline_modified_output_num2() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Num2 → Shift+7 (apostrophe on JIS)
    let output = run_press_release(KeyCode::Num2, &mut remap);
    let keys: Vec<KeyCode> = output.iter().map(|e| e.keycode()).collect();
    assert!(
        keys.contains(&KeyCode::Num7),
        "Num2 should inject Num7, got: {:?}",
        keys
    );
    assert!(
        keys.contains(&KeyCode::LShift),
        "Num2 should inject LShift, got: {:?}",
        keys
    );
}

#[test]
fn test_pipeline_passthrough_z() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Z is unmapped → should NOT inject (passthrough = no injection)
    let output = run_press_release(KeyCode::Z, &mut remap);
    // In passthrough mode, process_one_event doesn't inject
    // (mapping_triggered is false for unmapped keys)
    assert!(
        output.is_empty(),
        "Unmapped Z should not inject (passthrough), got: {:?}",
        output
    );
}

#[test]
fn test_pipeline_md00_layer_navigation() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Hold B (activate MD_00 via timeout), then press navigation keys
    // Use timestamped events for tap-hold
    let events = vec![
        KeyEvent::press(KeyCode::B).with_timestamp(0),
        // Simulate other key press during hold (permissive hold triggers MD_00)
        KeyEvent::press(KeyCode::Num2).with_timestamp(50_000),
        KeyEvent::release(KeyCode::Num2).with_timestamp(100_000),
        KeyEvent::release(KeyCode::B).with_timestamp(150_000),
    ];

    let output = run_events(events, &mut remap);
    let press_keys: Vec<KeyCode> = output
        .iter()
        .filter(|e| e.is_press())
        .map(|e| e.keycode())
        .collect();

    // With permissive hold, B hold + Num2 should output Left (MD_00 layer)
    assert!(
        press_keys.contains(&KeyCode::Left),
        "MD_00 + Num2 should inject Left, got press: {:?}",
        press_keys
    );
}

#[test]
fn test_pipeline_md00_layer_numbers() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Hold B + press W → should output 1 (MD_00 layer)
    let events = vec![
        KeyEvent::press(KeyCode::B).with_timestamp(0),
        KeyEvent::press(KeyCode::W).with_timestamp(50_000),
        KeyEvent::release(KeyCode::W).with_timestamp(100_000),
        KeyEvent::release(KeyCode::B).with_timestamp(150_000),
    ];

    let output = run_events(events, &mut remap);
    let press_keys: Vec<KeyCode> = output
        .iter()
        .filter(|e| e.is_press())
        .map(|e| e.keycode())
        .collect();

    assert!(
        press_keys.contains(&KeyCode::Num1),
        "MD_00 + W should inject Num1, got press: {:?}",
        press_keys
    );
}

#[test]
fn test_pipeline_md00_layer_fkeys() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Hold B + press S → should output F1 (MD_00 layer)
    let events = vec![
        KeyEvent::press(KeyCode::B).with_timestamp(0),
        KeyEvent::press(KeyCode::S).with_timestamp(50_000),
        KeyEvent::release(KeyCode::S).with_timestamp(100_000),
        KeyEvent::release(KeyCode::B).with_timestamp(150_000),
    ];

    let output = run_events(events, &mut remap);
    let press_keys: Vec<KeyCode> = output
        .iter()
        .filter(|e| e.is_press())
        .map(|e| e.keycode())
        .collect();

    assert!(
        press_keys.contains(&KeyCode::F1),
        "MD_00 + S should inject F1, got press: {:?}",
        press_keys
    );
}

#[test]
fn test_pipeline_inject_has_press_and_release() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Simple remap W→A should inject both press and release
    let output = run_press_release(KeyCode::W, &mut remap);

    let presses: Vec<_> = output.iter().filter(|e| e.is_press()).collect();
    let releases: Vec<_> = output.iter().filter(|e| e.is_release()).collect();

    assert_eq!(presses.len(), 1, "Should inject exactly 1 press");
    assert_eq!(releases.len(), 1, "Should inject exactly 1 release");
    assert_eq!(presses[0].keycode(), KeyCode::A);
    assert_eq!(releases[0].keycode(), KeyCode::A);
}

#[test]
fn test_pipeline_rapid_typing_sequence() {
    let config = default_profile_config();
    let mut remap = RemappingState::new(&config);

    // Simulate typing "wertyuiop" (physical keys) rapidly
    let physical_keys = [
        KeyCode::W,
        KeyCode::E,
        KeyCode::R,
        KeyCode::T,
        KeyCode::Y,
        KeyCode::U,
        KeyCode::I,
        KeyCode::O,
        KeyCode::P,
    ];
    let expected = [
        KeyCode::A,
        KeyCode::O,
        KeyCode::E,
        KeyCode::U,
        KeyCode::I,
        KeyCode::D,
        KeyCode::H,
        KeyCode::T,
        KeyCode::N,
    ];

    let mut events = Vec::new();
    let mut ts = 0u64;
    for key in &physical_keys {
        events.push(KeyEvent::press(*key).with_timestamp(ts));
        ts += 30_000; // 30ms between keys
        events.push(KeyEvent::release(*key).with_timestamp(ts));
        ts += 10_000;
    }

    let output = run_events(events, &mut remap);
    let press_keys: Vec<KeyCode> = output
        .iter()
        .filter(|e| e.is_press())
        .map(|e| e.keycode())
        .collect();

    assert_eq!(
        press_keys, expected,
        "Typing 'wertyuiop' should produce {:?}, got {:?}",
        expected, press_keys
    );
}
