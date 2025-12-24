extern crate alloc;
use alloc::vec::Vec;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Basic condition check for a single modifier or lock
///
/// Used in composite conditions.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
pub enum ConditionItem {
    /// Custom modifier is active (MD_XX)
    ModifierActive(u8),
    /// Custom lock is active (LK_XX)
    LockActive(u8),
}

/// Conditional mapping support for when/when_not blocks
///
/// Supports single conditions, AND combinations, device matching, and negation.
/// The `#[omit_bounds]` attribute enables recursive NotActive conditions.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug,
)]
pub enum Condition {
    /// Single custom modifier active (MD_XX)
    ModifierActive(u8),

    /// Single custom lock active (LK_XX)
    LockActive(u8),

    /// All conditions must be true (AND logic) - for when() with multiple conditions
    AllActive(Vec<ConditionItem>),

    /// Negated condition - NOT(...)
    /// Currently limited to negating single items or AND combinations
    /// Example: NOT(ModifierActive(0x01))
    NotActive(Vec<ConditionItem>),

    /// Device ID matches pattern (for per-device configuration)
    ///
    /// The pattern is matched against the event's device_id field at runtime.
    /// Supports exact match and simple glob patterns with `*` wildcard.
    ///
    /// # Examples
    ///
    /// - `DeviceMatches("usb-NumericKeypad-123")` - exact match
    /// - `DeviceMatches("*numpad*")` - contains "numpad" (case-sensitive)
    /// - `DeviceMatches("usb-*")` - starts with "usb-"
    ///
    /// If the event has no device_id (None), this condition evaluates to false.
    DeviceMatches(alloc::string::String),
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
}
