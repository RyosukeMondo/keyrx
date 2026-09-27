use evdev::Key;

use keyrx_core::config::KeyCode;

use super::*;

/// Test that all letter keys map correctly
#[test]
fn test_letter_keys_mapping() {
    // Test A-Z
    assert_eq!(evdev_to_keycode(Key::KEY_A.code()), Some(KeyCode::A));
    assert_eq!(evdev_to_keycode(Key::KEY_Z.code()), Some(KeyCode::Z));
    assert_eq!(evdev_to_keycode(Key::KEY_M.code()), Some(KeyCode::M));

    // Test round-trip
    assert_eq!(keycode_to_evdev(KeyCode::A), Key::KEY_A.code());
    assert_eq!(keycode_to_evdev(KeyCode::Z), Key::KEY_Z.code());
}

/// Test that number keys map correctly
#[test]
fn test_number_keys_mapping() {
    // Note: evdev KEY_0 is actually the '0' key, not at position 0
    assert_eq!(evdev_to_keycode(Key::KEY_1.code()), Some(KeyCode::Num1));
    assert_eq!(evdev_to_keycode(Key::KEY_0.code()), Some(KeyCode::Num0));
    assert_eq!(evdev_to_keycode(Key::KEY_5.code()), Some(KeyCode::Num5));

    // Round-trip
    assert_eq!(keycode_to_evdev(KeyCode::Num0), Key::KEY_0.code());
    assert_eq!(keycode_to_evdev(KeyCode::Num9), Key::KEY_9.code());
}

/// Test that function keys map correctly
#[test]
fn test_function_keys_mapping() {
    assert_eq!(evdev_to_keycode(Key::KEY_F1.code()), Some(KeyCode::F1));
    assert_eq!(evdev_to_keycode(Key::KEY_F12.code()), Some(KeyCode::F12));
    assert_eq!(evdev_to_keycode(Key::KEY_F24.code()), Some(KeyCode::F24));

    // Round-trip
    assert_eq!(keycode_to_evdev(KeyCode::F1), Key::KEY_F1.code());
    assert_eq!(keycode_to_evdev(KeyCode::F12), Key::KEY_F12.code());
}

/// Test that modifier keys map correctly
#[test]
fn test_modifier_keys_mapping() {
    assert_eq!(
        evdev_to_keycode(Key::KEY_LEFTSHIFT.code()),
        Some(KeyCode::LShift)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_RIGHTSHIFT.code()),
        Some(KeyCode::RShift)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_LEFTCTRL.code()),
        Some(KeyCode::LCtrl)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_RIGHTCTRL.code()),
        Some(KeyCode::RCtrl)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_LEFTALT.code()),
        Some(KeyCode::LAlt)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_RIGHTALT.code()),
        Some(KeyCode::RAlt)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_LEFTMETA.code()),
        Some(KeyCode::LMeta)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_RIGHTMETA.code()),
        Some(KeyCode::RMeta)
    );

    // Round-trip
    assert_eq!(keycode_to_evdev(KeyCode::LShift), Key::KEY_LEFTSHIFT.code());
    assert_eq!(keycode_to_evdev(KeyCode::RAlt), Key::KEY_RIGHTALT.code());
}

/// Test special keys mapping
#[test]
fn test_special_keys_mapping() {
    assert_eq!(evdev_to_keycode(Key::KEY_ESC.code()), Some(KeyCode::Escape));
    assert_eq!(
        evdev_to_keycode(Key::KEY_ENTER.code()),
        Some(KeyCode::Enter)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_BACKSPACE.code()),
        Some(KeyCode::Backspace)
    );
    assert_eq!(evdev_to_keycode(Key::KEY_TAB.code()), Some(KeyCode::Tab));
    assert_eq!(
        evdev_to_keycode(Key::KEY_SPACE.code()),
        Some(KeyCode::Space)
    );

    // Round-trip
    assert_eq!(keycode_to_evdev(KeyCode::Escape), Key::KEY_ESC.code());
    assert_eq!(keycode_to_evdev(KeyCode::Enter), Key::KEY_ENTER.code());
}

/// Test arrow keys mapping
#[test]
fn test_arrow_keys_mapping() {
    assert_eq!(evdev_to_keycode(Key::KEY_LEFT.code()), Some(KeyCode::Left));
    assert_eq!(
        evdev_to_keycode(Key::KEY_RIGHT.code()),
        Some(KeyCode::Right)
    );
    assert_eq!(evdev_to_keycode(Key::KEY_UP.code()), Some(KeyCode::Up));
    assert_eq!(evdev_to_keycode(Key::KEY_DOWN.code()), Some(KeyCode::Down));

    // Round-trip
    assert_eq!(keycode_to_evdev(KeyCode::Left), Key::KEY_LEFT.code());
    assert_eq!(keycode_to_evdev(KeyCode::Down), Key::KEY_DOWN.code());
}

/// Test numpad keys mapping
#[test]
fn test_numpad_keys_mapping() {
    assert_eq!(
        evdev_to_keycode(Key::KEY_KP0.code()),
        Some(KeyCode::Numpad0)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_KP9.code()),
        Some(KeyCode::Numpad9)
    );
    assert_eq!(
        evdev_to_keycode(Key::KEY_KPENTER.code()),
        Some(KeyCode::NumpadEnter)
    );

    // Round-trip
    assert_eq!(keycode_to_evdev(KeyCode::Numpad0), Key::KEY_KP0.code());
    assert_eq!(
        keycode_to_evdev(KeyCode::NumpadEnter),
        Key::KEY_KPENTER.code()
    );
}

/// Test unknown key returns None
#[test]
fn test_unknown_key_returns_none() {
    // Use an invalid/unknown key code
    assert_eq!(evdev_to_keycode(0xFFFF), None);
}

/// Test all KeyCode variants have round-trip consistency
#[test]
fn test_all_keycodes_roundtrip() {
    let all_keycodes = [
        KeyCode::A,
        KeyCode::B,
        KeyCode::C,
        KeyCode::D,
        KeyCode::E,
        KeyCode::F,
        KeyCode::G,
        KeyCode::H,
        KeyCode::I,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::M,
        KeyCode::N,
        KeyCode::O,
        KeyCode::P,
        KeyCode::Q,
        KeyCode::R,
        KeyCode::S,
        KeyCode::T,
        KeyCode::U,
        KeyCode::V,
        KeyCode::W,
        KeyCode::X,
        KeyCode::Y,
        KeyCode::Z,
        KeyCode::Num0,
        KeyCode::Num1,
        KeyCode::Num2,
        KeyCode::Num3,
        KeyCode::Num4,
        KeyCode::Num5,
        KeyCode::Num6,
        KeyCode::Num7,
        KeyCode::Num8,
        KeyCode::Num9,
        KeyCode::F1,
        KeyCode::F2,
        KeyCode::F3,
        KeyCode::F4,
        KeyCode::F5,
        KeyCode::F6,
        KeyCode::F7,
        KeyCode::F8,
        KeyCode::F9,
        KeyCode::F10,
        KeyCode::F11,
        KeyCode::F12,
        KeyCode::Escape,
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Tab,
        KeyCode::Space,
        KeyCode::LShift,
        KeyCode::RShift,
        KeyCode::LCtrl,
        KeyCode::RCtrl,
        KeyCode::LAlt,
        KeyCode::RAlt,
        KeyCode::LMeta,
        KeyCode::RMeta,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
    ];

    for keycode in all_keycodes {
        let evdev_code = keycode_to_evdev(keycode);
        let result = evdev_to_keycode(evdev_code);
        assert_eq!(result, Some(keycode), "Round-trip failed for {:?}", keycode);
    }
}
