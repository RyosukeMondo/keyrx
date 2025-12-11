# CI Check Results - Task 5.1

**Date:** 2025-12-12
**Command:** `just ci-check`

## Summary

| Check | Status |
|-------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy -- -D warnings` | PASS |
| `cargo doc --no-deps` | PASS (with warnings) |
| `cargo nextest run` | PARTIAL - 4259/4260 tests pass |
| `verify-bindings` | Blocked by test failure |

## Issues Found

### 1. Device Registry Tests (FIXED)

**Problem:** Three tests in `ffi_tests.rs` were failing due to hardware device detection:
- `test_device_registry_list_devices_empty`
- `test_device_registry_list_devices_with_devices`
- `test_c_api_device_registry_list_devices_round_trip`

**Root Cause:** The `cfg!(test)` check in `list_devices()` only applies to unit tests (code in `#[cfg(test)]` modules), not integration tests (files in `tests/` directory). When running on a machine with input devices, the function would scan real hardware.

**Fix Applied:** Modified `justfile` to set `KEYRX_SKIP_DEVICE_SCAN=1` environment variable for test recipes.

**Status:** RESOLVED - All 47 FFI tests now pass.

### 2. FFI Contract Adherence Test (PRE-EXISTING)

**Problem:** `verify_ffi_contract_adherence` test fails with 86 errors.

**Error Categories:**
1. **Missing Exports (4):** Functions defined in contracts but not found in code
2. **Unused Contract Imports (65):** Functions imported by contracts but not used/exported
3. **Orphan Exports (17):** FFI functions exported without corresponding contract definitions

**Key Orphan Functions:**
- `keyrx_engine_*` functions (8 functions)
- `keyrx_metrics_*` functions (6 functions)
- `keyrx_migration_*` functions (2 functions)
- `keyrx_definitions_*` functions (2 functions)
- `free_ffi_*` functions (3 functions)

**Status:** OUT OF SCOPE for this spec. This is a pre-existing issue that requires:
- Creating contract definitions for orphan exports, OR
- Removing unused FFI functions, OR
- Updating contracts to match actual exports

This should be tracked in a separate spec like `misc-improvements` or a dedicated FFI contract cleanup task.

## Recommendations

1. **Immediate:** The `fix-failing-tests` spec's original goals (fixing `test_c_api_null_label_clears` and `test_macro_generates_doc`) were completed in earlier phases.

2. **Follow-up:** Create a new task to address the 86 FFI contract validation errors. Options:
   - Add missing contract definitions
   - Clean up unused FFI functions
   - Mark the contract adherence test as `#[ignore]` until contracts are aligned

3. **CI Pipeline:** Consider adding `KEYRX_SKIP_DEVICE_SCAN=1` to GitHub Actions workflows if not already present.

## Test Results Summary

```
Total tests: 4697
Passed: 4259
Failed: 1 (contract_adherence_test)
Skipped: 14
Not run (due to fail-fast): 437
```

When running with `--no-fail-fast`, all non-contract tests pass.
