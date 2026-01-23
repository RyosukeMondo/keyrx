# Tasks: Simulator-Macro Recorder Integration

## Overview

Connect simulator event output to macro recorder input, enabling macro recording of simulated keyboard events.

**Total Tasks:** 5
**Estimated Time:** 3-4 hours

---

- [x] 1. Add event bus sender to SimulatorService
  - File: keyrx_daemon/src/services/simulator_service.rs
  - Add `event_tx: mpsc::Sender<KeyEvent>` field to SimulatorService struct
  - Update constructor to accept event_tx parameter
  - Purpose: Enable simulator to send events to event bus
  - _Leverage: keyrx_daemon/src/daemon/mod.rs (event bus pattern), tokio::sync::mpsc_
  - _Requirements: REQ-3.1.1_
  - _Prompt: Role: Rust Backend Developer with expertise in async channels and event-driven architecture | Task: Add event bus sender field to SimulatorService struct following REQ-3.1.1, using tokio mpsc channel pattern from daemon initialization | Restrictions: Must use existing event bus type, must not block on channel send, must handle send errors gracefully | Success: SimulatorService has event_tx field, constructor accepts event_tx, compiles without errors_

- [x] 2. Send simulated events to event bus
  - File: keyrx_daemon/src/services/simulator_service.rs
  - Update simulate_events() method to send KeyEvent to event_tx after generating events
  - Add timestamp to each event (microseconds since epoch)
  - Convert SimulatorEvent format to KeyEvent format with lowercase event types
  - Purpose: Route simulated events through event bus to macro recorder
  - _Leverage: keyrx_daemon/src/services/simulator_service.rs (existing simulate_events), std::time::SystemTime_
  - _Requirements: REQ-3.1.1, REQ-3.2.1, REQ-3.2.2, REQ-3.2.3_
  - _Prompt: Role: Rust Developer with expertise in event processing and time handling | Task: Update simulate_events method to send events to event bus following REQ-3.1.x and REQ-3.2.x, converting event format and adding timestamps | Restrictions: Must convert event_type to lowercase, must add timestamp_us field, must handle channel send errors, must not block API response | Success: Simulated events sent to event bus with correct format, timestamps accurate, no blocking_

- [x] 3. Connect macro recorder to event bus
  - File: keyrx_daemon/src/macro_recorder.rs
  - Add event_rx receiver parameter to macro recorder
  - Create async event loop that receives from event_rx and calls record_event() when recording active
  - Purpose: Enable macro recorder to receive events from event bus (both simulated and physical)
  - _Leverage: keyrx_daemon/src/macro_recorder.rs (existing MacroRecorder), tokio::sync::mpsc_
  - _Requirements: REQ-3.1.2, REQ-3.1.3, REQ-3.3.1_
  - _Prompt: Role: Rust Async Developer with expertise in tokio and event processing | Task: Add event loop to macro recorder that receives from event bus following REQ-3.1.x and REQ-3.3.1, only recording when recording flag active | Restrictions: Must check is_recording() before recording, must not block event loop, must handle channel closure gracefully | Success: Macro recorder receives events from bus, only records when active, event loop runs efficiently_

- [x] 4. Wire up event bus in daemon initialization
  - File: keyrx_daemon/src/daemon/mod.rs, keyrx_daemon/src/cli/run.rs
  - Create event bus channel (mpsc) at daemon startup
  - Pass event_tx to SimulatorService constructor
  - Pass event_rx to macro recorder event loop
  - Spawn macro recorder event loop as async task
  - Purpose: Complete event flow integration at system level
  - _Leverage: keyrx_daemon/src/daemon/mod.rs (daemon initialization), tokio::sync::mpsc_
  - _Requirements: REQ-3.1.1, REQ-3.1.2, REQ-3.1.3, REQ-4.1.1, REQ-4.1.2_
  - _Prompt: Role: Systems Engineer with expertise in Rust tokio and application architecture | Task: Wire up event bus in daemon initialization following REQ-3.1.x and REQ-4.1.x, creating channel and connecting simulator to macro recorder | Restrictions: Must use bounded channel with capacity 1000, must spawn macro recorder loop, must handle task cleanup on shutdown | Success: Event bus connects simulator to macro recorder, channel capacity 1000, event loop spawned correctly_

- [x] 5. Test simulator-macro integration
  - File: Run existing test: scripts/test-cases/workflows.tests.ts (workflow-006)
  - Execute workflow-006 test after integration: start recording → simulate events → stop → get events
  - Verify test passes and recorded events include simulated events with correct format
  - Run test 10 times to ensure no flakiness
  - Purpose: Validate simulator-macro integration works end-to-end
  - _Leverage: scripts/test-cases/workflows.tests.ts, scripts/automated-e2e-test.ts_
  - _Requirements: All requirements, acceptance criteria_
  - _Prompt: Role: QA Engineer with expertise in E2E testing and workflow validation | Task: Execute workflow-006 test and verify simulated events are recorded following all requirements, running 10 times to ensure reliability | Restrictions: Must verify event format (lowercase event_type, key field, timestamp_us), must check event ordering, must ensure no flaky failures | Success: workflow-006 test passes, events recorded with correct format, zero flaky failures in 10 runs_

---

## Task Dependencies

```
1 (Add event_tx to SimulatorService)
    ↓
2 (Send events to bus)
    ↓
3 (Connect macro recorder to bus) ──→ 4 (Wire up in daemon init)
                                          ↓
                                      5 (Test integration)
```

## Success Criteria

- ✅ All 5 tasks complete
- ✅ `workflow-006` test passes
- ✅ Simulated events recorded by macro recorder
- ✅ Event format correct (lowercase event_type, timestamp_us)
- ✅ Event ordering preserved
- ✅ Zero flaky test failures (10 consecutive runs)

## Verification Checklist

- [x] Run `cargo build --release` - compiles without errors
- [x] Start daemon and macro recording
- [x] POST /api/simulator/events with test events
- [x] GET /api/macros/recorded-events - returns simulated events
- [x] Run `npx tsx scripts/automated-e2e-test.ts --filter=workflow-006` - test passes
- [x] Run test 10 consecutive times - zero failures
- [x] Verify event format: {key: string, event_type: string, timestamp_us: number}
- [x] Check event_type values are lowercase ('press', 'release')

## Notes

- **Estimated Time:** 3-4 hours (straightforward integration)
- **Priority:** Low (only affects 1 test, architectural improvement)
- **Risk:** Low (isolated change, doesn't affect production features)
- **Benefit:** Completes E2E test suite, validates simulator-macro architecture
