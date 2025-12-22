# Tasks Document

## Phase 1: Core Virtual Device Infrastructure

- [x] 1. Create test_utils module structure
  - File: `keyrx_daemon/src/test_utils/mod.rs` (NEW)
  - Create module directory and mod.rs
  - Define VirtualDeviceError enum with thiserror
  - Export submodules (virtual_keyboard, output_capture)
  - Purpose: Establish test utility module structure
  - _Leverage: keyrx_daemon/src/platform/mod.rs pattern for module organization_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer specializing in module organization and error types
    Task: Create the test_utils module structure with VirtualDeviceError enum following requirements 1.1 and 2.1, using the platform module as a reference pattern
    Restrictions: Do not implement VirtualKeyboard or OutputCapture yet, only the module structure and error type. Keep error messages actionable with fix instructions.
    Success: Module compiles, VirtualDeviceError has all variants (PermissionDenied, NotFound, Timeout, Io, CreationFailed), proper exports configured
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 2. Implement VirtualKeyboard struct
  - File: `keyrx_daemon/src/test_utils/virtual_keyboard.rs` (NEW)
  - Create VirtualKeyboard struct with uinput device
  - Implement create(name) method with full keyboard capabilities
  - Use unique device names (include timestamp/random suffix)
  - Purpose: Enable test input injection via uinput
  - _Leverage: keyrx_daemon/src/platform/linux/mod.rs UinputOutput::create pattern_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with Linux uinput expertise
    Task: Implement VirtualKeyboard::create() following requirements 1.1-1.5, using UinputOutput as reference for uinput device creation
    Restrictions: Focus only on device creation and name handling. Do not implement inject() yet. Use existing keycode_map module for key registration.
    Success: VirtualKeyboard::create("test") returns device, device appears in /dev/input/, device has identifiable name, device is destroyed on drop
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 3. Implement VirtualKeyboard event injection
  - File: `keyrx_daemon/src/test_utils/virtual_keyboard.rs` (continue)
  - Implement inject(KeyEvent) method
  - Write EV_KEY + EV_SYN events to uinput
  - Implement inject_sequence with optional delay
  - Purpose: Enable injecting test key events
  - _Leverage: keyrx_daemon/src/platform/linux/mod.rs UinputOutput::inject_event pattern, keycode_map::keycode_to_evdev_
  - _Requirements: 1.2, 1.3_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with Linux input subsystem expertise
    Task: Implement VirtualKeyboard::inject() and inject_sequence() following requirements 1.2 and 1.3, using UinputOutput::inject_event as reference
    Restrictions: Must write SYN_REPORT after each key event. Use keycode_to_evdev for key conversion. Handle both Press and Release correctly (value=1 and value=0).
    Success: inject(Press(A)) writes correct evdev event, inject(Release(A)) writes correct event, inject_sequence works with delays
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 4. Write VirtualKeyboard unit tests
  - File: `keyrx_daemon/src/test_utils/virtual_keyboard.rs` (add tests module)
  - Test device creation and naming
  - Test device cleanup on drop
  - Test event injection produces readable events
  - Purpose: Verify VirtualKeyboard works correctly
  - _Leverage: keyrx_daemon/src/platform/linux/mod.rs test patterns_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust QA Engineer with testing expertise
    Task: Write comprehensive unit tests for VirtualKeyboard covering requirements 1.1-1.4
    Restrictions: Tests that require /dev/uinput should be marked #[ignore] with clear instructions. Include test for permission denied scenario.
    Success: Tests cover creation, injection, drop cleanup, all tests pass or are appropriately ignored with documentation
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

## Phase 2: Output Capture Implementation

- [x] 5. Implement OutputCapture struct
  - File: `keyrx_daemon/src/test_utils/output_capture.rs` (NEW)
  - Create OutputCapture struct wrapping evdev::Device
  - Implement find_by_name(name, timeout) with polling
  - Enumerate /dev/input/event* and match by name
  - Purpose: Enable finding and opening daemon's output device
  - _Leverage: keyrx_daemon/src/device_manager/linux.rs enumerate_keyboards pattern_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with Linux evdev expertise
    Task: Implement OutputCapture::find_by_name() following requirement 2.1, using device_manager enumerate_keyboards as reference
    Restrictions: Must poll for device existence (device may not exist immediately). Return Timeout error if not found within duration. Do not grab the device.
    Success: find_by_name("keyrx Virtual Keyboard", 5s) finds existing device, returns Timeout for non-existent device, handles race conditions
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 6. Implement OutputCapture event reading
  - File: `keyrx_daemon/src/test_utils/output_capture.rs` (continue)
  - Implement next_event(timeout) with non-blocking read
  - Implement collect_events(timeout) to gather multiple events
  - Implement drain() to clear pending events
  - Filter to EV_KEY events only (ignore EV_SYN, etc.)
  - Purpose: Enable reading output events from daemon
  - _Leverage: keyrx_daemon/src/platform/linux/mod.rs EvdevInput::next_event pattern, keycode_map::evdev_to_keycode_
  - _Requirements: 2.2, 2.3, 2.4_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with async I/O expertise
    Task: Implement OutputCapture event reading methods following requirements 2.2-2.4, using EvdevInput as reference
    Restrictions: Use non-blocking reads with poll/select. Convert evdev events to KeyEvent using evdev_to_keycode. Only capture EV_KEY events.
    Success: next_event returns events with timeout, collect_events gathers all pending events, drain clears buffer
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 7. Implement OutputCapture assertion helpers
  - File: `keyrx_daemon/src/test_utils/output_capture.rs` (continue)
  - Implement assert_events(captured, expected) with detailed diff
  - Show expected vs actual with markers for mismatches
  - Handle extra/missing events clearly
  - Purpose: Enable clear test assertions
  - _Leverage: Standard Rust testing patterns_
  - _Requirements: 2.5_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust QA Engineer specializing in test assertions
    Task: Implement assert_events helper following requirement 2.5
    Restrictions: Do not panic with opaque errors. Show side-by-side comparison. Mark mismatches, extras, and missing events clearly.
    Success: Assertion failure shows detailed diff, passing assertion returns cleanly, handles empty sequences
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 8. Write OutputCapture unit tests
  - File: `keyrx_daemon/src/test_utils/output_capture.rs` (add tests module)
  - Test device discovery by name
  - Test timeout behavior
  - Test event reading and filtering
  - Test assertion helper formatting
  - Purpose: Verify OutputCapture works correctly
  - _Leverage: VirtualKeyboard for creating test devices_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust QA Engineer with testing expertise
    Task: Write comprehensive unit tests for OutputCapture covering requirements 2.1-2.5
    Restrictions: Use VirtualKeyboard to create devices for testing OutputCapture. Tests requiring /dev/uinput should be #[ignore].
    Success: Tests cover discovery, reading, assertions, all tests pass or are appropriately ignored
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

## Phase 3: E2E Test Harness

- [x] 9. Create E2EError type and helpers
  - File: `keyrx_daemon/tests/e2e_harness.rs` (NEW)
  - Define E2EError enum with all error variants
  - Create E2EConfig struct for test configuration
  - Implement helper constructors (simple_remap, modifier, conditional)
  - Purpose: Establish harness error handling and config helpers
  - _Leverage: keyrx_core/src/config/mappings.rs KeyMapping constructors_
  - _Requirements: 3.1, 3.4, 3.6_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer specializing in test infrastructure
    Task: Create E2EError and E2EConfig types following requirements 3.1, 3.4, 3.6
    Restrictions: E2EError must wrap VirtualDeviceError. E2EConfig helpers should use existing KeyMapping constructors. Include VerificationFailed variant with captured/expected.
    Success: E2EError covers all failure modes, E2EConfig::simple_remap/modifier/conditional work correctly
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 10. Implement E2EHarness::setup
  - File: `keyrx_daemon/tests/e2e_harness.rs` (continue)
  - Implement setup(config) method
  - Create VirtualKeyboard with unique name
  - Generate .krx config file matching virtual keyboard
  - Start daemon as subprocess with config
  - Wait for daemon to grab device and create output
  - Find and open output device
  - Purpose: Initialize complete E2E test environment
  - _Leverage: keyrx_compiler serialize for .krx generation, tempfile for temp config_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with subprocess and IPC expertise
    Task: Implement E2EHarness::setup() following requirement 3.1
    Restrictions: Use unique device names to support parallel tests. Daemon must be started as subprocess (not in-process). Wait for daemon to be ready before returning. Capture daemon stderr for diagnostics.
    Success: setup() returns harness with working virtual keyboard, running daemon, and connected output capture
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 11. Implement E2EHarness inject and capture methods
  - File: `keyrx_daemon/tests/e2e_harness.rs` (continue)
  - Implement inject(events) delegating to VirtualKeyboard
  - Implement capture(timeout) delegating to OutputCapture
  - Implement inject_and_capture convenience method
  - Implement verify(captured, expected) using assert_events
  - Purpose: Provide test interaction methods
  - _Leverage: VirtualKeyboard::inject_sequence, OutputCapture::collect_events_
  - _Requirements: 3.2, 3.3, 3.4_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer specializing in test infrastructure
    Task: Implement E2EHarness interaction methods following requirements 3.2-3.4
    Restrictions: inject_and_capture should drain output before injecting to avoid stale events. verify should use OutputCapture::assert_events.
    Success: inject/capture/verify methods work correctly, inject_and_capture is atomic
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 12. Implement E2EHarness teardown and Drop
  - File: `keyrx_daemon/tests/e2e_harness.rs` (continue)
  - Implement teardown(self) for graceful cleanup
  - Send SIGTERM to daemon process
  - Wait for daemon to exit with timeout
  - Destroy virtual keyboard
  - Remove temporary config file
  - Implement Drop trait for panic-safe cleanup
  - Purpose: Ensure clean resource cleanup
  - _Leverage: nix crate for signal sending_
  - _Requirements: 3.5_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with systems programming expertise
    Task: Implement E2EHarness::teardown() and Drop following requirement 3.5
    Restrictions: Must handle case where daemon already exited. SIGKILL as fallback after SIGTERM timeout. Drop must not panic. Clean up all resources even on partial failure.
    Success: teardown cleans up all resources, Drop works on panic, no orphaned processes or devices
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 13. Add TestEvents helper
  - File: `keyrx_daemon/tests/e2e_harness.rs` (continue)
  - Create TestEvents struct with helper methods
  - Implement press(key), release(key), tap(key)
  - Implement type_keys(keys) for sequences
  - Purpose: Simplify test event creation
  - _Leverage: keyrx_core::runtime::event::KeyEvent_
  - _Requirements: 5.1-5.7_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer specializing in ergonomic APIs
    Task: Create TestEvents helper for concise test event creation
    Restrictions: Methods should be associated functions (no self). Return types should be compatible with harness.inject().
    Success: TestEvents::tap(KeyCode::A) returns [Press(A), Release(A)], type_keys works for sequences
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

## Phase 4: E2E Test Cases

- [x] 14. Create virtual_e2e_tests.rs with simple remap tests
  - File: `keyrx_daemon/tests/virtual_e2e_tests.rs` (NEW)
  - Test simple A→B remap (press and release)
  - Test multiple remaps in sequence
  - Test unmapped key passthrough
  - Purpose: Verify basic remapping works end-to-end
  - _Leverage: E2EHarness, TestEvents_
  - _Requirements: 5.1, 5.6_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust QA Engineer specializing in E2E testing
    Task: Write E2E tests for simple remapping covering requirements 5.1 and 5.6
    Restrictions: Tests should be self-contained (setup/teardown per test). Mark tests #[ignore] with clear instructions for running. Use descriptive test names.
    Success: Tests verify press→press, release→release, passthrough for unmapped keys
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 15. Add modifier and lock state tests
  - File: `keyrx_daemon/tests/virtual_e2e_tests.rs` (continue)
  - Test modifier activation (no output, state change)
  - Test lock toggle (toggle on press, ignore release)
  - Test conditional mapping with modifier active
  - Test conditional mapping without modifier (passthrough)
  - Purpose: Verify modifier/lock state management
  - _Leverage: E2EHarness, E2EConfig::conditional_
  - _Requirements: 5.2, 5.3, 5.4_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust QA Engineer specializing in state-based testing
    Task: Write E2E tests for modifier and lock behavior covering requirements 5.2-5.4
    Restrictions: Test both active and inactive modifier states. Verify lock toggles on second press. Test conditional mappings thoroughly.
    Success: Tests verify modifier sets internal state, lock toggles, conditionals work correctly
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 16. Add modified output tests
  - File: `keyrx_daemon/tests/virtual_e2e_tests.rs` (continue)
  - Test Shift+Key output sequence (modifier ordering)
  - Test Ctrl+Key combination
  - Test multiple modifiers (Ctrl+Shift+Key)
  - Verify correct press/release ordering
  - Purpose: Verify modified output sequences
  - _Leverage: E2EHarness, KeyMapping::modified_output_
  - _Requirements: 5.5_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust QA Engineer specializing in E2E testing
    Task: Write E2E tests for modified output behavior covering requirement 5.5
    Restrictions: Verify exact ordering: Press(modifiers) → Press(key) → Release(key) → Release(modifiers). Test with multiple modifier combinations.
    Success: Tests verify Shift+Key, Ctrl+Key, multi-modifier sequences with correct ordering
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [x] 17. Add multi-event sequence tests
  - File: `keyrx_daemon/tests/virtual_e2e_tests.rs` (continue)
  - Test typing pattern (multiple taps in sequence)
  - Test modifier hold during typing
  - Test state accumulation across events
  - Test complex vim-style navigation layer
  - Purpose: Verify complex event sequences
  - _Leverage: E2EHarness, TestEvents::type_keys_
  - _Requirements: 5.7_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust QA Engineer specializing in integration testing
    Task: Write E2E tests for complex event sequences covering requirement 5.7
    Restrictions: Test realistic typing patterns. Verify no event loss or reordering. Include vim-navigation-layer test.
    Success: Tests verify complex sequences work correctly, state persists across events
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

## Phase 5: CI/CD Integration

- [ ] 18. Update Cargo.toml with test dependencies
  - File: `keyrx_daemon/Cargo.toml` (MODIFY)
  - Add dev-dependencies: tempfile, nix (for signals)
  - Ensure evdev and uinput are available in test builds
  - Configure test binary for virtual_e2e_tests
  - Purpose: Enable compilation of E2E test infrastructure
  - _Leverage: Existing Cargo.toml structure_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with Cargo expertise
    Task: Update Cargo.toml with required test dependencies for virtual E2E tests
    Restrictions: Add to [dev-dependencies] only. Do not modify production dependencies. Keep versions consistent with existing deps.
    Success: cargo test --test virtual_e2e_tests compiles successfully
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [ ] 19. Create CI workflow for virtual E2E tests
  - File: `.github/workflows/ci.yml` (MODIFY)
  - Add virtual-e2e job that runs on ubuntu-latest
  - Setup uinput permissions (modprobe, chmod)
  - Run virtual E2E tests with --ignored flag
  - Capture test output and daemon logs on failure
  - Purpose: Enable automated E2E testing in CI
  - _Leverage: Existing ci.yml structure_
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: DevOps Engineer with GitHub Actions expertise
    Task: Add virtual E2E test job to CI workflow following requirements 4.1-4.4
    Restrictions: Use sudo for uinput setup. Run tests with cargo test --test virtual_e2e_tests -- --ignored. Upload logs as artifacts on failure.
    Success: CI job runs virtual E2E tests without hardware, reports clear pass/fail
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [ ] 20. Add test timeout handling
  - File: `keyrx_daemon/tests/e2e_harness.rs` (MODIFY)
  - Add test-level timeout wrapper
  - Force cleanup if test exceeds timeout
  - Kill daemon process if hung
  - Report timeout with diagnostic context
  - Purpose: Prevent hung tests from blocking CI
  - _Leverage: std::thread::spawn for timeout wrapper_
  - _Requirements: 4.5_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with concurrent programming expertise
    Task: Add test timeout handling following requirement 4.5
    Restrictions: Default timeout should be generous (30 seconds). Cleanup must run even on timeout. Report which phase timed out.
    Success: Hung tests are killed after timeout, resources cleaned up, clear timeout error reported
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

## Phase 6: Documentation and Integration

- [ ] 21. Add module documentation
  - File: `keyrx_daemon/src/test_utils/mod.rs` (MODIFY)
  - Add comprehensive module-level documentation
  - Document each public type and function
  - Include usage examples in doc comments
  - Purpose: Enable discoverability and correct usage
  - _Leverage: Existing rustdoc patterns in codebase_
  - _Requirements: Non-functional (Usability)_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Technical Writer with Rust documentation expertise
    Task: Add comprehensive rustdoc documentation to test_utils module
    Restrictions: Include code examples that compile (doc tests). Document error conditions and panics. Follow existing codebase documentation style.
    Success: cargo doc generates clean documentation, examples compile, all public items documented
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [ ] 22. Update existing e2e_tests.rs with references
  - File: `keyrx_daemon/tests/e2e_tests.rs` (MODIFY)
  - Add comment pointing to virtual_e2e_tests.rs for CI-compatible tests
  - Document which tests require real hardware vs virtual devices
  - Keep existing hardware tests as manual verification option
  - Purpose: Clarify relationship between test files
  - _Leverage: Existing test documentation patterns_
  - _Requirements: Non-functional (Usability)_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: Rust Developer with testing expertise
    Task: Update existing e2e_tests.rs with documentation linking to virtual tests
    Restrictions: Do not modify test logic. Add header comments explaining the difference. Keep hardware tests intact.
    Success: e2e_tests.rs clearly documents when to use hardware vs virtual tests
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_

- [ ] 23. Run full test suite and verify CI
  - File: N/A (verification task)
  - Run all unit tests: `cargo test --lib`
  - Run virtual E2E tests locally: `cargo test --test virtual_e2e_tests -- --ignored`
  - Verify CI workflow passes on PR
  - Fix any flaky tests or timing issues
  - Purpose: Final verification of E2E testing infrastructure
  - _Leverage: All implemented components_
  - _Requirements: All_
  - _Prompt: Implement the task for spec virtual-e2e-testing, first run spec-workflow-guide to get the workflow guide then implement the task:
    Role: QA Engineer with CI/CD expertise
    Task: Run full test suite and verify CI integration
    Restrictions: All tests must pass. No flaky tests allowed. CI must report clear results. Fix any issues found.
    Success: All tests pass locally and in CI, no flaky tests, clean CI output
    After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record what was built, then mark [x] as complete_
