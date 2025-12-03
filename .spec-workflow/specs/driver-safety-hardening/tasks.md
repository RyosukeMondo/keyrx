# Tasks Document

## Phase 1: Infrastructure

- [x] 1. Create DriverError type
  - File: `core/src/drivers/common/error.rs`
  - Define error variants with hints
  - Implement is_retryable() and suggested_action()
  - Purpose: Unified driver error handling
  - _Leverage: thiserror, existing error patterns_
  - _Requirements: 5.2, 5.3_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating error types | Task: Create DriverError in core/src/drivers/common/error.rs | Restrictions: Include recovery hints, is_retryable flag, suggested actions | _Leverage: thiserror patterns | Success: Error variants cover all driver scenarios, hints are helpful | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 2. Create safety module structure
  - Files: `core/src/drivers/{windows,linux}/safety/mod.rs`
  - Set up module hierarchy
  - Add documentation explaining safety approach
  - Purpose: Foundation for safety wrappers
  - _Leverage: Rust module patterns_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer setting up modules | Task: Create safety module structure in drivers | Restrictions: Clear documentation, proper visibility | _Leverage: Rust module patterns | Success: Module structure ready for safety wrappers | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Windows Safety Wrappers

- [ ] 3. Create HookCallback with panic catching
  - File: `core/src/drivers/windows/safety/callback.rs`
  - Implement panic-safe callback wrapper
  - Log panics for debugging
  - Return PassThrough on panic
  - Purpose: Prevent panics from breaking hooks
  - _Leverage: std::panic::catch_unwind_
  - _Requirements: 2.2_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with panic handling expertise | Task: Create HookCallback with panic catching in safety/callback.rs | Restrictions: Catch all panics, log details, safe fallback | _Leverage: std::panic::catch_unwind | Success: Panics in callbacks don't break hooks | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 4. Create ThreadLocalState wrapper
  - File: `core/src/drivers/windows/safety/thread_local.rs`
  - Encapsulate thread-local storage
  - Safe initialization and access
  - Purpose: Type-safe thread-local access
  - _Leverage: std::cell, thread_local!_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with thread-local expertise | Task: Create ThreadLocalState in safety/thread_local.rs | Restrictions: Safe access, no panics, proper initialization | _Leverage: thread_local! macro | Success: Thread-local access is safe and documented | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 5. Create SafeHook wrapper
  - File: `core/src/drivers/windows/safety/hook.rs`
  - Wrap SetWindowsHookEx with safe API
  - Implement Drop for cleanup
  - Add SAFETY comments to all unsafe blocks
  - Purpose: Safe hook lifecycle management
  - _Leverage: windows-rs, existing hook.rs_
  - _Requirements: 1.2, 1.3, 2.1, 2.3, 2.4_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Windows API expertise | Task: Create SafeHook in safety/hook.rs with SAFETY comments | Restrictions: RAII for cleanup, document invariants, handle all errors | _Leverage: windows-rs, existing hook.rs | Success: Hook lifecycle is safe, cleanup guaranteed | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 6. Update WindowsInputSource to use safety wrappers
  - File: `core/src/drivers/windows/mod.rs`
  - Replace direct unsafe with SafeHook
  - Use ThreadLocalState for event routing
  - Reduce unsafe blocks in main code
  - Purpose: Integrate safety wrappers
  - _Leverage: SafeHook, ThreadLocalState_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer integrating safety | Task: Update WindowsInputSource to use safety wrappers | Restrictions: Remove direct unsafe, use wrappers, maintain functionality | _Leverage: SafeHook, ThreadLocalState | Success: Windows driver uses safe wrappers, less unsafe code | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Linux Safety Wrappers

- [ ] 7. Create SafeDevice wrapper
  - File: `core/src/drivers/linux/safety/device.rs`
  - Wrap evdev device operations
  - Handle disconnection gracefully
  - RAII for grab/ungrab
  - Purpose: Safe device lifecycle
  - _Leverage: evdev crate_
  - _Requirements: 1.2, 4.1, 4.3_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Linux input expertise | Task: Create SafeDevice in safety/device.rs | Restrictions: RAII for grab, handle disconnection, document SAFETY | _Leverage: evdev crate | Success: Device operations are safe, disconnection handled | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 8. Create SafeUinput wrapper
  - File: `core/src/drivers/linux/safety/uinput.rs`
  - Wrap uinput device creation
  - Validate events before injection
  - RAII for cleanup
  - Purpose: Safe virtual device
  - _Leverage: uinput crate_
  - _Requirements: 1.2, 4.4_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with uinput expertise | Task: Create SafeUinput in safety/uinput.rs | Restrictions: RAII for cleanup, validate events, document SAFETY | _Leverage: uinput crate | Success: Virtual device operations are safe | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Add permission error handling
  - File: `core/src/drivers/linux/safety/permissions.rs`
  - Check and report permission issues
  - Provide helpful hints (input group, udev rules)
  - Purpose: User-friendly permission errors
  - _Leverage: DriverError_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Linux permissions knowledge | Task: Create permission checking in safety/permissions.rs | Restrictions: Check /dev/input access, suggest input group, udev rules | _Leverage: DriverError | Success: Permission errors have actionable suggestions | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Update LinuxInputSource to use safety wrappers
  - File: `core/src/drivers/linux/mod.rs`
  - Replace direct device operations with SafeDevice
  - Use SafeUinput for injection
  - Reduce unsafe blocks
  - Purpose: Integrate safety wrappers
  - _Leverage: SafeDevice, SafeUinput_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer integrating safety | Task: Update LinuxInputSource to use safety wrappers | Restrictions: Remove direct unsafe, use wrappers, maintain functionality | _Leverage: SafeDevice, SafeUinput | Success: Linux driver uses safe wrappers, less unsafe code | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Error Recovery

- [ ] 11. Implement retry with backoff
  - File: `core/src/drivers/common/recovery.rs`
  - Add exponential backoff for retryable errors
  - Configure max retries and delays
  - Purpose: Automatic error recovery
  - _Leverage: DriverError::is_retryable_
  - _Requirements: 5.1_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing recovery | Task: Create retry logic in common/recovery.rs | Restrictions: Exponential backoff, configurable, log attempts | _Leverage: DriverError::is_retryable | Success: Temporary errors are retried, recovery works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 12. Ensure emergency exit always works
  - Files: Both Windows and Linux drivers
  - Verify Ctrl+Alt+Shift+Esc is never blocked
  - Test in error scenarios
  - Purpose: User safety
  - _Leverage: Emergency exit logic_
  - _Requirements: 5.4_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer ensuring safety | Task: Verify emergency exit works in all error scenarios | Restrictions: Never block emergency combo, test with stuck states | _Leverage: Existing emergency exit logic | Success: Emergency exit works even during errors | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Documentation & Testing

- [ ] 13. Audit and document all unsafe blocks
  - Files: All driver files
  - Add SAFETY comments to every unsafe block
  - Explain invariants and guarantees
  - Purpose: Safety documentation
  - _Leverage: Rust unsafe documentation guidelines_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer documenting safety | Task: Audit all unsafe blocks and add SAFETY comments | Restrictions: Every unsafe has SAFETY comment, explain invariants | _Leverage: Rust unsafe documentation guidelines | Success: All unsafe blocks documented, clear invariants | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Add driver integration tests
  - File: `core/tests/integration/drivers/`
  - Test error handling paths
  - Test recovery scenarios
  - Mock device operations where possible
  - Purpose: Driver reliability
  - _Leverage: Test fixtures_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create driver integration tests | Restrictions: Test error paths, mock where needed, platform-specific | _Leverage: Test fixtures | Success: Error handling tested, recovery verified | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Create driver debugging guide
  - File: `docs/driver-debugging.md`
  - Document environment flags for debugging
  - Explain common issues and solutions
  - Platform-specific troubleshooting
  - Purpose: Developer/user support
  - _Leverage: Implementation knowledge_
  - _Requirements: Non-functional (usability)_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create driver debugging documentation | Restrictions: Cover both platforms, common issues, env flags | _Leverage: Implementation knowledge | Success: Documentation helps debug driver issues | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Verify unsafe block count reduction
  - Files: All driver files
  - Count unsafe blocks before/after
  - Report reduction percentage
  - Purpose: Measure success
  - _Leverage: Code metrics_
  - _Requirements: Non-functional (quality)_
  - _Prompt: Implement the task for spec driver-safety-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Developer | Task: Count and verify unsafe block reduction | Restrictions: Measure before/after, report improvement | _Leverage: grep for unsafe blocks | Success: Documented reduction in unsafe code, metrics captured | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
