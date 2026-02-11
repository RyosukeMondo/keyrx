//! Configuration constants (SSOT)
//!
//! This module defines all configuration limits and constants in a single source of truth.

/// Maximum custom modifier ID (MD_00 through MD_FE)
///
/// This allows 255 custom modifiers (IDs 0-254).
/// MD_FF (255) is reserved and not usable.
///
/// # Examples
///
/// ```
/// use keyrx_core::config::MAX_MODIFIER_ID;
///
/// assert_eq!(MAX_MODIFIER_ID, 0xFE); // 254 in decimal
/// assert_eq!(MAX_MODIFIER_ID + 1, 255); // Total modifiers: 0-254 inclusive
/// ```
pub const MAX_MODIFIER_ID: u16 = 0xFE;

/// Maximum custom lock ID (LK_00 through LK_FE)
///
/// This allows 255 custom locks (IDs 0-254).
/// LK_FF (255) is reserved and not usable.
///
/// # Examples
///
/// ```
/// use keyrx_core::config::MAX_LOCK_ID;
///
/// assert_eq!(MAX_LOCK_ID, 0xFE); // 254 in decimal
/// assert_eq!(MAX_LOCK_ID + 1, 255); // Total locks: 0-254 inclusive
/// ```
pub const MAX_LOCK_ID: u16 = 0xFE;

/// Total number of custom modifiers (255)
///
/// Derived from MAX_MODIFIER_ID + 1 (0-254 inclusive = 255 total)
pub const MODIFIER_COUNT: usize = (MAX_MODIFIER_ID + 1) as usize;

/// Total number of custom locks (255)
///
/// Derived from MAX_LOCK_ID + 1 (0-254 inclusive = 255 total)
pub const LOCK_COUNT: usize = (MAX_LOCK_ID + 1) as usize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_ssot() {
        // Verify SSOT consistency
        assert_eq!(
            MAX_MODIFIER_ID, MAX_LOCK_ID,
            "Modifier and lock limits should match"
        );
        assert_eq!(
            MODIFIER_COUNT, 255,
            "Should support 255 custom modifiers (0-254)"
        );
        assert_eq!(LOCK_COUNT, 255, "Should support 255 custom locks (0-254)");
    }

    #[test]
    fn test_max_id_range() {
        // Verify max IDs fit in u8
        assert!(MAX_MODIFIER_ID <= 0xFF, "MAX_MODIFIER_ID must fit in u8");
        assert!(MAX_LOCK_ID <= 0xFF, "MAX_LOCK_ID must fit in u8");

        // Verify max IDs are exactly 0xFE (254)
        assert_eq!(MAX_MODIFIER_ID, 0xFE);
        assert_eq!(MAX_LOCK_ID, 0xFE);
    }

    #[test]
    fn test_count_derivation() {
        // Verify counts are correctly derived from max IDs
        assert_eq!(MODIFIER_COUNT, (MAX_MODIFIER_ID + 1) as usize);
        assert_eq!(LOCK_COUNT, (MAX_LOCK_ID + 1) as usize);
    }
}
