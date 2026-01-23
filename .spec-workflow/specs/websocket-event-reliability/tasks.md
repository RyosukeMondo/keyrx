# Tasks: WebSocket Event Notification Reliability

## Overview

Fix 2 failing WebSocket event notification tests by adding event publishing to device and profile update endpoints.

**Total Tasks:** 6
**Estimated Time:** 2-3 hours

---

- [x] 1. Add event publishing to device update endpoint
  - File: keyrx_daemon/src/web/api/devices.rs
  - Add `event_tx.send(DaemonEvent::DeviceUpdated)` after successful device update in `update_device_config` handler
  - Send event asynchronously without blocking API response
  - Purpose: Notify WebSocket clients when device configuration changes
  - _Leverage: keyrx_daemon/src/web/ws.rs (event bus pattern), keyrx_daemon/src/daemon/events.rs (DaemonEvent enum)_
  - _Requirements: REQ-3.2.1, REQ-3.2.2, REQ-3.2.3_
  - _Prompt: Role: Backend Developer with expertise in Rust async/await and event-driven architecture | Task: Add event publishing to device update endpoint in update_device_config handler following REQ-3.2.x, using existing event bus pattern from ws.rs and DaemonEvent enum | Restrictions: Must not block API response, must handle channel send errors gracefully, must include device_id and updated fields in event payload | Success: Device updates publish events to event bus, events contain correct payload, API response time not affected_

- [x] 2. Add event publishing to profile activation endpoint
  - File: keyrx_daemon/src/web/api/profiles.rs
  - Add `event_tx.send(DaemonEvent::ProfileActivated)` after successful profile activation in `activate_profile` handler
  - Send event asynchronously without blocking API response
  - Purpose: Notify WebSocket clients when active profile changes
  - _Leverage: keyrx_daemon/src/web/ws.rs (event bus pattern), keyrx_daemon/src/daemon/events.rs (DaemonEvent enum)_
  - _Requirements: REQ-3.3.1, REQ-3.3.2, REQ-3.3.3_
  - _Prompt: Role: Backend Developer with expertise in Rust async/await and event-driven architecture | Task: Add event publishing to profile activation endpoint in activate_profile handler following REQ-3.3.x, using existing event bus pattern from ws.rs and DaemonEvent enum | Restrictions: Must not block API response, must handle channel send errors gracefully, must include profile name in event payload | Success: Profile activations publish events to event bus, events contain correct payload, API response time not affected_

- [x] 3. Verify WebSocket handler event processing
  - File: keyrx_daemon/src/web/ws.rs
  - Review WebSocket handler to ensure it properly receives and broadcasts events from event_rx channel
  - Add debug logging for event reception
  - Purpose: Confirm events flow from event bus to WebSocket clients
  - _Leverage: keyrx_daemon/src/web/ws.rs (existing WebSocket handler)_
  - _Requirements: REQ-3.1.1, REQ-3.1.2_
  - _Prompt: Role: Backend Developer with expertise in WebSocket protocols and Rust tokio | Task: Review and verify WebSocket handler event processing following REQ-3.1.x, ensuring events from event_rx channel are properly received and broadcast to subscribed clients | Restrictions: Must maintain existing subscription filtering, must not introduce race conditions, must log events for debugging | Success: WebSocket handler receives events from event bus, events are broadcast to all subscribed clients, debug logs confirm event flow_

- [x] 4. Test device update event notification
  - File: Run existing test: scripts/test-cases/websocket.tests.ts (websocket-003)
  - Execute test after implementing event publishing
  - Verify test passes and event received within 1 second
  - Purpose: Validate device update event notification works end-to-end
  - _Leverage: scripts/test-cases/websocket.tests.ts (existing test), scripts/api-client/websocket-client.ts_
  - _Requirements: REQ-3.2.1, REQ-3.2.2, REQ-3.2.3, REQ-4.1.1_
  - _Prompt: Role: QA Engineer with expertise in E2E testing and WebSocket protocols | Task: Execute and verify websocket-003 test passes after event publishing implementation following REQ-3.2.x and REQ-4.1.1, measuring event delivery latency | Restrictions: Must run test 10 times to verify no flakiness, must measure and log event delivery latency, must verify event payload correctness | Success: Test passes consistently, event delivery latency < 100ms, event payload contains correct device ID and updates_

- [x] 5. Test profile activation event notification
  - File: Run existing test: scripts/test-cases/websocket.tests.ts (websocket-004)
  - Execute test after implementing event publishing
  - Verify test passes and event received within 1 second
  - Purpose: Validate profile activation event notification works end-to-end
  - _Leverage: scripts/test-cases/websocket.tests.ts (existing test), scripts/api-client/websocket-client.ts_
  - _Requirements: REQ-3.3.1, REQ-3.3.2, REQ-3.3.3, REQ-4.1.1_
  - _Prompt: Role: QA Engineer with expertise in E2E testing and WebSocket protocols | Task: Execute and verify websocket-004 test passes after event publishing implementation following REQ-3.3.x and REQ-4.1.1, measuring event delivery latency | Restrictions: Must run test 10 times to verify no flakiness, must measure and log event delivery latency, must verify event payload correctness | Success: Test passes consistently, event delivery latency < 100ms, event payload contains correct profile name_

- [x] 6. Run full WebSocket test suite
  - File: Run all tests: scripts/test-cases/websocket.tests.ts
  - Execute complete WebSocket test suite (5 tests)
  - Verify 100% pass rate (5/5)
  - Purpose: Ensure no regressions and all WebSocket functionality works
  - _Leverage: scripts/test-cases/websocket.tests.ts, scripts/automated-e2e-test.ts_
  - _Requirements: All REQ-3.x, REQ-4.x_
  - _Prompt: Role: QA Engineer with expertise in integration testing and test automation | Task: Execute complete WebSocket test suite and verify 100% pass rate following all requirements, running multiple times to ensure reliability | Restrictions: Must test all 5 WebSocket tests, must run 10 consecutive times, must verify no flaky failures | Success: All 5 WebSocket tests pass, zero flaky failures in 10 runs, event delivery reliable and fast_

---

## Task Dependencies

```
1 (Device event publishing) ──┐
                              ├──→ 3 (Verify WebSocket handler) ──→ 4 (Test device events)
2 (Profile event publishing) ─┘                                    └──→ 6 (Full suite)
                                                                    ┌─→
                                                                    5 (Test profile events)
```

## Success Criteria

- ✅ All 6 tasks complete
- ✅ websocket-003 and websocket-004 tests pass
- ✅ 100% WebSocket test pass rate (5/5)
- ✅ Event delivery latency < 100ms
- ✅ Zero flaky failures in 10 consecutive test runs

## Verification Checklist

- [ ] Run `npx tsx scripts/automated-e2e-test.ts --filter=websocket` - all 5 tests pass
- [ ] Run tests 10 consecutive times - zero failures
- [ ] Measure event latency - < 100ms (p95)
- [ ] Check logs - events published and received correctly
- [ ] Verify no API performance regression

## Notes

- **Estimated Time:** 2-3 hours (simple fix, just add event publishing)
- **Priority:** Medium (improves test coverage but doesn't block features)
- **Risk:** Low (event publishing is non-blocking, won't affect API)
