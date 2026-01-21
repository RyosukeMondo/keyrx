# E2E Test Fixing Notes

## Current Status (2026-01-22)

- **Pass Rate**: 36/83 tests passing (43.4%)
- **Previous**: 32/83 (38.6%)
- **Progress**: +4 tests fixed

## Test Executor Architecture

### Important Pattern

The test executor calls `assert(response.data, expected)`, NOT `assert(response, expected)`.

```typescript
// In test-executor/executor.ts line 320:
testResult = testCase.assert(response.data, expectedResult);
```

This means:
- Test `execute()` can return `{ status, data }` for debugging
- But `assert()` only receives the `data` part
- **Don't try to check `result.status` in assert functions**

### Correct Error Handling Pattern

```typescript
// ❌ WRONG - trying to check HTTP status
assert: (actual, expected) => {
  const result = actual as { status: number; data: any };
  if (result.status !== 400) {  // result.status will be undefined!
    return { passed: false, error: 'Expected 400' };
  }
}

// ✅ CORRECT - check error in response data
assert: (actual, expected) => {
  const result = actual as { success: boolean; error?: { code: string; message: string } };
  if (result.success !== false || result.error?.code !== 'BAD_REQUEST') {
    return { passed: false, error: 'Expected BAD_REQUEST error' };
  }
}
```

## Tests Fixed (2026-01-22)

1. **macros-001b**: POST /api/macros/start-recording - Fail when already recording
2. **macros-002b**: POST /api/macros/stop-recording - Fail when not recording
3. **simulator-001d**: POST /api/simulator/events - Fail with no events or scenario
4. **simulator-001e**: POST /api/simulator/events - Fail with unknown scenario

All had the same issue: checking `result.status` which was undefined.

## Remaining Known Issues

### 1. Error Assertion Pattern (High Priority)

**Impact**: Many tests failing with "got undefined"

**Files to Check**:
- `test-cases/profile-management.tests.ts` - Tests expecting 404 for nonexistent profiles
- `test-cases/device-management.tests.ts` - Tests expecting 404 for nonexistent devices
- `test-cases/config-layers.tests.ts` - Tests expecting 400 for invalid operations
- `test-cases/layouts.tests.ts` - Tests expecting 404 for nonexistent layouts

**Fix**: Search for `result.status` in assert functions and replace with error structure checks.

### 2. Profile API 'template' Field (Medium Priority)

**Symptoms**:
```
HTTP 422: "Failed to deserialize the JSON body into the target type: missing field `template` at line 1 column 92"
```

**Affected Tests**: Profile creation/update tests

**Fix**: Add `template` field to profile creation requests. Check current API schema in `keyrx_daemon/src/web/api/profiles.rs`.

### 3. Device Configuration Issues (Medium Priority)

**Symptoms**:
- "Device block not found" errors
- Schema mismatches (expecting string, got null for layout field)

**Root Cause**: Tests running without active devices or configuration

**Potential Fix**:
- Update fixtures to ensure daemon has active devices
- Adjust test expectations for daemon running without devices

### 4. WebSocket Event Notifications (Low Priority)

**Symptoms**:
- Device event notifications fail with 404
- Profile event notifications fail with 422 (missing template)

**Related**: Same as issue #2 for profiles

### 5. Daemon State Tests (Low Priority)

**Test**: status-001 - GET /api/status

**Issue**: Expects `daemon_running: true`, but gets `false`

**Root Cause**: Daemon web server starts, but main event loop (IPC socket) not running

**Options**:
- Fix daemon to start IPC socket in test mode
- Adjust test expectations for test environment

## Systematic Fixing Approach

### Phase 1: Quick Wins - Error Assertions (Estimated: 1-2 hours)

Search and fix all tests with pattern:
```bash
grep -r "result.status" scripts/test-cases/*.ts
```

Replace with error structure checks. This could fix ~10-15 more tests.

### Phase 2: API Schema Updates (Estimated: 2-3 hours)

1. Check current API schemas in backend
2. Update test requests to match current schema
3. Focus on profile tests with missing 'template' field

### Phase 3: Environmental Issues (Estimated: 3-4 hours)

1. Fix daemon fixture to ensure devices are available
2. Add proper device configuration in setup
3. Handle cases where daemon runs without devices

## Testing Strategy

After each fix batch:
```bash
# Run full suite
npx tsx automated-e2e-test.ts --daemon-path ../target/release/keyrx_daemon

# Check specific category
grep "Config:" output  # or Devices:, Profiles:, etc.

# Update metrics
echo '{"timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)'","tests_total":83,"tests_passed":XX,"notes":"..."}' >> metrics.jsonl
```

## Goal

Target: 83/83 tests passing (100%)

Current: 36/83 (43.4%)
Remaining: 47 tests

Estimated effort:
- Phase 1 quick wins: ~15 tests (1-2 hours)
- Phase 2 schema fixes: ~10 tests (2-3 hours)
- Phase 3 environmental: ~10 tests (3-4 hours)
- Remaining complex issues: ~12 tests (variable)

Total estimated: 6-10 hours of focused work
