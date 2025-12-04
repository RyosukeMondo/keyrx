# Tasks Document

## Phase 1: Infrastructure

- [x] 1. Create CriticalError type
  - File: `core/src/errors/critical.rs`
  - Define error variants with fallback actions
  - Implement is_recoverable and fallback_action
  - Purpose: Error type for critical path
  - _Leverage: thiserror_
  - _Requirements: 2.2, 2.3_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating error types | Task: Create CriticalError with fallback actions | Restrictions: All variants have recovery path, serializable | _Leverage: thiserror | Success: Error type covers all critical failures | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create CriticalResult type
  - File: `core/src/errors/critical_result.rs`
  - Implement without panic-inducing methods
  - Add unwrap_or_fallback and similar safe methods
  - Purpose: Result type that can't panic
  - _Leverage: Rust type system_
  - _Requirements: 2.1, 2.4_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create CriticalResult without unwrap/expect | Restrictions: No panic methods, must_use, safe fallbacks | _Leverage: Type system | Success: Type prevents panic at compile time | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create PanicGuard
  - File: `core/src/safety/panic_guard.rs`
  - Implement catch_unwind wrapper
  - Add backtrace capture and logging
  - Purpose: Catch panics in critical code
  - _Leverage: std::panic::catch_unwind_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with panic handling expertise | Task: Create PanicGuard with backtrace logging | Restrictions: Catch all panics, preserve backtraces | _Leverage: std::panic | Success: Panics caught and logged | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 4. Create CircuitBreaker
  - File: `core/src/safety/circuit_breaker.rs`
  - Implement state machine (Closed/Open/HalfOpen)
  - Add configurable thresholds and timeouts
  - Purpose: Prevent repeated failures
  - _Leverage: Atomic operations_
  - _Requirements: 3.4, 4.1_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing patterns | Task: Create CircuitBreaker with state machine | Restrictions: Thread-safe, configurable, lock-free | _Leverage: Atomics | Success: Circuit breaker prevents cascading failures | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Fallback Infrastructure

- [x] 5. Create FallbackEngine
  - File: `core/src/engine/fallback.rs`
  - Implement minimal passthrough engine
  - Add activation/deactivation with reason tracking
  - Purpose: Fallback when main engine fails
  - _Leverage: Null object pattern_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating fallback | Task: Create FallbackEngine for passthrough mode | Restrictions: Minimal, always works, no dependencies | _Leverage: Null object pattern | Success: Fallback passes keys through | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Create extension traits for safe unwrapping
  - File: `core/src/safety/extensions.rs`
  - OptionExt with unwrap_or_log
  - ResultExt with map_critical
  - Purpose: Migration helpers
  - _Leverage: Extension traits_
  - _Requirements: 2.1, 2.4_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating utilities | Task: Create OptionExt and ResultExt traits | Restrictions: Log on fallback, preserve context | _Leverage: Extension traits | Success: Easy migration from unwrap | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Critical Path Audit

- [x] 7. Audit and fix driver unwraps (Windows)
  - Files: `core/src/drivers/windows/*.rs`
  - Replace unwrap/expect with error handling
  - Wrap hook callback in PanicGuard
  - Purpose: Safe Windows driver
  - _Leverage: CriticalResult, PanicGuard_
  - _Requirements: 1.1, 1.3, 3.1_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Windows expertise | Task: Remove unwraps from Windows driver, add PanicGuard | Restrictions: Zero unwraps in hook callback, panic recovery | _Leverage: PanicGuard | Success: Windows driver panic-safe | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 8. Audit and fix driver unwraps (Linux)
  - Files: `core/src/drivers/linux/*.rs`
  - Replace unwrap/expect with error handling
  - Ensure existing catch_unwind is comprehensive
  - Purpose: Safe Linux driver
  - _Leverage: CriticalResult, PanicGuard_
  - _Requirements: 1.1, 1.3, 3.1_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Linux expertise | Task: Remove unwraps from Linux driver | Restrictions: Zero unwraps in reader loop, complete catch_unwind | _Leverage: PanicGuard | Success: Linux driver panic-safe | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 9. Audit and fix engine unwraps
  - Files: `core/src/engine/*.rs`
  - Replace unwrap/expect with CriticalResult
  - Add state recovery on errors
  - Purpose: Safe engine core
  - _Leverage: CriticalResult_
  - _Requirements: 1.1, 1.2, 2.1_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer auditing code | Task: Remove unwraps from engine core | Restrictions: Zero unwraps in process path, state recovery | _Leverage: CriticalResult | Success: Engine handles all errors | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 10. Audit and fix FFI boundary unwraps
  - Files: `core/src/ffi/exports_*.rs`
  - Replace unwrap/expect with error returns
  - Never panic across FFI boundary
  - Purpose: Safe FFI
  - _Leverage: CriticalResult_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer fixing FFI | Task: Remove unwraps from FFI boundary | Restrictions: Never panic across FFI, return errors | _Leverage: CriticalResult | Success: FFI never panics | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Discovery and Config

- [x] 11. Fix discovery module unwraps
  - Files: `core/src/discovery/*.rs`
  - Replace device parsing unwraps
  - Handle missing/invalid devices gracefully
  - Purpose: Robust device discovery
  - _Leverage: CriticalResult_
  - _Requirements: 1.1, 4.2_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer fixing discovery | Task: Remove unwraps from discovery module | Restrictions: Handle invalid devices, graceful fallback | _Leverage: CriticalResult | Success: Discovery never panics | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 12. Fix config loading unwraps
  - Files: `core/src/config/*.rs`
  - Use defaults on parse failure
  - Log errors but continue
  - Purpose: Robust config loading
  - _Leverage: Extension traits_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer fixing config | Task: Remove unwraps from config loading | Restrictions: Use defaults on failure, warn user | _Leverage: Extension traits | Success: Config always loads | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Integration

- [x] 13. Integrate CircuitBreaker into drivers
  - Files: `core/src/drivers/mod.rs`
  - Wrap driver operations in circuit breaker
  - Activate fallback when circuit opens
  - Purpose: Automatic failure recovery
  - _Leverage: CircuitBreaker, FallbackEngine_
  - _Requirements: 3.4, 4.1_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer integrating safety | Task: Add CircuitBreaker to driver operations | Restrictions: Auto-fallback on repeated failures | _Leverage: CircuitBreaker | Success: Drivers recover automatically | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 14. Add panic telemetry to FFI
  - File: `core/src/ffi/exports_telemetry.rs`
  - Export panic counts and recovery events
  - Enable Flutter to show recovery notifications
  - Purpose: User visibility into issues
  - _Leverage: FFI patterns_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding telemetry | Task: Export panic telemetry via FFI | Restrictions: Recovery events, panic counts, backtraces | _Leverage: FFI patterns | Success: Flutter shows recovery notifications | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 15. Verify emergency exit always works
  - Files: All driver files
  - Test emergency exit in panic scenarios
  - Test in circuit breaker open state
  - Purpose: User safety guarantee
  - _Leverage: Testing_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Developer testing safety | Task: Verify emergency exit works in all scenarios | Restrictions: Must work during panic, during fallback | _Leverage: Testing | Success: Emergency exit always works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Testing and Verification

- [x] 16. Add panic injection tests
  - File: `core/tests/panic_recovery_test.rs`
  - Inject panics at various points
  - Verify recovery and logging
  - Purpose: Verify panic handling
  - _Leverage: Test fixtures_
  - _Requirements: 3.1, 3.2_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create panic injection tests | Restrictions: Test all critical paths, verify recovery | _Leverage: Test fixtures | Success: All panics recovered | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Add lint to prevent new unwraps
  - File: `.cargo/config.toml` or clippy.toml
  - Deny unwrap/expect in critical modules
  - Allow only with explicit annotation
  - Purpose: Prevent regression
  - _Leverage: Clippy lints_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer configuring lints | Task: Add clippy lint to deny unwrap in critical paths | Restrictions: Deny in drivers/engine, allow with annotation | _Leverage: Clippy | Success: New unwraps cause build failure | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 18. Document panic handling architecture
  - File: `docs/panic-handling.md`
  - Explain PanicGuard usage
  - Document fallback behavior
  - Purpose: Developer documentation
  - _Leverage: Implementation knowledge_
  - _Requirements: Non-functional (documentation)_
  - _Prompt: Implement the task for spec unwrap-panic-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Document panic handling architecture | Restrictions: Cover all components, usage examples | _Leverage: Implementation | Success: Developers understand panic handling | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
