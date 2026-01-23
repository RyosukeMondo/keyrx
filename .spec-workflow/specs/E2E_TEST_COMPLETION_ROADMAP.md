# E2E Test Completion Roadmap

**Date:** 2026-01-22
**Current Status:** 75/83 tests passing (90.4%)
**Target:** 83/83 tests passing (100%)

---

## Executive Summary

The REST API E2E testing infrastructure is 90.4% complete with 75/83 tests passing. The remaining 8 test failures (9.6%) are due to **architectural gaps, not bugs**:

- **5 IPC-dependent tests** - Require full daemon with IPC socket
- **2 WebSocket event tests** - Event notification not wired up
- **1 architectural limitation** - Simulator doesn't feed macro recorder

Three laser-focused specs address these gaps, requiring **2-3 days total effort** to achieve 100% test coverage.

---

## Current State Analysis

### Test Results Breakdown

| Category | Passing | Total | Pass Rate | Status |
|----------|---------|-------|-----------|--------|
| Config | 11 | 11 | 100% | ✅ Complete |
| Devices | 15 | 15 | 100% | ✅ Complete |
| Health | 3 | 3 | 100% | ✅ Complete |
| Layouts | 3 | 3 | 100% | ✅ Complete |
| Macros | 8 | 8 | 100% | ✅ Complete |
| Metrics | 4 | 4 | 100% | ✅ Complete |
| Profiles | 19 | 20 | 95% | ⚠️ 1 failure (IPC) |
| Simulator | 7 | 7 | 100% | ✅ Complete |
| Status | 0 | 1 | 0% | ❌ 1 failure (IPC) |
| Workflows | 2 | 6 | 33% | ❌ 4 failures (IPC + arch) |
| Websocket | 3 | 5 | 60% | ⚠️ 2 failures (events) |
| **TOTAL** | **75** | **83** | **90.4%** | ⚠️ In Progress |

### Failure Analysis

**IPC-Dependent Failures (5 tests - 6.0%):**
- `status-001` - GET /api/status (daemon_running field)
- `integration-001` - Profile lifecycle
- `workflow-002` - Profile duplicate→rename→activate
- `workflow-003` - Profile validation→fix→activate
- `workflow-007` - Simulator event → mapping → output

**WebSocket Event Failures (2 tests - 2.4%):**
- `websocket-003` - Device update event notification
- `websocket-004` - Profile activation event notification

**Architectural Limitation (1 test - 1.2%):**
- `workflow-006` - Macro record → simulate → playback

---

## Solution Specs

### Spec 1: WebSocket Event Reliability
**Location:** `.spec-workflow/specs/websocket-event-reliability/`
**Impact:** +2 tests (90.4% → 92.8%)
**Effort:** 2-3 hours
**Priority:** Medium

**Problem:** Device/profile updates don't publish events to event bus.

**Solution:** Add `event_tx.send()` after operations.

**Tasks:**
1. Add event publishing to device update endpoint
2. Add event publishing to profile activation endpoint
3. Verify WebSocket handler event processing
4. Test device/profile event notifications
5. Run full WebSocket suite

**Risk:** Low (non-blocking event publishing)

---

### Spec 2: IPC Test Mode
**Location:** `.spec-workflow/specs/ipc-test-mode/`
**Impact:** +5 tests (90.4% → 96.4%)
**Effort:** 1-2 days
**Priority:** High

**Problem:** Tests require full daemon with IPC socket for profile activation.

**Solution:** Add `--test-mode` flag with IPC infrastructure.

**Tasks:**
1. Add --test-mode CLI flag
2. Create IPC module (server, client, commands)
3. Implement IPC server with Unix socket
4. Implement profile activation via IPC
5. Implement daemon status query via IPC
6. Integrate IPC with REST API handlers
7. Update daemon startup for test mode
8. Test all IPC-dependent tests

**Risk:** Medium (new IPC infrastructure)

---

### Spec 3: Simulator-Macro Integration
**Location:** `.spec-workflow/specs/simulator-macro-integration/`
**Impact:** +1 test (96.4% → 100%)
**Effort:** 3-4 hours
**Priority:** Low

**Problem:** Simulator doesn't feed events to macro recorder.

**Solution:** Route simulator events through event bus.

**Tasks:**
1. Add event_tx to SimulatorService
2. Send simulated events to event bus
3. Connect macro recorder to event bus
4. Wire up in daemon initialization
5. Test simulator-macro integration

**Risk:** Low (isolated integration)

---

## Implementation Roadmap

### Phase 1: WebSocket Events (2-3 hours)
**Priority:** Medium | **Impact:** +2 tests

```
Week 1, Days 1-2
├─ Task 1: Add device update event publishing (30 min)
├─ Task 2: Add profile activation event publishing (30 min)
├─ Task 3: Verify WebSocket handler (30 min)
├─ Task 4-5: Test device/profile notifications (45 min)
└─ Task 6: Run full WebSocket suite (15 min)
```

**Deliverable:** 77/83 tests passing (92.8%)

---

### Phase 2: IPC Test Mode (1-2 days)
**Priority:** High | **Impact:** +5 tests

```
Week 1, Days 3-4
├─ Task 1: Add --test-mode CLI flag (2 hours)
├─ Task 2-3: Create IPC server infrastructure (4 hours)
├─ Task 4-5: Implement IPC commands (4 hours)
├─ Task 6: Integrate with REST API (2 hours)
├─ Task 7: Update daemon startup (2 hours)
└─ Task 8: Test IPC-dependent tests (2 hours)
```

**Deliverable:** 82/83 tests passing (98.8%)

---

### Phase 3: Simulator-Macro Integration (3-4 hours)
**Priority:** Low | **Impact:** +1 test

```
Week 2, Day 1
├─ Task 1: Add event_tx to SimulatorService (45 min)
├─ Task 2: Send events to event bus (1 hour)
├─ Task 3: Connect macro recorder (1 hour)
├─ Task 4: Wire up in daemon init (45 min)
└─ Task 5: Test integration (30 min)
```

**Deliverable:** 83/83 tests passing (100%) ✅

---

## Total Effort Estimate

| Phase | Tasks | Effort | Impact |
|-------|-------|--------|--------|
| Phase 1: WebSocket | 6 | 2-3 hours | +2 tests |
| Phase 2: IPC Test Mode | 8 | 1-2 days | +5 tests |
| Phase 3: Simulator-Macro | 5 | 3-4 hours | +1 test |
| **TOTAL** | **19 tasks** | **2-3 days** | **+8 tests** |

**Timeline:**
- **Optimistic:** 2 days (16 hours focused work)
- **Realistic:** 2.5 days (20 hours with testing)
- **Conservative:** 3 days (24 hours with documentation)

---

## Success Metrics

### Quantitative
- ✅ 100% E2E test pass rate (83/83)
- ✅ < 30 seconds test execution time (currently ~21s)
- ✅ Zero flaky test failures
- ✅ 100% endpoint coverage (30/30 REST endpoints)

### Qualitative
- ✅ WebSocket events reliable and fast (< 100ms)
- ✅ IPC infrastructure production-ready
- ✅ Simulator-macro integration complete
- ✅ Full test suite runs in CI/CD

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| IPC implementation complexity | Medium | Start with minimal IPC, iterate |
| Test flakiness | Low | Run 10x after each phase |
| Performance regression | Low | Measure latency, non-blocking |
| Security concerns | Low | Test mode debug-only, local socket |
| Schedule slippage | Low | Phases independent, can defer Phase 3 |

---

## Dependency Graph

```
Phase 1: WebSocket Events (parallel, no dependencies)
    ├─ Task 1-2: Add event publishing
    ├─ Task 3: Verify handler
    └─ Task 4-6: Test

Phase 2: IPC Test Mode (sequential dependencies)
    Task 1 → Task 2 → Task 3 → Tasks 4-5 → Task 6 → Task 7 → Task 8

Phase 3: Simulator-Macro (sequential dependencies)
    Task 1 → Task 2 → Task 3 → Task 4 → Task 5
```

**Parallelization:** Phase 1 can start immediately. Phases 2-3 independent.

---

## Rollback Strategy

### If WebSocket Events Fail
- Remove event_tx.send() calls
- Tests fail but REST API unaffected
- No production impact

### If IPC Test Mode Fails
- Remove --test-mode flag
- Mark 5 tests as skipped
- Document limitation
- No production impact

### If Simulator-Macro Fails
- Remove event bus connection
- 1 test fails but features work
- No production impact

---

## Post-Completion Checklist

- [ ] All 83 tests pass (100%)
- [ ] Run tests 100 consecutive times - zero flaky failures
- [ ] Measure latencies:
  - [ ] WebSocket event delivery < 100ms
  - [ ] IPC round-trip < 50ms
  - [ ] Simulator-macro delivery < 1ms
- [ ] Update documentation:
  - [ ] README with --test-mode usage
  - [ ] DEV_GUIDE with IPC architecture
  - [ ] TROUBLESHOOTING with common issues
- [ ] CI/CD integration:
  - [ ] Update GitHub Actions workflow
  - [ ] Add test-mode flag to CI runs
  - [ ] Upload test results as artifacts
- [ ] Security review:
  - [ ] Verify --test-mode debug-only
  - [ ] Check IPC socket permissions
  - [ ] Audit event bus security

---

## References

- **REST API E2E Testing (Complete):** `.spec-workflow/specs/rest-api-comprehensive-e2e/`
- **WebSocket Events Spec:** `.spec-workflow/specs/websocket-event-reliability/`
- **IPC Test Mode Spec:** `.spec-workflow/specs/ipc-test-mode/`
- **Simulator-Macro Spec:** `.spec-workflow/specs/simulator-macro-integration/`

---

**Generated:** 2026-01-22
**Target Completion:** 2026-01-24 (2-3 days)
**Expected Outcome:** 100% E2E test coverage, production-ready test infrastructure
