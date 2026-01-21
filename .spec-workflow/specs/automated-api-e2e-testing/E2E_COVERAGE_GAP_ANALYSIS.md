# E2E Coverage Gap Analysis - Automated API E2E Testing

**Date:** 2026-01-21
**Spec:** automated-api-e2e-testing
**Status:** ❌ Incomplete - Significant gaps identified

---

## Executive Summary

The automated E2E testing implementation has **significant coverage gaps**:

- ✅ **14 endpoints tested** out of 40+ available endpoints (35% coverage)
- ❌ **20 test cases** vs. required **30+ test cases** (REQ-3.3.1 not met)
- ❌ **Tests currently broken** due to missing `zod` dependency
- ❌ **26+ API endpoints not tested** (65% missing coverage)
- ❌ **WebSocket functionality not tested**
- ❌ **UI-specific features not tested** (config editor, simulator, macro recorder)

---

## Critical Issues

### 1. Tests Are Not Executable ⚠️

```bash
$ npm run test:e2e:auto
Error: Cannot find module 'zod'
```

**Impact:** Cannot run any automated E2E tests currently
**Fix Required:** Install zod dependency in root package.json or scripts directory

### 2. Insufficient Test Coverage

**Requirement:** Minimum 30 test cases (REQ-3.3.1)
**Current:** 20 test cases (18 basic + 2 integration)
**Gap:** 10 additional test cases needed

---

## API Endpoint Coverage Analysis

### ✅ Tested Endpoints (14/40+)

| Endpoint | Test ID | Status |
|----------|---------|--------|
| GET /api/health | health-001 | ✅ |
| GET /api/version | version-001 | ✅ |
| GET /api/status | status-001 | ✅ |
| GET /api/devices | devices-001 | ✅ |
| PATCH /api/devices/:id | devices-002, devices-003 | ✅ |
| GET /api/profiles | profiles-001 | ✅ |
| GET /api/profiles/active | profiles-002 | ✅ |
| GET /api/profiles/:name | profiles-009 | ✅ |
| POST /api/profiles | profiles-003, profiles-004 | ✅ |
| PUT /api/profiles/:name | profiles-010 | ✅ |
| POST /api/profiles/:name/activate | profiles-005, profiles-006 | ✅ |
| DELETE /api/profiles/:name | profiles-007, profiles-008 | ✅ |
| GET /api/metrics/latency | metrics-001 | ✅ |
| GET /api/layouts | layouts-001 | ✅ |

### ❌ Missing Critical Endpoints (26+)

#### Profiles (4 missing)
- ❌ POST /api/profiles/:name/duplicate - Clone existing profiles
- ❌ PUT /api/profiles/:name/rename - Rename profiles
- ❌ POST /api/profiles/:name/validate - Validate profile configs
- ❌ GET /api/profiles/:name/config - Get profile config (typo in route: `/profiles:name/config`)

#### Devices (4 missing)
- ❌ PUT /api/devices/:id/name - Rename devices
- ❌ PUT /api/devices/:id/layout - Set device layout
- ❌ GET /api/devices/:id/layout - Get device layout
- ❌ DELETE /api/devices/:id - Forget/remove device

#### Config/Layers (5 missing)
- ❌ GET /api/config - Get full configuration
- ❌ PUT /api/config - Update configuration
- ❌ POST /api/config/key-mappings - Add key mapping
- ❌ DELETE /api/config/key-mappings/:id - Remove key mapping
- ❌ GET /api/layers - List all layers

#### Layouts (1 missing)
- ❌ GET /api/layouts/:name - Get specific layout

#### Metrics (2 missing)
- ❌ GET /api/metrics/events - Get event log
- ❌ DELETE /api/metrics/events - Clear event log

#### Daemon State (1 missing)
- ❌ GET /api/daemon/state - Get full daemon state

#### Macros (4 missing - **NOT TESTED AT ALL**)
- ❌ POST /api/macros/start-recording - Start macro recording
- ❌ POST /api/macros/stop-recording - Stop macro recording
- ❌ GET /api/macros/recorded-events - Get recorded events
- ❌ POST /api/macros/clear - Clear recorded macros

#### Simulator (2 missing - **NOT TESTED AT ALL**)
- ❌ POST /api/simulator/events - Simulate keyboard events
- ❌ POST /api/simulator/reset - Reset simulator state

#### WebSocket (Not tested - **CRITICAL GAP**)
- ❌ /ws - WebSocket connection for real-time updates
- ❌ Event subscriptions
- ❌ Real-time device/profile changes

---

## UI Feature Coverage

The spec claims to test "real web UI" but focuses only on REST API. **No UI-specific features are tested:**

### ❌ Missing UI E2E Tests

| Feature | Location | Status |
|---------|----------|--------|
| Config Editor (Monaco) | ConfigPage | ❌ Not tested |
| Keyboard Visualizer | ConfigPage | ❌ Not tested |
| Layer Visualization | ConfigPage | ❌ Not tested |
| Key Mapping UI | ConfigPage | ❌ Not tested |
| Profile Management UI | ProfilesPage | ❌ Not tested |
| Device Management UI | DevicesPage | ❌ Not tested |
| Metrics Dashboard | MetricsPage | ❌ Not tested |
| Simulator Interface | SimulatorPage | ❌ Not tested |
| Macro Recorder UI | SimulatorPage | ❌ Not tested |
| WebSocket Live Updates | All pages | ❌ Not tested |

**Note:** There ARE Playwright E2E tests in `keyrx_ui/e2e/` and `keyrx_ui/tests/e2e/`, but they are separate from this automated API E2E testing spec.

---

## Requirements Coverage Matrix

| Requirement | Description | Status | Notes |
|-------------|-------------|--------|-------|
| REQ-3.1.1 | GET /api/status | ✅ | Tested |
| REQ-3.1.2 | GET /api/devices | ✅ | Tested |
| REQ-3.1.3 | GET /api/profiles | ✅ | Tested |
| REQ-3.1.4 | GET /api/profiles/:name/config | ⚠️ | Route has typo |
| REQ-3.1.5 | POST /api/profiles | ✅ | Tested |
| REQ-3.1.6 | DELETE /api/profiles/:name | ✅ | Tested |
| REQ-3.1.7 | POST /api/profiles/:name/activate | ✅ | Tested |
| REQ-3.1.8 | PATCH /api/devices/:id | ✅ | Tested |
| REQ-3.1.9 | GET /api/metrics/latency | ✅ | Tested |
| REQ-3.1.10 | GET /api/layouts | ✅ | Tested |
| REQ-3.1.11 | POST /api/config/reload | ❌ | Endpoint doesn't exist |
| REQ-3.3.1 | Minimum 30 test cases | ❌ | Only 20 test cases |

---

## Test Quality Issues

### 1. Missing Scenarios (REQ-3.2.x)

Many endpoints lack comprehensive scenario coverage:

| Endpoint | Success | Empty | Error | Edge Cases |
|----------|---------|-------|-------|------------|
| GET /api/devices | ✅ | ✅ | ❌ | ❌ |
| GET /api/profiles | ✅ | ❌ | ❌ | ❌ |
| POST /api/profiles | ✅ | N/A | ✅ (duplicate) | ❌ |
| DELETE /api/profiles/:name | ✅ | N/A | ✅ (not found) | ❌ |
| PATCH /api/devices/:id | ✅ | N/A | ✅ (not found) | ❌ |

**Missing edge cases:**
- Invalid JSON payloads
- Concurrent requests (race conditions)
- Large payloads (stress testing)
- Malformed parameters
- Permission/auth errors (if applicable)

### 2. No Performance Testing

**REQ-2.1.x** requires:
- Test suite completion < 2 minutes
- Individual test < 5 seconds
- Daemon startup < 10 seconds

**Current:** No performance metrics collected

### 3. Missing Auto-Fix Coverage

The auto-fix engine is implemented but not proven to work on:
- Schema changes (new fields added to API)
- Daemon crashes during tests
- Port conflicts
- Transient network errors

---

## Comparison: API Tests vs. Playwright E2E Tests

### Automated API E2E Tests (This Spec)
- **Location:** `scripts/test-cases/api-tests.ts`
- **Focus:** REST API endpoints only
- **Coverage:** 14/40+ endpoints (35%)
- **Status:** ❌ Broken (missing zod dependency)
- **Run Command:** `npm run test:e2e:auto`

### Playwright E2E Tests (Separate)
- **Location:** `keyrx_ui/e2e/*.spec.ts`, `keyrx_ui/tests/e2e/*.spec.ts`
- **Focus:** Full UI workflows with browser automation
- **Coverage:** ~24 test files covering UI interactions
- **Status:** ✅ Working (separate test suite)
- **Run Command:** `npm run test:e2e` (Playwright)

**Gap:** These two test suites are disconnected. No unified E2E testing strategy.

---

## Recommendations

### Immediate Fixes (P0)

1. **Fix broken tests**
   ```bash
   cd /home/rmondo/repos/keyrx
   npm install zod
   # OR add to root package.json
   ```

2. **Run tests to verify baseline**
   ```bash
   npm run test:e2e:auto --prefix keyrx_ui
   ```

3. **Add missing API endpoint tests** (priority order):
   - Macro endpoints (critical for UI feature)
   - Simulator endpoints (critical for UI feature)
   - Config/layer endpoints (core functionality)
   - Device management endpoints (rename, layout, forget)

### Short-term Improvements (P1)

4. **Increase test coverage to 30+ cases**
   - Add edge case scenarios for existing endpoints
   - Add error handling tests (invalid input, concurrent requests)
   - Add performance benchmarks

5. **Test WebSocket functionality**
   - Connection establishment
   - Event subscriptions
   - Real-time updates
   - Reconnection handling

6. **Add UI-specific E2E tests** (or clarify spec scope)
   - Either add browser automation tests to this spec
   - OR update spec to clarify it's "API-only" testing

### Long-term Enhancements (P2)

7. **Unify test strategies**
   - Consolidate Playwright E2E tests with API E2E tests
   - Create unified test dashboard
   - Share fixtures and utilities

8. **Add integration tests**
   - Multi-step workflows (tested: 2, need more)
   - Cross-feature interactions
   - State persistence across daemon restarts

9. **Implement continuous monitoring**
   - Metrics dashboard (implemented but not proven)
   - Flaky test detection
   - Performance regression tracking

---

## Test Execution Status

**Last Run:** 2026-01-21
**Result:** ❌ FAILED - Cannot execute due to missing dependency
**Error:**
```
Error: Cannot find module 'zod'
Require stack:
- /home/rmondo/repos/keyrx/scripts/api-client/client.ts
- /home/rmondo/repos/keyrx/scripts/automated-e2e-test.ts
```

**Next Steps:**
1. Install dependencies
2. Run full test suite
3. Identify failing tests
4. Update this report with actual test results

---

## Conclusion

**Current State:** The automated E2E testing implementation is incomplete and non-functional.

**Coverage:** 35% of daemon API endpoints are tested (14/40+)

**Missing:**
- 26+ API endpoints (65%)
- WebSocket testing
- UI workflow testing
- Performance testing
- 10 additional test cases to meet requirements

**Actionable:** The implementation has good infrastructure (test runner, fixtures, comparators, auto-fix engine) but needs:
1. Fix broken dependencies
2. Add missing endpoint tests
3. Add WebSocket tests
4. Clarify whether "real web UI" means browser automation or just API calls

**Recommendation:** Prioritize fixing the broken tests, then systematically add coverage for macros, simulator, and config/layer endpoints before considering this spec complete.
