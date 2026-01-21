# REST API E2E Test Status

## Summary

**Test Run Date:** 2026-01-22
**Test Suite:** 83 tests
**Results:** 18 passed, 65 failed
**Pass Rate:** 21.7%

## Fixed Issues

### 1. Layout Endpoint Schema (Fixed)
- **Issue:** Test expected wrapped object, API returns raw KLE JSON array
- **Fix:** Updated `layouts.tests.ts` to expect array format
- **Commit:** d727a391

### 2. Event Type Case Sensitivity (Fixed)
- **Issue:** Tests sent 'Press'/'Release', API expects 'press'/'release'
- **Fix:** Updated `simulator.tests.ts` to use lowercase
- **Commit:** d727a391

## Remaining Issues

### 3. API Client Method Missing (Critical)
- **Files:** `workflows.tests.ts`
- **Issue:** Tests call `client.post()`, `client.put()` methods that don't exist
- **Affected Tests:**
  - workflow-002: Profile duplicate → rename → activate
  - workflow-003: Profile validation → fix → activate
  - workflow-005: Config update → add mappings
  - workflow-006: Macro record → simulate → playback
  - workflow-007: Simulator event → mapping → output
- **Fix Needed:** Either:
  - Add convenience methods to ApiClient (post, put, delete)
  - OR refactor tests to use customRequest()
  - OR refactor tests to use specific typed methods

### 4. WebSocket Subscription Timeouts (Critical)
- **Files:** `websocket.tests.ts`
- **Issue:** All WebSocket subscription tests timeout after 5 seconds
- **Affected Tests:**
  - websocket-002: Subscribe to channel
  - websocket-003: Device event notification
  - websocket-004: Profile event notification
  - websocket-005: Reconnection test
- **Symptoms:** `Subscription timeout for channel: devices/profiles`
- **Possible Causes:**
  - WebSocket server not sending subscription acknowledgments
  - Client waiting for wrong message format
  - Channel names mismatch
- **Fix Needed:** Investigate WebSocket server implementation

### 5. Profile/Config API Errors (High Priority)
- **Issue:** Many profile and config tests failing with errors
- **Examples:**
  - "Cannot read properties of undefined (reading 'status')"
  - "Generator error: Device block not found"
  - "Invalid request: Invalid template"
- **Affected Categories:**
  - Profile management (profiles-003 through profiles-013)
  - Config operations (config-001 through config-004)
  - Device operations (devices-004 through devices-007)
- **Fix Needed:**
  - Investigate why API returns unexpected error formats
  - Check if daemon state is properly initialized
  - Verify template validation logic

### 6. Error Response Validation
- **Issue:** Tests expect specific error status codes but get different responses
- **Examples:**
  - Test expects 400, gets undefined status
  - Test expects error object, gets null
- **Fix Needed:**
  - Align test expectations with actual error response format
  - Ensure API returns consistent error structure

## Test Categories Breakdown

| Category | Total | Passed | Failed | Pass Rate |
|----------|-------|--------|--------|-----------|
| Health   | 4     | 3      | 1      | 75%       |
| Devices  | 11    | 1      | 10     | 9%        |
| Profiles | 13    | 0      | 13     | 0%        |
| Config   | 9     | 0      | 9      | 0%        |
| Layouts  | 2     | 0      | 2      | 0%        |
| Macros   | 8     | 6      | 2      | 75%       |
| Simulator| 7     | 3      | 4      | 43%       |
| Workflows| 6     | 0      | 6      | 0%        |
| WebSocket| 5     | 1      | 4      | 20%       |
| Metrics  | 4     | 1      | 3      | 25%       |
| Integration | 14 | 3      | 11     | 21%       |

## Next Steps

### Immediate Actions
1. Add missing ApiClient methods (post, put, delete) or refactor workflow tests
2. Debug WebSocket subscription mechanism
3. Fix profile/config API error handling

### Medium Priority
4. Align error response format expectations
5. Fix device management tests
6. Investigate "Device block not found" errors

### Long-term
7. Add better error messages to tests
8. Improve test isolation
9. Add retry logic for flaky tests
10. Update documentation to reflect actual API behavior

## Verification Checklist Status

- [x] npm install succeeds
- [x] Test suite runs (but many fail)
- [ ] All 65+ tests pass (currently 18/83)
- [ ] No flaky tests (not yet tested)
- [ ] Execution time < 3 minutes (currently unknown)
- [ ] All endpoints covered (83 tests exist)
- [ ] CI workflow passes (not yet tested)
- [ ] Documentation complete (yes, but needs updates)

## Conclusion

While all 54 implementation tasks were marked complete, the verification phase reveals that:
- **Only 21.7% of tests pass**
- **Major API client issues** prevent workflow tests from running
- **WebSocket tests completely broken** due to subscription timeouts
- **Profile/config APIs** have fundamental errors

The spec's "completed" status reflects test *code written*, not tests *passing*. Significant work remains to achieve the stated goal of 100% passing tests.
