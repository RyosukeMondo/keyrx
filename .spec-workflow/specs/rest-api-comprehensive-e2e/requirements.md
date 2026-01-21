# Requirements: REST API Comprehensive E2E Testing

## 1. Overview

Test ALL daemon features via REST API endpoints with JSON-based communication. No browser/JavaScript required - pure API-driven feature testing that exercises the full backend stack (CLI, daemon, services).

## 2. Goals

1. **Complete API Coverage**: Test all 40+ REST endpoints across all feature categories
2. **Feature Validation**: Exercise core features (profiles, devices, config, macros, simulator) via API
3. **JSON-Based Testing**: All inputs/outputs in JSON format for automation
4. **Fix Existing Tests**: Resolve broken tests (missing dependencies, incorrect assertions)
5. **Production Ready**: Tests run in CI/CD with < 3 minute execution time

## 3. Functional Requirements

### 3.1 Endpoint Coverage (40+ endpoints)

#### 3.1.1 Health & Metrics (6 endpoints)
- **REQ-3.1.1**: GET /api/health - Health check
- **REQ-3.1.2**: GET /api/version - Daemon version info
- **REQ-3.1.3**: GET /api/status - Daemon status
- **REQ-3.1.4**: GET /api/metrics/latency - Latency statistics
- **REQ-3.1.5**: GET /api/metrics/events - Event log with filtering
- **REQ-3.1.6**: DELETE /api/metrics/events - Clear event log
- **REQ-3.1.7**: GET /api/daemon/state - Full daemon state

#### 3.1.2 Device Management (7 endpoints)
- **REQ-3.1.8**: GET /api/devices - List all devices
- **REQ-3.1.9**: PATCH /api/devices/:id - Update device config (enable/disable)
- **REQ-3.1.10**: PUT /api/devices/:id/name - Rename device
- **REQ-3.1.11**: PUT /api/devices/:id/layout - Set device keyboard layout
- **REQ-3.1.12**: GET /api/devices/:id/layout - Get device keyboard layout
- **REQ-3.1.13**: DELETE /api/devices/:id - Forget/remove device
- **REQ-3.1.14**: Device filtering by type, status

#### 3.1.3 Profile Management (9 endpoints)
- **REQ-3.1.15**: GET /api/profiles - List all profiles
- **REQ-3.1.16**: GET /api/profiles/active - Get active profile
- **REQ-3.1.17**: GET /api/profiles/:name - Get profile configuration
- **REQ-3.1.18**: POST /api/profiles - Create new profile
- **REQ-3.1.19**: PUT /api/profiles/:name - Update profile configuration
- **REQ-3.1.20**: POST /api/profiles/:name/activate - Activate profile
- **REQ-3.1.21**: DELETE /api/profiles/:name - Delete profile
- **REQ-3.1.22**: POST /api/profiles/:name/duplicate - Clone profile
- **REQ-3.1.23**: PUT /api/profiles/:name/rename - Rename profile
- **REQ-3.1.24**: POST /api/profiles/:name/validate - Validate profile syntax

#### 3.1.4 Configuration & Layers (5 endpoints)
- **REQ-3.1.25**: GET /api/config - Get full configuration
- **REQ-3.1.26**: PUT /api/config - Update configuration
- **REQ-3.1.27**: POST /api/config/key-mappings - Add key mapping
- **REQ-3.1.28**: DELETE /api/config/key-mappings/:id - Remove key mapping
- **REQ-3.1.29**: GET /api/layers - List all layers

#### 3.1.5 Keyboard Layouts (2 endpoints)
- **REQ-3.1.30**: GET /api/layouts - List available layouts
- **REQ-3.1.31**: GET /api/layouts/:name - Get specific layout details

#### 3.1.6 Macro Recorder (4 endpoints)
- **REQ-3.1.32**: POST /api/macros/start-recording - Start recording
- **REQ-3.1.33**: POST /api/macros/stop-recording - Stop recording
- **REQ-3.1.34**: GET /api/macros/recorded-events - Get recorded events
- **REQ-3.1.35**: POST /api/macros/clear - Clear recorded macros

#### 3.1.7 Simulator (2 endpoints)
- **REQ-3.1.36**: POST /api/simulator/events - Simulate keyboard events
- **REQ-3.1.37**: POST /api/simulator/reset - Reset simulator state

#### 3.1.8 WebSocket (1 connection)
- **REQ-3.1.38**: GET /ws - WebSocket connection for real-time updates
- **REQ-3.1.39**: Event subscriptions (device changes, profile changes, metrics)
- **REQ-3.1.40**: Connection resilience (reconnect, heartbeat)

### 3.2 Test Scenario Coverage

Each endpoint must have:
- **REQ-3.2.1**: Success case (200/201 response)
- **REQ-3.2.2**: Empty state case (e.g., no devices, no profiles)
- **REQ-3.2.3**: Error case (404 not found, 409 conflict, 400 invalid)
- **REQ-3.2.4**: Edge cases (invalid input, concurrent requests, large payloads)

**Minimum Test Cases**: 60+ (40+ endpoints × 1.5 avg scenarios per endpoint)

### 3.3 Fix Existing Tests

#### 3.3.1 Dependency Issues
- **REQ-3.3.1**: Install missing `zod` dependency in root package.json or scripts
- **REQ-3.3.2**: Verify all imports resolve correctly
- **REQ-3.3.3**: Ensure tests are executable via `npm run test:e2e:auto`

#### 3.3.2 Test Assertion Fixes
- **REQ-3.3.4**: Fix loose assertions (check all response fields, not just existence)
- **REQ-3.3.5**: Add proper error code validation (PROFILE_NOT_FOUND, etc.)
- **REQ-3.3.6**: Validate response structure matches Zod schemas

#### 3.3.3 Test Reliability
- **REQ-3.3.7**: Ensure proper cleanup (delete created profiles/devices)
- **REQ-3.3.8**: Avoid test interdependencies (each test isolated)
- **REQ-3.3.9**: Handle daemon startup race conditions

### 3.4 Feature Validation via API

Tests must validate end-to-end feature workflows:

#### 3.4.1 Profile Workflow
- **REQ-3.4.1**: Create profile → Edit config → Activate → Verify active → Delete
- **REQ-3.4.2**: Duplicate profile → Rename → Activate → Delete both
- **REQ-3.4.3**: Validate profile → Fix syntax errors → Validate again → Activate

#### 3.4.2 Device Management Workflow
- **REQ-3.4.4**: List devices → Rename device → Verify name change
- **REQ-3.4.5**: Disable device → Verify not receiving events → Re-enable → Verify events
- **REQ-3.4.6**: Change device layout → Verify layout applied

#### 3.4.3 Config & Mapping Workflow
- **REQ-3.4.7**: Get config → Add key mapping → Verify mapping exists → Delete mapping
- **REQ-3.4.8**: List layers → Verify layer structure → Update config → Verify layers updated

#### 3.4.4 Macro Recording Workflow
- **REQ-3.4.9**: Start recording → Simulate events → Stop recording → Get events → Clear
- **REQ-3.4.10**: Start recording → Stop without events → Verify empty → Clear

#### 3.4.5 Simulator Workflow
- **REQ-3.4.11**: Simulate key press → Verify event in metrics → Reset simulator
- **REQ-3.4.12**: Simulate key sequence → Verify mapping applied → Check output events

## 4. Non-Functional Requirements

### 4.1 Performance
- **REQ-4.1.1**: Full test suite completes in < 3 minutes (60+ tests)
- **REQ-4.1.2**: Individual test completes in < 5 seconds
- **REQ-4.1.3**: Daemon startup < 10 seconds
- **REQ-4.1.4**: API response time < 500ms (95th percentile)

### 4.2 Reliability
- **REQ-4.2.1**: Zero flaky tests (deterministic execution)
- **REQ-4.2.2**: Tests pass 100% of time on clean daemon
- **REQ-4.2.3**: Tests recover from daemon crashes gracefully
- **REQ-4.2.4**: Tests cleanup resources on SIGINT/SIGTERM

### 4.3 Maintainability
- **REQ-4.3.1**: All files < 500 lines (excluding comments)
- **REQ-4.3.2**: All functions < 50 lines
- **REQ-4.3.3**: Test coverage ≥ 80% (test infrastructure code)
- **REQ-4.3.4**: Clear documentation for adding new tests

### 4.4 Usability
- **REQ-4.4.1**: Single command to run: `npm run test:e2e:auto`
- **REQ-4.4.2**: Clear progress reporting during execution
- **REQ-4.4.3**: Actionable error messages with diffs
- **REQ-4.4.4**: HTML report for visual inspection
- **REQ-4.4.5**: JSON report for CI/CD integration

### 4.5 CI/CD Integration
- **REQ-4.5.1**: Run on GitHub Actions (ubuntu-latest)
- **REQ-4.5.2**: Upload test results as artifacts
- **REQ-4.5.3**: Comment test summary on PRs
- **REQ-4.5.4**: Fail PR if tests fail
- **REQ-4.5.5**: Cache dependencies for faster runs

## 5. Out of Scope

- ❌ Browser UI testing (covered by separate Playwright specs)
- ❌ Performance profiling/load testing
- ❌ Security testing (fuzzing, penetration testing)
- ❌ Manual test execution
- ❌ Visual regression testing

## 6. Acceptance Criteria

### 6.1 Coverage
- ✅ All 40+ REST endpoints have at least 1 test case
- ✅ Minimum 60 test cases total
- ✅ All feature workflows validated end-to-end
- ✅ WebSocket connection tested

### 6.2 Quality
- ✅ All tests pass on clean daemon
- ✅ Zero flaky tests (100 consecutive runs)
- ✅ Test suite completes in < 3 minutes
- ✅ Clear error messages with diffs

### 6.3 CI Integration
- ✅ Tests run on GitHub Actions
- ✅ Test results uploaded as artifacts
- ✅ PR comments with test summary
- ✅ Workflow fails if tests fail

### 6.4 Documentation
- ✅ README with quick start guide
- ✅ Developer guide for adding tests
- ✅ Example test case with comments
- ✅ Troubleshooting guide

## 7. Dependencies

### 7.1 Runtime
- Node.js 18+
- TypeScript 5+
- keyrx_daemon (release binary)

### 7.2 Libraries
- `zod` - Schema validation
- `axios` or `fetch` - HTTP client
- `ws` - WebSocket client
- `chalk` - Console colors
- `commander` - CLI parsing
- `deep-diff` - Object comparison

### 7.3 Development
- `tsx` - TypeScript execution
- `@types/node` - Node.js types
- `vitest` - Test framework (for testing test infrastructure)

## 8. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Flaky tests | High | Enforce determinism, sequential execution, proper cleanup |
| Slow tests | Medium | Set strict timeouts, parallelize where safe, optimize fixtures |
| Daemon crashes | Medium | Graceful error handling, restart capability, collect logs |
| Missing endpoints | High | Comprehensive audit of daemon routes before implementation |
| Schema changes | Medium | Use Zod schemas from keyrx_ui, validate against daemon responses |
| CI resource limits | Low | Optimize test execution, cache dependencies, timeout workflow |

## 9. Success Metrics

### Quantitative
- ✅ 100% endpoint coverage (40+/40+)
- ✅ ≥60 test cases
- ✅ 100% test pass rate
- ✅ < 3 minute execution time
- ✅ 0 flaky tests

### Qualitative
- ✅ Easy to add new tests (< 20 lines per test)
- ✅ Clear failure diagnostics
- ✅ Comprehensive documentation
- ✅ Positive developer feedback

## 10. Timeline Estimate

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| Phase 1: Fix Existing Tests | 3 tasks | 2-3 hours |
| Phase 2: Add Missing Endpoints | 8 tasks | 1-2 days |
| Phase 3: Feature Workflows | 5 tasks | 1 day |
| Phase 4: WebSocket Testing | 3 tasks | 4-6 hours |
| Phase 5: CI Integration | 2 tasks | 2-3 hours |
| Phase 6: Documentation | 3 tasks | 2-3 hours |
| **Total** | **24 tasks** | **3-4 days** |

## 11. Traceability Matrix

| Category | Requirements | Coverage |
|----------|--------------|----------|
| Health & Metrics | REQ-3.1.1 to REQ-3.1.7 | 7 endpoints |
| Devices | REQ-3.1.8 to REQ-3.1.14 | 7 endpoints |
| Profiles | REQ-3.1.15 to REQ-3.1.24 | 10 endpoints |
| Config & Layers | REQ-3.1.25 to REQ-3.1.29 | 5 endpoints |
| Layouts | REQ-3.1.30 to REQ-3.1.31 | 2 endpoints |
| Macros | REQ-3.1.32 to REQ-3.1.35 | 4 endpoints |
| Simulator | REQ-3.1.36 to REQ-3.1.37 | 2 endpoints |
| WebSocket | REQ-3.1.38 to REQ-3.1.40 | 3 tests |
| **Total** | **40 requirements** | **40 endpoints + 3 WS tests** |
