use keyrx_compiler::error::ParseError;
use keyrx_compiler::parser::validators::{
    parse_condition_string, parse_lock_id, parse_modifier_id, parse_physical_key, parse_virtual_key,
};
use keyrx_core::config::{Condition, KeyCode};

#[cfg(test)]
mod parse_physical_key_tests {
    use super::*;

    #[test]
    fn test_parse_physical_key_with_vk_prefix() {
        assert_eq!(parse_physical_key("VK_A").unwrap(), KeyCode::A);
        assert_eq!(parse_physical_key("VK_Enter").unwrap(), KeyCode::Enter);
        assert_eq!(parse_physical_key("VK_LShift").unwrap(), KeyCode::LShift);
    }

    #[test]
    fn test_parse_physical_key_without_prefix() {
        assert_eq!(parse_physical_key("A").unwrap(), KeyCode::A);
        assert_eq!(parse_physical_key("Enter").unwrap(), KeyCode::Enter);
        assert_eq!(parse_physical_key("LShift").unwrap(), KeyCode::LShift);
    }

    #[test]
    fn test_parse_physical_key_all_letters() {
        for c in 'A'..='Z' {
            let key_name = c.to_string();
            let result = parse_physical_key(&key_name);
            assert!(result.is_ok(), "Failed to parse letter: {}", c);
        }
    }

    #[test]
    fn test_parse_physical_key_numbers() {
        assert_eq!(parse_physical_key("Num0").unwrap(), KeyCode::Num0);
        assert_eq!(parse_physical_key("0").unwrap(), KeyCode::Num0);
        assert_eq!(parse_physical_key("Num9").unwrap(), KeyCode::Num9);
        assert_eq!(parse_physical_key("9").unwrap(), KeyCode::Num9);
    }

    #[test]
    fn test_parse_physical_key_function_keys() {
        assert_eq!(parse_physical_key("F1").unwrap(), KeyCode::F1);
        assert_eq!(parse_physical_key("F12").unwrap(), KeyCode::F12);
        assert_eq!(parse_physical_key("F24").unwrap(), KeyCode::F24);
    }

    #[test]
    fn test_parse_physical_key_special_keys() {
        assert_eq!(parse_physical_key("Escape").unwrap(), KeyCode::Escape);
        assert_eq!(parse_physical_key("Esc").unwrap(), KeyCode::Escape);
        assert_eq!(parse_physical_key("Enter").unwrap(), KeyCode::Enter);
        assert_eq!(parse_physical_key("Return").unwrap(), KeyCode::Enter);
        assert_eq!(parse_physical_key("Space").unwrap(), KeyCode::Space);
        assert_eq!(parse_physical_key("Tab").unwrap(), KeyCode::Tab);
    }

    #[test]
    fn test_parse_physical_key_arrow_keys() {
        assert_eq!(parse_physical_key("Left").unwrap(), KeyCode::Left);
        assert_eq!(parse_physical_key("Right").unwrap(), KeyCode::Right);
        assert_eq!(parse_physical_key("Up").unwrap(), KeyCode::Up);
        assert_eq!(parse_physical_key("Down").unwrap(), KeyCode::Down);
    }

    #[test]
    fn test_parse_physical_key_modifiers() {
        assert_eq!(parse_physical_key("LShift").unwrap(), KeyCode::LShift);
        assert_eq!(parse_physical_key("RShift").unwrap(), KeyCode::RShift);
        assert_eq!(parse_physical_key("LCtrl").unwrap(), KeyCode::LCtrl);
        assert_eq!(parse_physical_key("RCtrl").unwrap(), KeyCode::RCtrl);
        assert_eq!(parse_physical_key("LAlt").unwrap(), KeyCode::LAlt);
        assert_eq!(parse_physical_key("RAlt").unwrap(), KeyCode::RAlt);
    }

    #[test]
    fn test_parse_physical_key_media_keys() {
        assert_eq!(parse_physical_key("Mute").unwrap(), KeyCode::Mute);
        assert_eq!(parse_physical_key("VolumeUp").unwrap(), KeyCode::VolumeUp);
        assert_eq!(
            parse_physical_key("VolumeDown").unwrap(),
            KeyCode::VolumeDown
        );
    }

    #[test]
    fn test_parse_physical_key_invalid() {
        let result = parse_physical_key("InvalidKey");
        assert!(matches!(result, Err(ParseError::SyntaxError { .. })));
    }

    #[test]
    fn test_parse_physical_key_fuzzy_suggestions() {
        let result = parse_physical_key("Shft");
        assert!(result.is_err());
        if let Err(ParseError::SyntaxError { message, .. }) = result {
            // Should suggest similar keys
            assert!(
                message.contains("LShift") || message.contains("RShift"),
                "Expected fuzzy suggestions, got: {}",
                message
            );
        }
    }
}

#[cfg(test)]
mod parse_virtual_key_tests {
    use super::*;

    #[test]
    fn test_parse_virtual_key_with_prefix() {
        assert_eq!(parse_virtual_key("VK_A").unwrap(), KeyCode::A);
        assert_eq!(parse_virtual_key("VK_Enter").unwrap(), KeyCode::Enter);
        assert_eq!(parse_virtual_key("VK_Space").unwrap(), KeyCode::Space);
    }

    #[test]
    fn test_parse_virtual_key_missing_prefix() {
        let result = parse_virtual_key("A");
        assert!(matches!(result, Err(ParseError::MissingPrefix { .. })));
    }

    #[test]
    fn test_parse_virtual_key_all_variants() {
        // Test a representative sample of all key types
        let keys = vec![
            ("VK_A", KeyCode::A),
            ("VK_Z", KeyCode::Z),
            ("VK_Num0", KeyCode::Num0),
            ("VK_F1", KeyCode::F1),
            ("VK_F24", KeyCode::F24),
            ("VK_LShift", KeyCode::LShift),
            ("VK_Escape", KeyCode::Escape),
            ("VK_Left", KeyCode::Left),
            ("VK_Mute", KeyCode::Mute),
        ];

        for (input, expected) in keys {
            assert_eq!(
                parse_virtual_key(input).unwrap(),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }
}

#[cfg(test)]
mod parse_modifier_id_tests {
    use super::*;

    #[test]
    fn test_parse_modifier_id_valid_range() {
        // Test boundary values
        assert_eq!(parse_modifier_id("MD_00").unwrap(), 0x00);
        assert_eq!(parse_modifier_id("MD_01").unwrap(), 0x01);
        assert_eq!(parse_modifier_id("MD_FE").unwrap(), 0xFE);
        assert_eq!(parse_modifier_id("MD_fe").unwrap(), 0xFE); // lowercase
    }

    #[test]
    fn test_parse_modifier_id_all_valid_values() {
        // Test a sample of valid hex values
        for i in 0..=0xFE {
            let id_str = format!("MD_{:02X}", i);
            let result = parse_modifier_id(&id_str);
            assert_eq!(result.unwrap(), i as u8, "Failed to parse {}", id_str);
        }
    }

    #[test]
    fn test_parse_modifier_id_out_of_range() {
        let result = parse_modifier_id("MD_FF");
        assert!(matches!(
            result,
            Err(ParseError::ModifierIdOutOfRange {
                got: 0xFF,
                max: 0xFE,
                import_chain: _,
            })
        ));
    }

    #[test]
    fn test_parse_modifier_id_too_large() {
        let result = parse_modifier_id("MD_100");
        assert!(matches!(
            result,
            Err(ParseError::ModifierIdOutOfRange { .. })
        ));
    }

    #[test]
    fn test_parse_modifier_id_physical_names_rejected() {
        // Test all physical modifier names are rejected
        let physical_names = vec![
            "MD_LShift",
            "MD_RShift",
            "MD_LCtrl",
            "MD_RCtrl",
            "MD_LAlt",
            "MD_RAlt",
            "MD_LMeta",
            "MD_RMeta",
        ];

        for name in physical_names {
            let result = parse_modifier_id(name);
            assert!(
                matches!(result, Err(ParseError::PhysicalModifierInMD { .. })),
                "Expected PhysicalModifierInMD error for {}, got: {:?}",
                name,
                result
            );
        }
    }

    #[test]
    fn test_parse_modifier_id_missing_prefix() {
        let result = parse_modifier_id("00");
        assert!(matches!(result, Err(ParseError::MissingPrefix { .. })));
    }

    #[test]
    fn test_parse_modifier_id_invalid_hex() {
        let result = parse_modifier_id("MD_GG");
        assert!(matches!(result, Err(ParseError::InvalidPrefix { .. })));
    }

    #[test]
    fn test_parse_modifier_id_empty_suffix() {
        let result = parse_modifier_id("MD_");
        assert!(matches!(result, Err(ParseError::InvalidPrefix { .. })));
    }
}

#[cfg(test)]
mod parse_lock_id_tests {
    use super::*;

    #[test]
    fn test_parse_lock_id_valid_range() {
        assert_eq!(parse_lock_id("LK_00").unwrap(), 0x00);
        assert_eq!(parse_lock_id("LK_01").unwrap(), 0x01);
        assert_eq!(parse_lock_id("LK_FE").unwrap(), 0xFE);
        assert_eq!(parse_lock_id("LK_fe").unwrap(), 0xFE); // lowercase
    }

    #[test]
    fn test_parse_lock_id_all_valid_values() {
        for i in 0..=0xFE {
            let id_str = format!("LK_{:02X}", i);
            let result = parse_lock_id(&id_str);
            assert_eq!(result.unwrap(), i as u8, "Failed to parse {}", id_str);
        }
    }

    #[test]
    fn test_parse_lock_id_out_of_range() {
        let result = parse_lock_id("LK_FF");
        assert!(matches!(
            result,
            Err(ParseError::LockIdOutOfRange {
                got: 0xFF,
                max: 0xFE,
                import_chain: _,
            })
        ));
    }

    #[test]
    fn test_parse_lock_id_missing_prefix() {
        let result = parse_lock_id("00");
        assert!(matches!(result, Err(ParseError::MissingPrefix { .. })));
    }

    #[test]
    fn test_parse_lock_id_invalid_hex() {
        let result = parse_lock_id("LK_ZZ");
        assert!(matches!(result, Err(ParseError::InvalidPrefix { .. })));
    }

    #[test]
    fn test_parse_lock_id_wrong_prefix() {
        let result = parse_lock_id("MD_00");
        assert!(matches!(result, Err(ParseError::MissingPrefix { .. })));
    }
}

#[cfg(test)]
mod parse_condition_string_tests {
    use super::*;

    #[test]
    fn test_parse_condition_modifier_active() {
        let result = parse_condition_string("MD_00").unwrap();
        assert!(matches!(result, Condition::ModifierActive(0x00)));

        let result = parse_condition_string("MD_FE").unwrap();
        assert!(matches!(result, Condition::ModifierActive(0xFE)));
    }

    #[test]
    fn test_parse_condition_lock_active() {
        let result = parse_condition_string("LK_00").unwrap();
        assert!(matches!(result, Condition::LockActive(0x00)));

        let result = parse_condition_string("LK_FE").unwrap();
        assert!(matches!(result, Condition::LockActive(0xFE)));
    }

    #[test]
    fn test_parse_condition_all_modifier_ids() {
        for i in 0..=0xFE {
            let cond_str = format!("MD_{:02X}", i);
            let result = parse_condition_string(&cond_str);
            assert!(
                matches!(result, Ok(Condition::ModifierActive(_))),
                "Failed to parse condition: {}",
                cond_str
            );
        }
    }

    #[test]
    fn test_parse_condition_all_lock_ids() {
        for i in 0..=0xFE {
            let cond_str = format!("LK_{:02X}", i);
            let result = parse_condition_string(&cond_str);
            assert!(
                matches!(result, Ok(Condition::LockActive(_))),
                "Failed to parse condition: {}",
                cond_str
            );
        }
    }

    #[test]
    fn test_parse_condition_invalid_prefix() {
        let result = parse_condition_string("VK_A");
        assert!(matches!(result, Err(ParseError::InvalidPrefix { .. })));
    }

    #[test]
    fn test_parse_condition_out_of_range() {
        let result = parse_condition_string("MD_FF");
        assert!(matches!(
            result,
            Err(ParseError::ModifierIdOutOfRange { .. })
        ));

        let result = parse_condition_string("LK_FF");
        assert!(matches!(result, Err(ParseError::LockIdOutOfRange { .. })));
    }

    #[test]
    fn test_parse_condition_physical_modifier_rejected() {
        let result = parse_condition_string("MD_LShift");
        assert!(matches!(
            result,
            Err(ParseError::PhysicalModifierInMD { .. })
        ));
    }

    #[test]
    fn test_parse_condition_empty_string() {
        let result = parse_condition_string("");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_case_sensitivity_virtual_keys() {
        // KeyCode names should be case-sensitive
        assert!(parse_virtual_key("VK_a").is_err());
        assert!(parse_virtual_key("VK_escape").is_err());
    }

    #[test]
    fn test_hex_case_insensitivity() {
        // Hex values should accept both uppercase and lowercase
        assert_eq!(parse_modifier_id("MD_ab").unwrap(), 0xAB);
        assert_eq!(parse_modifier_id("MD_AB").unwrap(), 0xAB);
        assert_eq!(parse_modifier_id("MD_Ab").unwrap(), 0xAB);

        assert_eq!(parse_lock_id("LK_cd").unwrap(), 0xCD);
        assert_eq!(parse_lock_id("LK_CD").unwrap(), 0xCD);
    }

    #[test]
    fn test_alias_consistency() {
        // Test that aliases produce the same result
        assert_eq!(
            parse_physical_key("Escape").unwrap(),
            parse_physical_key("Esc").unwrap()
        );
        assert_eq!(
            parse_physical_key("Enter").unwrap(),
            parse_physical_key("Return").unwrap()
        );
        assert_eq!(
            parse_physical_key("Insert").unwrap(),
            parse_physical_key("Ins").unwrap()
        );
        assert_eq!(
            parse_physical_key("Delete").unwrap(),
            parse_physical_key("Del").unwrap()
        );
    }

    #[test]
    fn test_whitespace_not_trimmed() {
        // Whitespace should NOT be trimmed (exact match required)
        assert!(parse_physical_key(" A").is_err());
        assert!(parse_physical_key("A ").is_err());
        assert!(parse_modifier_id(" MD_00").is_err());
        assert!(parse_modifier_id("MD_00 ").is_err());
    }
}
