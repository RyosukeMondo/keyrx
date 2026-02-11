# SSOT Refactoring: Modifier and Lock Count Centralization

## Problem

The codebase violated the Single Source of Truth (SSOT) principle for modifier and lock limits:

1. **Scattered hardcoded values:** `0xFE`, `254`, `255` appeared in multiple files
2. **Documentation inconsistency:** Docs mentioned "MD_00 - MD_10" (11 modifiers) but system supports 255
3. **No central constant:** Range validation, state initialization, and docs had different values
4. **Incomplete handling:** System supports 255 modifiers/locks but docs and comments only mentioned subsets

## Solution: Single Source of Truth

Created `keyrx_core/src/config/constants.rs` as the SSOT for all configuration limits.

### Constants Defined

```rust
/// Maximum custom modifier ID (MD_00 through MD_FE)
/// This allows 255 custom modifiers (IDs 0-254).
/// MD_FF (255) is reserved and not usable.
pub const MAX_MODIFIER_ID: u16 = 0xFE;

/// Maximum custom lock ID (LK_00 through LK_FE)
/// This allows 255 custom locks (IDs 0-254).
/// LK_FF (255) is reserved and not usable.
pub const MAX_LOCK_ID: u16 = 0xFE;

/// Total number of custom modifiers (255)
/// Derived from MAX_MODIFIER_ID + 1 (0-254 inclusive = 255 total)
pub const MODIFIER_COUNT: usize = (MAX_MODIFIER_ID + 1) as usize;

/// Total number of custom locks (255)
/// Derived from MAX_LOCK_ID + 1 (0-254 inclusive = 255 total)
pub const LOCK_COUNT: usize = (MAX_LOCK_ID + 1) as usize;
```

### Key Decisions

1. **u16 for MAX_*_ID:** Uses u16 to safely represent hex values (0xFE) in range checks
2. **usize for *_COUNT:** Uses usize for array/vector sizes (BitVec initialization)
3. **Derived constants:** COUNT constants are derived from MAX_ID to maintain consistency
4. **Reserved 0xFF:** ID 255 (0xFF) is explicitly reserved and not usable

## Files Updated

### 1. keyrx_core/src/config/constants.rs (NEW)
**+73 lines**
- Defines all SSOT constants
- Includes comprehensive tests validating consistency

### 2. keyrx_core/src/config/mod.rs
**+2 lines**
```rust
pub mod constants;
pub use constants::{LOCK_COUNT, MAX_LOCK_ID, MAX_MODIFIER_ID, MODIFIER_COUNT};
```

### 3. keyrx_core/src/parser/validators.rs
**+4 lines**
```rust
use crate::config::{Condition, KeyCode, MAX_LOCK_ID, MAX_MODIFIER_ID};

// Changed from hardcoded 0xFE to:
if id > MAX_MODIFIER_ID {
    return Err(ParseError::ModifierIdOutOfRange { got: id, max: MAX_MODIFIER_ID });
}

if id > MAX_LOCK_ID {
    return Err(ParseError::LockIdOutOfRange { got: id, max: MAX_LOCK_ID });
}
```

### 4. keyrx_core/src/runtime/state.rs
**+3 lines**
```rust
use crate::config::{Condition, ConditionItem, KeyCode, MAX_MODIFIER_ID, MODIFIER_COUNT};

// Changed from hardcoded 254 to:
const MAX_VALID_ID: u8 = MAX_MODIFIER_ID as u8;

// Changed from hardcoded 255 to:
pub fn new() -> Self {
    Self {
        modifiers: bitvec![u8, Lsb0; 0; MODIFIER_COUNT],
        locks: bitvec![u8, Lsb0; 0; MODIFIER_COUNT],
        // ...
    }
}
```

### 5. Documentation Updates

Updated all docs to reflect **255 modifiers** (MD_00 through MD_FE) and **255 locks** (LK_00 through LK_FE):

- `docs/tap-hold-md-xx-test-plan.md`
- `docs/architecture/dynamic-key-blocking-implementation.md`
- Removed references to "MD_00 - MD_10" (11 modifiers)
- Added explicit mention of full range: MD_00 - MD_FE, LK_00 - LK_FE

## Verification

### Tests Pass
```bash
$ cargo test -p keyrx_core --lib config::constants
running 3 tests
test config::constants::tests::test_constants_ssot ... ok
test config::constants::tests::test_count_derivation ... ok
test config::constants::tests::test_max_id_range ... ok
```

### Build Succeeds
```bash
$ cargo build --release
Finished `release` profile [optimized] target(s) in 2m 37s
```

## Benefits

1. **Single Source of Truth:** All modifier/lock limits defined in one place
2. **Consistency Guaranteed:** Derived constants prevent drift
3. **Easy to Change:** Future limit changes require updating only constants.rs
4. **Type Safety:** Compile-time checks prevent mismatched types
5. **Self-Documenting:** Constants have clear names and comprehensive docs
6. **Testable:** Constants include validation tests

## Architecture Compliance

### SOLID Principles

**Single Responsibility Principle (SRP):**
- constants.rs: Only defines configuration limits
- validators.rs: Only validates input against limits
- state.rs: Only manages state using defined limits

**Open/Closed Principle:**
- Extending to 256 modifiers requires changing only constants.rs
- No code changes needed in consumers

### SSOT (Single Source of Truth)

**Before:**
- Hardcoded `0xFE` in validators.rs
- Hardcoded `254` in state.rs
- Hardcoded `255` in state.rs BitVec initialization
- Documentation mentioned "11 modifiers"

**After:**
- Single constants.rs defines all limits
- All code references the same constants
- Documentation updated to match reality
- Tests verify consistency

## Migration Path

If the limit needs to change (e.g., to 256 modifiers):

1. Update `constants.rs`:
   ```rust
   pub const MAX_MODIFIER_ID: u16 = 0xFF; // Was 0xFE
   ```

2. All dependent code automatically uses the new limit:
   - validators.rs: Accepts MD_FF
   - state.rs: Creates 256-bit BitVec
   - No other code changes needed

3. Update tests to verify new limit

## Compatibility

**Backward Compatible:** Yes
- Same functional behavior (255 modifiers/locks)
- Only internal refactoring
- No API changes
- .krx file format unchanged

**Forward Compatible:** Yes
- Easy to extend to 256 modifiers by changing constants.rs
- Derived constants automatically adjust

## Summary

This refactoring establishes SSOT for modifier/lock limits, eliminating hardcoded magic numbers and ensuring consistency across:

- Range validation (validators.rs)
- State management (state.rs)
- Documentation
- Tests

**Result:** Clean, maintainable codebase that correctly handles all 255 custom modifiers (MD_00 - MD_FE) and 255 custom locks (LK_00 - LK_FE).
