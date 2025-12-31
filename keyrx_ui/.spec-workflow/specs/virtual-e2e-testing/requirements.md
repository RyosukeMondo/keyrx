# Requirements Document

## Introduction

The Virtual E2E Testing spec implements a fully automated end-to-end test framework for keyrx that runs without real hardware. This enables CI/CD integration and autonomous development by using Linux's uinput subsystem to create virtual input devices that simulate physical keyboards, capturing output from the daemon's virtual keyboard for verification.

**Current State:** E2E tests exist but require real hardware (all marked `#[ignore]`). Mock implementations (MockInput/MockOutput) exist for unit tests but don't exercise the real kernel evdev/uinput path. CI runs unit and integration tests but cannot run E2E tests.

**This Spec Delivers:** A complete virtual device testing framework that:
1. Creates virtual keyboards using uinput for input injection
2. Captures output from the daemon's virtual keyboard via evdev
3. Orchestrates the full input→daemon→output pipeline
4. Enables E2E tests to run in CI without real hardware

## Alignment with Product Vision

This spec directly enables the **AI Coding Agent First** principle from product.md:

**From product.md - AI Coding Agent First:**
> "keyrx is designed to be verified, modified, and deployed by AI agents without human intervention"
> "Zero Manual Testing: All validation automated (no UAT phase)"

**How This Spec Delivers:**
- **Fully automated E2E tests**: No human intervention required
- **CI-compatible**: Tests run in GitHub Actions without real keyboards
- **Deterministic verification**: Same input → same output, verifiable by AI agents
- **No UAT phase**: All validation happens programmatically

**From product.md - Quality Metrics:**
> "Test Coverage: 80% minimum, 90% for critical paths"
> "Fuzz Testing: No crashes/panics under 1M+ random input sequences"

**How This Spec Enables:**
- **Higher coverage**: E2E tests verify full kernel path (evdev→daemon→uinput)
- **Regression prevention**: Automated tests catch issues before merge
- **Platform verification**: Tests real Linux subsystem behavior

**From tech.md - Known Limitation #3:**
> "WASM Simulation Cannot Test OS-Specific Quirks"
> "E2E tests on real OS (GitHub Actions matrix)"
> "Future Solution: Record/replay of OS events for deterministic testing"

**How This Spec Addresses:**
- **Real OS path**: Tests use actual evdev/uinput kernel interfaces
- **Virtual devices**: No hardware dependency, runs in VMs/containers
- **Deterministic replay**: Input sequences can be recorded and replayed

## Requirements

### Requirement 1: Virtual Input Device Creation

**User Story:** As a test framework, I want to create virtual keyboards using uinput, so that I can inject key events into the daemon without physical hardware.

#### Acceptance Criteria

1. **WHEN** test harness calls `VirtualKeyboard::create(name)` **THEN** system **SHALL** create uinput device
   - Device appears at `/dev/input/eventX`
   - Device name matches the specified name for pattern matching
   - Device has keyboard capability (EV_KEY with all key codes)
   - Returns error if `/dev/uinput` not accessible

2. **WHEN** calling `inject(KeyEvent::Press(keycode))` **THEN** system **SHALL** write evdev event
   - Write EV_KEY event with value=1 (press)
   - Write EV_SYN/SYN_REPORT after key event
   - Event is readable by daemon via evdev

3. **WHEN** calling `inject(KeyEvent::Release(keycode))` **THEN** system **SHALL** write evdev event
   - Write EV_KEY event with value=0 (release)
   - Write EV_SYN/SYN_REPORT after key event
   - Event is readable by daemon via evdev

4. **WHEN** VirtualKeyboard is dropped **THEN** system **SHALL** destroy device cleanly
   - Remove device from `/dev/input/`
   - Close file descriptors
   - No orphaned virtual devices remain

5. **WHEN** multiple VirtualKeyboards created **THEN** system **SHALL** support concurrent devices
   - Each device has unique path
   - Each device can inject events independently
   - Enables multi-device test scenarios

### Requirement 2: Output Event Capture

**User Story:** As a test framework, I want to capture key events from the daemon's virtual keyboard output, so that I can verify remapping behavior.

#### Acceptance Criteria

1. **WHEN** test harness calls `OutputCapture::find_by_name(name)` **THEN** system **SHALL** find device
   - Enumerate `/dev/input/event*` devices
   - Match device by name (e.g., "keyrx Virtual Keyboard")
   - Return error if device not found within timeout

2. **WHEN** daemon injects event via uinput **THEN** system **SHALL** capture via evdev
   - Read EV_KEY events from daemon's output device
   - Convert to KeyEvent (Press/Release)
   - Ignore non-key events (EV_SYN, etc.)

3. **WHEN** calling `next_event(timeout)` **THEN** system **SHALL** return event or timeout
   - Return `Some(KeyEvent)` if event available
   - Return `None` if timeout expires
   - Non-blocking by default with configurable timeout

4. **WHEN** calling `collect_events(timeout)` **THEN** system **SHALL** collect all events
   - Read events until timeout
   - Return Vec<KeyEvent> with all captured events
   - Preserve event ordering

5. **WHEN** calling `assert_events(expected)` **THEN** system **SHALL** verify sequence
   - Compare captured events against expected sequence
   - Panic with clear diff on mismatch
   - Support partial matching (ignore extra events)

### Requirement 3: E2E Test Harness Orchestration

**User Story:** As a test developer, I want a high-level harness that manages the full E2E pipeline, so that writing tests is simple and reliable.

#### Acceptance Criteria

1. **WHEN** test calls `E2EHarness::setup(config)` **THEN** system **SHALL** initialize pipeline
   - Create VirtualKeyboard with identifiable name
   - Create .krx config file matching virtual keyboard pattern
   - Start daemon process with config
   - Wait for daemon to grab virtual keyboard
   - Find and open daemon's output device
   - Return harness ready for testing

2. **WHEN** test calls `harness.inject(events)` **THEN** system **SHALL** inject sequence
   - Inject each KeyEvent via VirtualKeyboard
   - Support configurable delay between events (default: no delay)
   - Return after all events injected

3. **WHEN** test calls `harness.capture(timeout)` **THEN** system **SHALL** capture output
   - Collect events from OutputCapture
   - Return Vec<KeyEvent> of captured events
   - Handle timing differences between inject and output

4. **WHEN** test calls `harness.verify(expected)` **THEN** system **SHALL** verify output
   - Compare captured events against expected
   - Provide detailed assertion on failure
   - Support flexible matching modes

5. **WHEN** harness is dropped or `teardown()` called **THEN** system **SHALL** cleanup
   - Stop daemon process gracefully (SIGTERM)
   - Wait for daemon to release devices
   - Destroy virtual keyboard
   - Remove temporary config files
   - No orphaned processes or devices

6. **WHEN** daemon fails to start **THEN** system **SHALL** provide diagnostics
   - Capture daemon stderr output
   - Report clear error with logs
   - Cleanup partial resources

### Requirement 4: CI/CD Integration

**User Story:** As a CI/CD pipeline, I want to run virtual E2E tests automatically, so that every PR is validated without manual intervention.

#### Acceptance Criteria

1. **WHEN** CI runs virtual E2E tests **THEN** system **SHALL** execute without hardware
   - Tests use virtual devices only
   - No physical keyboard required
   - Works in GitHub Actions ubuntu-latest

2. **WHEN** CI environment lacks uinput **THEN** system **SHALL** setup permissions
   - Load uinput kernel module: `modprobe uinput`
   - Set permissions: `chmod 666 /dev/uinput`
   - Document required CI setup steps

3. **WHEN** tests pass **THEN** CI **SHALL** report success
   - Exit code 0 on all tests passing
   - Clear output showing test results
   - No false positives

4. **WHEN** tests fail **THEN** CI **SHALL** report failure with diagnostics
   - Exit code non-zero on failure
   - Show which test failed and why
   - Include captured vs expected events
   - Daemon logs attached for debugging

5. **WHEN** test times out **THEN** system **SHALL** cleanup and fail gracefully
   - Kill daemon process if hung
   - Destroy virtual devices
   - Report timeout with context
   - No zombie processes left

### Requirement 5: Test Coverage for Remapping Scenarios

**User Story:** As a developer, I want comprehensive E2E test cases covering all remapping types, so that I can verify the daemon works correctly end-to-end.

#### Acceptance Criteria

1. **WHEN** testing simple remap (A→B) **THEN** test **SHALL** verify
   - Press A → outputs Press B
   - Release A → outputs Release B
   - Original key (A) not passed through

2. **WHEN** testing modifier activation (CapsLock→MD_00) **THEN** test **SHALL** verify
   - Press CapsLock → no output (modifier state set internally)
   - Release CapsLock → no output (modifier state cleared)
   - State change verifiable via conditional test

3. **WHEN** testing lock toggle (ScrollLock→LK_00) **THEN** test **SHALL** verify
   - Press ScrollLock → lock state toggled
   - Release ScrollLock → no output
   - Second press toggles lock off

4. **WHEN** testing conditional mapping **THEN** test **SHALL** verify
   - With modifier active: conditional mapping applies
   - Without modifier: fallback or passthrough
   - Complex conditions (AllActive, NotActive)

5. **WHEN** testing modified output (Shift+Key) **THEN** test **SHALL** verify
   - Press trigger → outputs Press(Shift), Press(Key)
   - Release trigger → outputs Release(Key), Release(Shift)
   - Correct ordering maintained

6. **WHEN** testing passthrough (no mapping) **THEN** test **SHALL** verify
   - Unmapped key passes through unchanged
   - Press→Press, Release→Release
   - No state side effects

7. **WHEN** testing multi-event sequences **THEN** test **SHALL** verify
   - Complex typing patterns work correctly
   - State accumulates properly across events
   - No event loss or reordering

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: VirtualKeyboard handles input injection only, OutputCapture handles output reading only, E2EHarness orchestrates both
- **Modular Design**: Test utilities are reusable across test files
- **Dependency Management**: Minimal dependencies, use existing evdev/uinput crates
- **Clear Interfaces**: Public API is simple and discoverable

### Performance

- **Test Execution Time**: Each E2E test completes in <5 seconds
- **Event Latency**: Inject→capture round-trip <100ms
- **Parallel Test Support**: Tests can run in parallel with isolated virtual devices
- **Resource Cleanup**: No resource leaks across test runs

### Security

- **No Privilege Escalation**: Tests run with minimal permissions (input/uinput groups)
- **Sandboxed Devices**: Virtual devices only affect test environment
- **No Key Logging**: Test output contains keycodes only, no PII

### Reliability

- **Deterministic Results**: Same test input → same output (no flaky tests)
- **Graceful Cleanup**: Resources released even on panic/timeout
- **Error Recovery**: Failed tests don't block subsequent tests
- **Timeout Handling**: Hung daemon detected and killed

### Usability

- **Simple API**: `harness.inject()` and `harness.verify()` pattern
- **Clear Errors**: Assertion failures show expected vs actual
- **Easy Debugging**: Debug logs available, daemon output captured
- **Documentation**: All public APIs documented with examples
