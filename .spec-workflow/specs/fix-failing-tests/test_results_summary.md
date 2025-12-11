# Complete Test Suite Results

**Date**: 2025-12-11
**Environment Variable**: `KEYRX_SKIP_DEVICE_SCAN=1`

## Summary

| Metric | Value |
|--------|-------|
| Total Tests Passed | 3,264 |
| Total Tests Failed | 1 |
| Total Ignored | 8 |

## Known Issues

### 1. FFI Contract Adherence Test (FAILING)
- **Test**: `verify_ffi_contract_adherence`
- **Status**: Expected failure - separate issue
- **Cause**: 86 FFI functions don't have corresponding contract definitions
- **Resolution**: Requires adding contract definitions (out of scope for fix-failing-tests spec)

### 2. Device Scan Tests (FIXED with env var)
- **Tests**: `test_device_registry_list_devices_empty`, `test_device_registry_list_devices_with_devices`, `test_c_api_device_registry_list_devices_round_trip`
- **Cause**: Tests pick up real hardware devices from host system
- **Resolution**: Set `KEYRX_SKIP_DEVICE_SCAN=1` to disable device scanning in tests

### 3. Flaky Test (Intermittent)
- **Test**: `test_search_with_deprecated`
- **Cause**: Serial lock contention when running with other tests
- **Status**: Passes consistently when run in isolation, occasionally fails in full suite
- **Severity**: Low - test isolation issue, not a code bug

## Originally Fixed Tests (from spec scope)

Both tests that were originally failing are now passing:
1. `test_c_api_null_label_clears` - FFI device registry test
2. `test_macro_generates_doc` - Scripting documentation test

## Recommendations

1. **CI Configuration**: Ensure `KEYRX_SKIP_DEVICE_SCAN=1` is set in CI environment
2. **Contract Definitions**: Create a separate spec/task to add missing FFI contract definitions
3. **Flaky Test Fix**: Consider adding a serial group annotation to isolate `test_search_with_deprecated`

## Full Test Output

See `test_results_all.txt` for complete test output.
