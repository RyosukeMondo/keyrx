//! Tests for modifier helper functions

use super::*;

/// Test with_shift() creates ModifiedOutput mapping with shift=true
#[test]
fn test_with_shift_creates_modified_output() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_2", with_shift("VK_1"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 1);

    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        }) => {
            assert_eq!(*from, KeyCode::Num2);
            assert_eq!(*to, KeyCode::Num1);
            assert_eq!(*shift, true);
            assert_eq!(*ctrl, false);
            assert_eq!(*alt, false);
            assert_eq!(*win, false);
        }
        _ => panic!(
            "Expected ModifiedOutput mapping, got {:?}",
            config.devices[0].mappings[0]
        ),
    }
}

/// Test with_ctrl() creates ModifiedOutput mapping with ctrl=true
#[test]
fn test_with_ctrl_creates_modified_output() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_A", with_ctrl("VK_C"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        }) => {
            assert_eq!(*from, KeyCode::A);
            assert_eq!(*to, KeyCode::C);
            assert_eq!(*shift, false);
            assert_eq!(*ctrl, true);
            assert_eq!(*alt, false);
            assert_eq!(*win, false);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test with_alt() creates ModifiedOutput mapping with alt=true
#[test]
fn test_with_alt_creates_modified_output() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_F1", with_alt("VK_F4"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        }) => {
            assert_eq!(*from, KeyCode::F1);
            assert_eq!(*to, KeyCode::F4);
            assert_eq!(*shift, false);
            assert_eq!(*ctrl, false);
            assert_eq!(*alt, true);
            assert_eq!(*win, false);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test with_win() creates ModifiedOutput mapping with win=true
#[test]
fn test_with_win_creates_modified_output() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_L", with_win("VK_L"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        }) => {
            assert_eq!(*from, KeyCode::L);
            assert_eq!(*to, KeyCode::L);
            assert_eq!(*shift, false);
            assert_eq!(*ctrl, false);
            assert_eq!(*alt, false);
            assert_eq!(*win, true);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test with_mods() with multiple modifiers
#[test]
fn test_with_mods_multiple_modifiers() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_1", with_mods("VK_2", true, true, false, false));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        }) => {
            assert_eq!(*from, KeyCode::Num1);
            assert_eq!(*to, KeyCode::Num2);
            assert_eq!(*shift, true);
            assert_eq!(*ctrl, true);
            assert_eq!(*alt, false);
            assert_eq!(*win, false);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test with_mods() with all modifiers enabled
#[test]
fn test_with_mods_all_modifiers() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_A", with_mods("VK_Z", true, true, true, true));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        }) => {
            assert_eq!(*from, KeyCode::A);
            assert_eq!(*to, KeyCode::Z);
            assert_eq!(*shift, true);
            assert_eq!(*ctrl, true);
            assert_eq!(*alt, true);
            assert_eq!(*win, true);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test with_mods() with no modifiers
#[test]
fn test_with_mods_no_modifiers() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_X", with_mods("VK_Y", false, false, false, false));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        }) => {
            assert_eq!(*from, KeyCode::X);
            assert_eq!(*to, KeyCode::Y);
            assert_eq!(*shift, false);
            assert_eq!(*ctrl, false);
            assert_eq!(*alt, false);
            assert_eq!(*win, false);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test multiple modifier helper calls in same device
#[test]
fn test_multiple_modifier_helpers() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_1", with_shift("VK_1"));
        map("VK_2", with_ctrl("VK_2"));
        map("VK_3", with_alt("VK_3"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].mappings.len(), 3);
}

/// Test with_shift() rejects invalid key
#[test]
fn test_with_shift_rejects_invalid_key() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_A", with_shift("VK_InvalidKey"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid") || err_msg.contains("key"),
        "Error should mention invalid key: {}",
        err_msg
    );
}

/// Test with_ctrl() rejects missing VK_ prefix
#[test]
fn test_with_ctrl_rejects_missing_prefix() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_A", with_ctrl("C"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("VK_") || err_msg.contains("prefix"),
        "Error should mention VK_ prefix requirement: {}",
        err_msg
    );
}

/// Test with_mods() rejects invalid key
#[test]
fn test_with_mods_rejects_invalid_key() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_A", with_mods("VK_NoSuchKey", true, false, false, false));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid") || err_msg.contains("key"),
        "Error should mention invalid key: {}",
        err_msg
    );
}

/// Test map() with ModifiedKey requires device context
#[test]
fn test_modifier_helper_requires_device_context() {
    let mut parser = Parser::new();
    let script = r#"
        map("VK_A", with_shift("VK_B"));
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("device"),
        "Error should mention device context requirement: {}",
        err_msg
    );
}

/// Test with_shift() with function keys
#[test]
fn test_with_shift_function_keys() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_F12", with_shift("VK_F1"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput { from, to, .. }) => {
            assert_eq!(*from, KeyCode::F12);
            assert_eq!(*to, KeyCode::F1);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test with_ctrl() with special keys
#[test]
fn test_with_ctrl_special_keys() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_Tab", with_ctrl("VK_Tab"));
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput { from, to, .. }) => {
            assert_eq!(*from, KeyCode::Tab);
            assert_eq!(*to, KeyCode::Tab);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test realistic example: Shift+Number for symbols
#[test]
fn test_realistic_shift_number_for_symbol() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_9", with_shift("VK_8"));  // Remap 9 to Shift+8 (*)
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from, to, shift, ..
        }) => {
            assert_eq!(*from, KeyCode::Num9);
            assert_eq!(*to, KeyCode::Num8);
            assert_eq!(*shift, true);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}

/// Test realistic example: Ctrl+C for copy
#[test]
fn test_realistic_ctrl_c_copy() {
    let mut parser = Parser::new();
    let script = r#"
        device_start("Test");
        map("VK_F2", with_ctrl("VK_C"));  // F2 sends Ctrl+C
        device_end();
    "#;

    let result = parser.parse_string(script, &PathBuf::from("test.rhai"));
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let config = result.unwrap();
    match &config.devices[0].mappings[0] {
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput { from, to, ctrl, .. }) => {
            assert_eq!(*from, KeyCode::F2);
            assert_eq!(*to, KeyCode::C);
            assert_eq!(*ctrl, true);
        }
        _ => panic!("Expected ModifiedOutput mapping"),
    }
}
