# Frontend Test Fix Plan

## Current Status
- **Test Files**: 44 failed | 47 passed (91 total)
- **Tests**: 134 failed | 906 passed (1109 total)
- **Failure Rate**: 12% of tests failing

## Failure Categories

### 1. E2E Tests (Playwright) - ~20 files
These require a running server and browser automation. Likely causes:
- Server not running during tests
- Page selectors changed
- Timeouts too short

**Action**: Skip for now - E2E tests should run in CI with proper setup

### 2. Integration Tests - ~10 files
Tests that involve WebSocket connections and multi-component interactions.

**Root Cause**: WebSocket mocking infrastructure broken

**Action**: Fix WebSocket mocking

### 3. Unit Tests - ~14 files
Individual component/function tests failing.

**Root Cause**: Various - need to investigate each

**Action**: Fix high-priority unit tests

## Fix Strategy

### Priority 1: Fix WebSocket Mocking (HIGH IMPACT)
Files affected:
- `src/api/websocket.test.ts` - 10/17 tests failing
- All integration tests using WebSocket

**Issue**: `vi.useFakeTimers()` + `vi.advanceTimersByTimeAsync()` not working with MockWebSocket

**Solution Options**:
1. Use real timers for WebSocket tests
2. Fix MockWebSocket to work with fake timers properly
3. Use `msw` (Mock Service Worker) for WebSocket mocking

### Priority 2: Skip E2E/Performance/Visual Tests in Unit Test Run
These tests require special setup (Playwright, Lighthouse, Percy) and should run separately.

**Solution**: Update vitest.config.ts to exclude these from default run

### Priority 3: Fix Remaining Unit Tests
Address individual test failures on a case-by-case basis.

## Implementation Plan

1. ✅ **Identify failure patterns** (DONE)
2. ⏳ **Fix WebSocket timer mocking**
3. ⏳ **Configure test exclusions**
4. ⏳ **Fix high-priority unit tests**
5. ⏳ **Update CI to run E2E tests separately**

## Expected Outcome
- Unit tests: >95% pass rate
- Integration tests: >90% pass rate
- E2E tests: Run separately in CI (not blocking local development)

---

**Note**: Frontend tests were working before web-ui-ux-refinement spec implementation.
The failures are likely due to incomplete WebSocket mocking setup or test configuration changes.
