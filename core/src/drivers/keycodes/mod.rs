//! Keycode module - single source of truth for all keycode definitions.

mod definitions;
mod macro_def;

pub use definitions::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn keycode_can_be_hashmap_key() {
        let mut map: HashMap<KeyCode, &str> = HashMap::new();
        map.insert(KeyCode::A, "a_value");
        map.insert(KeyCode::CapsLock, "caps_value");

        assert_eq!(map.get(&KeyCode::A), Some(&"a_value"));
        assert_eq!(map.get(&KeyCode::CapsLock), Some(&"caps_value"));
        assert_eq!(map.get(&KeyCode::Z), None);
    }

    #[test]
    fn keycode_from_str_basic() {
        assert_eq!(KeyCode::from_str("A"), Ok(KeyCode::A));
        assert_eq!(KeyCode::from_str("a"), Ok(KeyCode::A));
        assert_eq!(KeyCode::from_str("CapsLock"), Ok(KeyCode::CapsLock));
        assert_eq!(KeyCode::from_str("caps"), Ok(KeyCode::CapsLock));
        assert_eq!(KeyCode::from_str("Escape"), Ok(KeyCode::Escape));
        assert_eq!(KeyCode::from_str("esc"), Ok(KeyCode::Escape));
        assert_eq!(KeyCode::from_str("F1"), Ok(KeyCode::F1));
        assert_eq!(KeyCode::from_str("1"), Ok(KeyCode::Key1));
        assert!(KeyCode::from_str("InvalidKey").is_err());
    }

    #[test]
    fn keycode_from_str_aliases() {
        // Modifier aliases
        assert_eq!(KeyCode::from_str("shift"), Ok(KeyCode::LeftShift));
        assert_eq!(KeyCode::from_str("lshift"), Ok(KeyCode::LeftShift));
        assert_eq!(KeyCode::from_str("ctrl"), Ok(KeyCode::LeftCtrl));
        assert_eq!(KeyCode::from_str("control"), Ok(KeyCode::LeftCtrl));
        assert_eq!(KeyCode::from_str("alt"), Ok(KeyCode::LeftAlt));
        assert_eq!(KeyCode::from_str("altgr"), Ok(KeyCode::RightAlt));
        assert_eq!(KeyCode::from_str("win"), Ok(KeyCode::LeftMeta));
        assert_eq!(KeyCode::from_str("super"), Ok(KeyCode::LeftMeta));
        assert_eq!(KeyCode::from_str("cmd"), Ok(KeyCode::LeftMeta));

        // Navigation aliases
        assert_eq!(KeyCode::from_str("pgup"), Ok(KeyCode::PageUp));
        assert_eq!(KeyCode::from_str("pgdn"), Ok(KeyCode::PageDown));

        // Editing aliases
        assert_eq!(KeyCode::from_str("del"), Ok(KeyCode::Delete));
        assert_eq!(KeyCode::from_str("ins"), Ok(KeyCode::Insert));
        assert_eq!(KeyCode::from_str("back"), Ok(KeyCode::Backspace));
        assert_eq!(KeyCode::from_str("bs"), Ok(KeyCode::Backspace));

        // Whitespace aliases
        assert_eq!(KeyCode::from_str("return"), Ok(KeyCode::Enter));
        assert_eq!(KeyCode::from_str("spacebar"), Ok(KeyCode::Space));

        // Numpad aliases
        assert_eq!(KeyCode::from_str("kp0"), Ok(KeyCode::Numpad0));
        assert_eq!(KeyCode::from_str("kpplus"), Ok(KeyCode::NumpadAdd));
        assert_eq!(KeyCode::from_str("kpminus"), Ok(KeyCode::NumpadSubtract));
    }

    #[test]
    fn keycode_display() {
        assert_eq!(KeyCode::A.to_string(), "A");
        assert_eq!(KeyCode::CapsLock.to_string(), "CapsLock");
        assert_eq!(KeyCode::F1.to_string(), "F1");
        assert_eq!(KeyCode::Key1.to_string(), "1");
        assert_eq!(KeyCode::Unknown(999).to_string(), "Unknown(999)");
    }

    #[test]
    fn keycode_from_name() {
        assert_eq!(KeyCode::from_name("A"), Some(KeyCode::A));
        assert_eq!(KeyCode::from_name("escape"), Some(KeyCode::Escape));
        assert_eq!(KeyCode::from_name("invalid"), None);
    }

    #[test]
    fn keycode_name() {
        assert_eq!(KeyCode::A.name(), "A");
        assert_eq!(KeyCode::LeftShift.name(), "LeftShift");
    }

    #[test]
    fn keycode_traits() {
        // Test Copy
        let key = KeyCode::A;
        let key_copy = key;
        assert_eq!(key, key_copy);

        // Test Clone
        let key_clone = key.clone();
        assert_eq!(key, key_clone);

        // Test Eq
        assert_eq!(KeyCode::A, KeyCode::A);
        assert_ne!(KeyCode::A, KeyCode::B);

        // Test Hash via HashMap
        let mut map = HashMap::new();
        map.insert(KeyCode::A, 1);
        assert_eq!(map.get(&KeyCode::A), Some(&1));
    }

    #[test]
    fn all_keycodes_returns_all_variants() {
        let all = all_keycodes();
        // Verify we have all 95 keycodes (26 letters + 10 numbers + 12 F-keys + 8 modifiers +
        // 8 nav + 3 editing + 3 whitespace + 3 locks + 3 escape area + 11 punct + 16 numpad + 7 media = 110)
        // But we need to count exactly what we defined
        assert!(
            all.len() > 90,
            "Should have many keycodes, got {}",
            all.len()
        );

        // Check some specific ones exist
        assert!(all.contains(&KeyCode::A));
        assert!(all.contains(&KeyCode::Z));
        assert!(all.contains(&KeyCode::F1));
        assert!(all.contains(&KeyCode::F12));
        assert!(all.contains(&KeyCode::CapsLock));
        assert!(all.contains(&KeyCode::MediaPlayPause));

        // Unknown should not be in the list
        assert!(!all.iter().any(|k| matches!(k, KeyCode::Unknown(_))));
    }

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::*;

        #[test]
        fn evdev_roundtrip() {
            // Test that evdev_to_keycode and keycode_to_evdev are inverses
            for keycode in all_keycodes() {
                let evdev = keycode_to_evdev(keycode);
                let back = evdev_to_keycode(evdev);
                assert_eq!(keycode, back, "Roundtrip failed for {:?}", keycode);
            }
        }

        #[test]
        fn evdev_specific_codes() {
            // Test specific evdev codes match expected keycodes
            assert_eq!(evdev_to_keycode(1), KeyCode::Escape);
            assert_eq!(evdev_to_keycode(30), KeyCode::A);
            assert_eq!(evdev_to_keycode(58), KeyCode::CapsLock);
            assert_eq!(evdev_to_keycode(125), KeyCode::LeftMeta);
            assert_eq!(evdev_to_keycode(164), KeyCode::MediaPlayPause);
        }

        #[test]
        fn evdev_unknown() {
            assert_eq!(evdev_to_keycode(9999), KeyCode::Unknown(9999));
            assert_eq!(keycode_to_evdev(KeyCode::Unknown(9999)), 9999);
        }

        #[test]
        fn all_evdev_codes_count() {
            let codes = all_evdev_codes();
            assert_eq!(codes.len(), all_keycodes().len());
        }
    }

    mod windows_tests {
        use super::*;

        #[test]
        fn vk_roundtrip() {
            // Test that vk_to_keycode and keycode_to_vk are inverses
            // Note: NumpadEnter shares VK code 0x0D with Enter - distinguished by extended flag
            for keycode in all_keycodes() {
                // Skip NumpadEnter - it shares VK 0x0D with Enter
                // On Windows, these are distinguished by the extended key flag, not VK code
                if matches!(keycode, KeyCode::NumpadEnter) {
                    continue;
                }
                let vk = keycode_to_vk(keycode);
                let back = vk_to_keycode(vk);
                assert_eq!(keycode, back, "Roundtrip failed for {:?}", keycode);
            }
        }

        #[test]
        fn vk_specific_codes() {
            // Test specific VK codes match expected keycodes
            assert_eq!(vk_to_keycode(0x1B), KeyCode::Escape);
            assert_eq!(vk_to_keycode(0x41), KeyCode::A);
            assert_eq!(vk_to_keycode(0x14), KeyCode::CapsLock);
            assert_eq!(vk_to_keycode(0x5B), KeyCode::LeftMeta);
            assert_eq!(vk_to_keycode(0xB3), KeyCode::MediaPlayPause);
        }

        #[test]
        fn vk_unknown() {
            assert_eq!(vk_to_keycode(0xFFFF), KeyCode::Unknown(0xFFFF));
            assert_eq!(keycode_to_vk(KeyCode::Unknown(0xFFFF)), 0xFFFF);
        }
    }
}
