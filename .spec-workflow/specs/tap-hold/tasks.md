# Tasks Document: Tap-Hold Functionality

## Phase 1: Core Infrastructure

- [x] 1. Add timestamp field to KeyEvent structure
  - File: `keyrx_core/src/runtime/event.rs`
  - Add `timestamp_us: u64` field to KeyEvent struct
  - Update all KeyEvent constructors to accept timestamp
  - Add `KeyEvent::with_timestamp()` helper
  - Purpose: Enable timing-based decisions in event processing
  - _Leverage: keyrx_core/src/runtime/event.rs (existing KeyEvent)_
  - _Requirements: 4.3, 4.4_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in no_std embedded systems | Task: Add timestamp_us field to KeyEvent struct in keyrx_core/src/runtime/event.rs, update constructors, ensure no_std compatibility | Restrictions: Must maintain no_std, no heap allocation, backward compatible with existing code | Success: KeyEvent has timestamp field, all existing code compiles, no performance regression | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [x] 2. Create Clock trait for time abstraction
  - File: `keyrx_core/src/runtime/clock.rs` (new file)
  - Define Clock trait with `now(&self) -> u64` method
  - Implement SystemClock (uses timestamp from events)
  - Implement VirtualClock for testing (manually advanceable)
  - Purpose: Enable deterministic testing with virtual time
  - _Leverage: None (new abstraction)_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with testing expertise | Task: Create Clock trait in new file keyrx_core/src/runtime/clock.rs with SystemClock and VirtualClock implementations | Restrictions: Must be no_std compatible, VirtualClock must be thread-safe for parallel tests | Success: Clock trait defined, both implementations work, VirtualClock can advance time programmatically | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [x] 3. Create TapHoldState enum and struct
  - File: `keyrx_core/src/runtime/tap_hold.rs` (new file)
  - Define TapHoldPhase enum: Idle, Pending, Hold
  - Define TapHoldState struct with key, phase, config fields
  - Implement state transition methods
  - Purpose: Represent individual tap-hold key state
  - _Leverage: keyrx_core/src/config/mappings.rs (BaseKeyMapping::TapHold)_
  - _Requirements: 3.1, 3.2, 3.4_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with state machine expertise | Task: Create TapHoldPhase enum and TapHoldState struct in new file keyrx_core/src/runtime/tap_hold.rs with state transition logic | Restrictions: Use Copy types only, no heap, match existing code style | Success: State transitions are clear and tested, no invalid state combinations possible | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

## Phase 2: State Machine Implementation

- [x] 4. Implement PendingKeyRegistry
  - File: `keyrx_core/src/runtime/tap_hold.rs` (extend)
  - Create PendingKeyRegistry<const N: usize> using ArrayVec
  - Implement add, remove, get, get_mut, iter methods
  - Implement check_timeouts for batch timeout processing
  - Purpose: Track multiple concurrent tap-hold keys
  - _Leverage: arrayvec crate (already in dependencies)_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in embedded collections | Task: Implement PendingKeyRegistry using ArrayVec in keyrx_core/src/runtime/tap_hold.rs with O(n) lookup but cache-friendly iteration | Restrictions: Max 32 concurrent tap-holds, no heap, handle registry full gracefully | Success: Registry works with concurrent keys, timeout checking is efficient | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [x] 5. Implement TapHoldProcessor core logic
  - File: `keyrx_core/src/runtime/tap_hold.rs` (extend)
  - Create TapHoldProcessor struct
  - Implement process_event() for Press, Release, Repeat handling
  - Implement state transitions: Idle→Pending, Pending→Hold, Pending→Tap, Hold→Idle
  - Purpose: Central tap-hold event processing
  - _Leverage: keyrx_core/src/runtime/state.rs (DeviceState)_
  - _Requirements: 1.1, 1.2, 1.3, 3.1, 3.2_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with DFA/state machine expertise | Task: Implement TapHoldProcessor with process_event method handling all state transitions in keyrx_core/src/runtime/tap_hold.rs | Restrictions: Return ArrayVec of output events, integrate with DeviceState for modifier management | Success: All state transitions work correctly, output events match expected tap/hold behavior | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [x] 6. Implement Permissive Hold logic
  - File: `keyrx_core/src/runtime/tap_hold.rs` (extend)
  - Detect other key press while in Pending state
  - Immediately transition to Hold when interrupted
  - Activate hold modifier before processing interrupting key
  - Purpose: Enable natural typing flow with tap-hold keys
  - _Leverage: Task 5 (TapHoldProcessor)_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with keyboard firmware experience | Task: Add Permissive Hold logic to TapHoldProcessor - when any other key is pressed while tap-hold is Pending, immediately confirm Hold | Restrictions: Must activate modifier before returning so interrupting key sees active modifier | Success: Typing "CapsLock(hold)+A" produces Ctrl+A even if A pressed before threshold | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

## Phase 3: Integration

- [x] 7. Integrate TapHoldProcessor into process_event
  - File: `keyrx_core/src/runtime/event.rs`
  - Import and instantiate TapHoldProcessor
  - Replace stubbed TapHold match arm with processor call
  - Pass timestamp and state to processor
  - Purpose: Activate tap-hold in main event pipeline
  - _Leverage: keyrx_core/src/runtime/tap_hold.rs, keyrx_core/src/runtime/event.rs_
  - _Requirements: 1.1, 1.2, 1.3, 5.1_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with integration expertise | Task: Replace stubbed TapHold branch in process_event with actual TapHoldProcessor call in keyrx_core/src/runtime/event.rs | Restrictions: Maintain existing function signature, no breaking changes to other mapping types | Success: TapHold mappings now produce output events instead of empty vector | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [x] 8. Pass timestamps from daemon to core
  - File: `keyrx_daemon/src/platform/linux.rs`
  - Extract timestamp from evdev event
  - Convert to microseconds and pass to KeyEvent
  - Handle missing timestamps gracefully
  - Purpose: Provide accurate timing data to tap-hold processor
  - _Leverage: evdev crate timestamp API_
  - _Requirements: 4.3, 4.4_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Linux evdev experience | Task: Extract timestamps from evdev events in keyrx_daemon/src/platform/linux.rs and pass to KeyEvent constructor | Restrictions: Handle missing timestamps by falling back to monotonic clock, no panics | Success: KeyEvents from Linux have accurate timestamps in microseconds | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [x] 9. Add timer checking to daemon event loop
  - File: `keyrx_daemon/src/daemon/mod.rs`
  - Call check_timeouts() periodically (every 10ms or on event)
  - Process timeout-triggered Hold activations
  - Inject resulting events into output
  - Purpose: Detect threshold crossing for held keys
  - _Leverage: keyrx_core/src/runtime/tap_hold.rs (check_timeouts)_
  - _Requirements: 3.2_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with async/event loop expertise | Task: Add periodic check_timeouts call to daemon event loop in keyrx_daemon/src/daemon/mod.rs for threshold detection | Restrictions: Use non-blocking timeout check, batch process multiple timeouts | Success: Keys held past threshold activate Hold even without release event | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

## Phase 4: Testing

- [x] 10. Unit tests for state machine
  - File: `keyrx_core/src/runtime/tap_hold.rs` (tests module)
  - Test Tap path: Press → Release(quick) → outputs tap key
  - Test Hold path: Press → timeout → Hold active → Release → deactivate
  - Test edge cases: exact threshold, threshold ± 1μs
  - Purpose: Verify state machine correctness
  - _Leverage: VirtualClock from Task 2_
  - _Requirements: 1.1, 1.2, 1.3, 4.1_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with Rust testing expertise | Task: Write comprehensive unit tests for TapHoldProcessor in keyrx_core/src/runtime/tap_hold.rs covering tap, hold, and edge cases | Restrictions: Use VirtualClock for determinism, test both success and error paths | Success: 100% branch coverage on state machine, all timing edge cases covered | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [x] 11. Unit tests for Permissive Hold
  - File: `keyrx_core/src/runtime/tap_hold.rs` (tests module)
  - Test interrupted tap-hold confirms Hold immediately
  - Test modifier active before interrupted key processed
  - Test multiple concurrent tap-holds with interruption
  - Purpose: Verify Permissive Hold behavior
  - _Leverage: VirtualClock, Task 10 test patterns_
  - _Requirements: 3.1, 3.3_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with keyboard behavior expertise | Task: Write unit tests for Permissive Hold behavior - interrupting key press immediately confirms Hold | Restrictions: Verify modifier state before processing interrupting key | Success: Permissive Hold works correctly in all tested scenarios | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [ ] 12. Integration tests with compiled configs
  - File: `keyrx_core/tests/tap_hold_integration.rs` (new file)
  - Compile example tap_hold config to .krx
  - Load config and process simulated events
  - Verify correct output events
  - Purpose: Test full pipeline from config to output
  - _Leverage: keyrx_compiler, examples/04-dual-function-keys.rhai_
  - _Requirements: 2.1, 2.4, 5.1, 5.4_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration Test Engineer | Task: Create integration tests that compile tap_hold Rhai config, load .krx, and verify event processing in keyrx_core/tests/tap_hold_integration.rs | Restrictions: Use real compiler output, test realistic key sequences | Success: Integration tests pass with compiled configs, end-to-end behavior verified | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [ ] 13. Property-based tests for determinism
  - File: `keyrx_core/tests/tap_hold_proptest.rs` (new file)
  - Generate random key event sequences
  - Verify same sequence always produces same output
  - Test with varying thresholds and timing
  - Purpose: Ensure deterministic behavior under all inputs
  - _Leverage: proptest crate_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Property-Based Testing Expert | Task: Create proptest-based tests verifying tap-hold determinism - same input always produces same output | Restrictions: Use VirtualClock, generate realistic key sequences, run 10K+ cases | Success: No determinism violations found in 100K generated test cases | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

## Phase 5: Virtual E2E and Documentation

- [ ] 14. Virtual E2E tests for tap-hold
  - File: `keyrx_daemon/tests/tap_hold_e2e.rs` (new file)
  - Use VirtualKeyboard to inject tap-hold key presses
  - Use OutputCapture to verify output timing and keys
  - Test realistic usage patterns
  - Purpose: Verify end-to-end behavior with virtual devices
  - _Leverage: keyrx_daemon/src/test_utils/ (VirtualKeyboard, OutputCapture)_
  - _Requirements: 1.1, 1.2, 3.1, 5.3_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: E2E Test Engineer | Task: Create virtual E2E tests for tap-hold using VirtualKeyboard and OutputCapture in keyrx_daemon/tests/tap_hold_e2e.rs | Restrictions: Use skip_if_no_uinput macro for CI compatibility, test realistic sequences | Success: E2E tests pass locally with uinput access, skip gracefully in CI | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [ ] 15. Update example configurations
  - File: `examples/04-dual-function-keys.rhai`
  - Verify example works with new implementation
  - Add comments explaining tap vs hold behavior
  - Add edge case examples (multiple tap-holds, nested conditions)
  - Purpose: Provide working reference configurations
  - _Leverage: Existing example file_
  - _Requirements: 2.1, 2.4_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with keyboard expertise | Task: Update examples/04-dual-function-keys.rhai with working tap-hold examples and comprehensive comments | Restrictions: Ensure examples compile and work correctly, explain behavior clearly | Success: Example file is self-documenting, all configurations work as described | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [ ] 16. Add debug logging for state transitions
  - File: `keyrx_core/src/runtime/tap_hold.rs`
  - Add trace-level logging for state transitions
  - Include key, old state, new state, elapsed time
  - Compile out in release mode (cfg(debug_assertions))
  - Purpose: Enable debugging of tap-hold behavior
  - _Leverage: log crate (already in dependencies)_
  - _Requirements: Non-functional (Usability)_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Observability Engineer | Task: Add debug logging to TapHoldProcessor state transitions with key, state change, and timing info | Restrictions: Use trace! level, compile out in release, no performance impact in production | Success: Debug mode shows clear state transition logs, release mode has no overhead | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

## Phase 6: Final Verification

- [ ] 17. Run full test suite and verify coverage
  - Files: All test files
  - Run cargo test with coverage
  - Verify 80%+ coverage on tap_hold.rs
  - Fix any failing tests
  - Purpose: Ensure quality meets standards
  - _Leverage: cargo tarpaulin_
  - _Requirements: All_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Lead | Task: Run full test suite with coverage, verify 80%+ on tap_hold module, fix any issues | Restrictions: All tests must pass, coverage threshold must be met | Success: All tests green, coverage >= 80%, no regressions in other modules | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_

- [ ] 18. Manual UAT with real keyboard
  - Files: None (manual testing)
  - Build release daemon with tap-hold config
  - Test CapsLock as tap=Escape, hold=Ctrl
  - Verify timing feels natural (200ms threshold)
  - Purpose: Validate user experience
  - _Leverage: examples/04-dual-function-keys.rhai_
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Implement the task for spec tap-hold, first run spec-workflow-guide to get the workflow guide then implement the task: Role: User Experience Tester | Task: Perform manual UAT with real keyboard - test CapsLock tap-hold behavior, verify natural feel | Restrictions: Document any timing issues, suggest threshold adjustments if needed | Success: Tap-hold feels natural and responsive, no stuck modifiers, no missed inputs | Instructions: Set task status to [-] in tasks.md before starting, use log-implementation tool after completion with detailed artifacts, then mark [x] when done_
