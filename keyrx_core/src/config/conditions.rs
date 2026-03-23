extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
// CheckBytes is used by #[archive(check_bytes)] derive macro
#[allow(unused_imports)]
use rkyv::{Archive, CheckBytes, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// IME state injected by the daemon at runtime.
///
/// Plain data struct (no rkyv — not serialized to .krx).
/// The daemon queries OS IME APIs and sets this on `DeviceState`
/// before each event is processed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImeState {
    /// Whether an IME is currently active (open/composing mode)
    pub active: bool,
    /// BCP 47 language tag of the current input source (e.g., "ja", "ko", "zh").
    /// Empty string means unknown or no language detected.
    pub language: String,
}

/// Basic condition check for a single modifier or lock
///
/// Used in composite conditions.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[archive(check_bytes)]
#[repr(u8)]
pub enum ConditionItem {
    /// Custom modifier is active (MD_XX)
    ModifierActive(u8) = 0,
    /// Custom lock is active (LK_XX)
    LockActive(u8) = 1,
    /// IME is active (any language)
    ImeActive = 2,
    /// Input language matches (BCP 47 prefix, e.g., "ja", "ko", "zh")
    InputLanguage(String) = 3,
}

/// Conditional mapping support for when/when_not blocks
///
/// Supports single conditions, AND combinations, device matching, and negation.
/// The `#[omit_bounds]` attribute enables recursive NotActive conditions.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
#[archive(check_bytes)]
#[repr(u8)]
pub enum Condition {
    /// Single custom modifier active (MD_XX)
    ModifierActive(u8) = 0,

    /// Single custom lock active (LK_XX)
    LockActive(u8) = 1,

    /// All conditions must be true (AND logic) - for when() with multiple conditions
    AllActive(Vec<ConditionItem>) = 2,

    /// Negated condition - NOT(...)
    NotActive(Vec<ConditionItem>) = 3,

    /// Device ID matches pattern (for per-device configuration)
    DeviceMatches(String) = 4,

    /// IME is active (any language) — standalone condition
    ImeActive = 5,

    /// Input language matches (BCP 47 prefix, e.g., "ja", "ko") — standalone condition
    InputLanguage(String) = 6,
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;

    #[test]
    fn test_condition_variants() {
        // Test ModifierActive variant
        let cond1 = Condition::ModifierActive(0x01);
        assert_eq!(cond1, Condition::ModifierActive(0x01));

        // Test LockActive variant
        let cond2 = Condition::LockActive(0x02);
        assert_eq!(cond2, Condition::LockActive(0x02));

        // Test AllActive variant with multiple conditions
        let cond3 = Condition::AllActive(alloc::vec![
            ConditionItem::ModifierActive(0x01),
            ConditionItem::LockActive(0x02),
        ]);
        if let Condition::AllActive(items) = &cond3 {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected AllActive variant");
        }

        // Test NotActive variant (negation)
        let cond4 = Condition::NotActive(alloc::vec![ConditionItem::ModifierActive(0x01)]);
        if let Condition::NotActive(items) = &cond4 {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], ConditionItem::ModifierActive(0x01));
        } else {
            panic!("Expected NotActive variant");
        }
    }

    #[test]
    fn test_notactive_with_multiple_items() {
        // Test NOT(modifier AND lock)
        let not_multi = Condition::NotActive(alloc::vec![
            ConditionItem::ModifierActive(0x01),
            ConditionItem::LockActive(0x02),
        ]);

        // Verify structure
        if let Condition::NotActive(items) = &not_multi {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], ConditionItem::ModifierActive(0x01));
            assert_eq!(items[1], ConditionItem::LockActive(0x02));
        } else {
            panic!("Expected NotActive");
        }
    }

    #[test]
    fn test_device_matches_condition() {
        use alloc::string::String;

        // Test exact device ID match
        let cond1 = Condition::DeviceMatches(String::from("usb-NumericKeypad-123"));
        if let Condition::DeviceMatches(pattern) = &cond1 {
            assert_eq!(pattern, "usb-NumericKeypad-123");
        } else {
            panic!("Expected DeviceMatches variant");
        }

        // Test wildcard pattern
        let cond2 = Condition::DeviceMatches(String::from("*numpad*"));
        if let Condition::DeviceMatches(pattern) = &cond2 {
            assert_eq!(pattern, "*numpad*");
        } else {
            panic!("Expected DeviceMatches variant");
        }

        // Test equality
        let cond3 = Condition::DeviceMatches(String::from("usb-NumericKeypad-123"));
        assert_eq!(cond1, cond3);

        let cond4 = Condition::DeviceMatches(String::from("different-device"));
        assert_ne!(cond1, cond4);
    }

    #[test]
    fn test_ime_active_condition() {
        let cond = Condition::ImeActive;
        assert_eq!(cond, Condition::ImeActive);

        let item = ConditionItem::ImeActive;
        assert_eq!(item, ConditionItem::ImeActive);
    }

    #[test]
    fn test_input_language_condition() {
        let cond = Condition::InputLanguage(String::from("ja"));
        if let Condition::InputLanguage(lang) = &cond {
            assert_eq!(lang, "ja");
        } else {
            panic!("Expected InputLanguage variant");
        }

        let item = ConditionItem::InputLanguage(String::from("ko"));
        if let ConditionItem::InputLanguage(lang) = &item {
            assert_eq!(lang, "ko");
        } else {
            panic!("Expected InputLanguage variant");
        }
    }

    #[test]
    fn test_all_active_with_ime_items() {
        let cond = Condition::AllActive(alloc::vec![
            ConditionItem::ImeActive,
            ConditionItem::InputLanguage(String::from("ja")),
            ConditionItem::ModifierActive(0x01),
        ]);
        if let Condition::AllActive(items) = &cond {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], ConditionItem::ImeActive);
        } else {
            panic!("Expected AllActive variant");
        }
    }

    #[test]
    fn test_ime_state_default() {
        let state = ImeState::default();
        assert!(!state.active);
        assert_eq!(state.language, "");
    }

    #[test]
    fn test_condition_discriminant_stability() {
        // Verify rkyv round-trip for all Condition variants.
        // This catches reordering or discriminant changes that would break .krx files.
        let cases: alloc::vec::Vec<Condition> = alloc::vec![
            Condition::ModifierActive(0x01),
            Condition::LockActive(0x02),
            Condition::AllActive(alloc::vec![ConditionItem::ModifierActive(0x01)]),
            Condition::NotActive(alloc::vec![ConditionItem::LockActive(0x02)]),
            Condition::DeviceMatches(String::from("test")),
            Condition::ImeActive,
            Condition::InputLanguage(String::from("ja")),
        ];

        for original in &cases {
            let bytes = rkyv::to_bytes::<_, 256>(original).expect("serialize");
            let archived = rkyv::check_archived_root::<Condition>(&bytes).expect("deserialize");
            let deserialized: Condition = archived
                .deserialize(&mut rkyv::Infallible)
                .expect("convert");
            assert_eq!(
                &deserialized, original,
                "Round-trip failed for {:?}",
                original
            );
        }
    }

    #[test]
    fn test_condition_item_discriminant_stability() {
        let cases: alloc::vec::Vec<ConditionItem> = alloc::vec![
            ConditionItem::ModifierActive(0x01),
            ConditionItem::LockActive(0x02),
            ConditionItem::ImeActive,
            ConditionItem::InputLanguage(String::from("ja")),
        ];

        for original in &cases {
            let bytes = rkyv::to_bytes::<_, 256>(original).expect("serialize");
            let archived = rkyv::check_archived_root::<ConditionItem>(&bytes).expect("deserialize");
            let deserialized: ConditionItem = archived
                .deserialize(&mut rkyv::Infallible)
                .expect("convert");
            assert_eq!(
                &deserialized, original,
                "Round-trip failed for {:?}",
                original
            );
        }
    }
}
