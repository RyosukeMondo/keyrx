# Requirements Document: platform-drivers

## Introduction

This specification implements Phase 2 ("The Nervous System") from the KeyRx product roadmap - real platform-specific keyboard drivers for Linux and Windows. This transforms KeyRx from a simulation-only tool into a fully functional keyboard remapper that intercepts and transforms real keyboard input.

## Alignment with Product Vision

From product.md:
- **Phase 2: The Nervous System (Drivers)**: Platform-specific drivers for Windows (WH_KEYBOARD_LL) and Linux (uinput/evdev)
- **Performance > Features**: Sub-1ms latency requirement for "invisible" feel
- **Cross-Platform**: Consistent behavior on Windows and Linux

From tech.md:
- **Trait Abstraction**: OS adapters implement generic `InputSource` trait
- **Modular Drivers**: Drivers are plugins, enabling mock testing
- **No Elevated Privileges**: Runs in user space where possible

## Requirements

### REQ-1: Linux Keyboard Capture

**User Story:** As a Linux user, I want KeyRx to intercept my keyboard input, so that I can remap keys system-wide.

#### Acceptance Criteria

1. WHEN KeyRx starts with a script THEN it SHALL grab the keyboard device via evdev
2. WHEN a key is pressed THEN the engine SHALL receive the event within 500μs
3. WHEN the keyboard is grabbed THEN other applications SHALL NOT receive raw input
4. WHEN KeyRx stops THEN it SHALL release the keyboard grab cleanly
5. IF multiple keyboards are connected THEN KeyRx SHALL support selecting which to grab
6. IF /dev/input/* is not accessible THEN clear error with remediation SHALL be shown
7. WHEN running THEN original keyboard device SHALL be exclusively grabbed (EVIOCGRAB)

### REQ-2: Linux Key Injection

**User Story:** As a Linux user, I want KeyRx to inject remapped keys, so that applications receive my configured output.

#### Acceptance Criteria

1. WHEN a key is remapped THEN the remapped key SHALL be injected via uinput
2. WHEN a key is blocked THEN NO output SHALL be sent to applications
3. WHEN a key passes through THEN it SHALL be re-injected with original timing
4. WHEN KeyRx creates uinput device THEN it SHALL register all supported key codes
5. WHEN injecting THEN both KEY_DOWN and KEY_UP events SHALL be sent correctly
6. WHEN injecting modifier combinations THEN proper ordering SHALL be maintained
7. IF uinput write fails THEN error SHALL be logged and operation retried once

### REQ-3: Windows Keyboard Hook

**User Story:** As a Windows user, I want KeyRx to intercept my keyboard input, so that I can remap keys system-wide.

#### Acceptance Criteria

1. WHEN KeyRx starts THEN it SHALL install WH_KEYBOARD_LL hook via SetWindowsHookExW
2. WHEN a key is pressed THEN the hook callback SHALL receive the event
3. WHEN the hook is installed THEN it SHALL intercept all keyboard input system-wide
4. WHEN KeyRx stops THEN it SHALL uninstall hook via UnhookWindowsHookEx
5. WHEN hook callback processes THEN it SHALL return within 100ms (Windows requirement)
6. IF hook installation fails THEN clear error with remediation SHALL be shown
7. WHEN running THEN a message pump thread SHALL process hook callbacks

### REQ-4: Windows Key Injection

**User Story:** As a Windows user, I want KeyRx to inject remapped keys, so that applications receive my configured output.

#### Acceptance Criteria

1. WHEN a key is remapped THEN the remapped key SHALL be injected via SendInput
2. WHEN a key is blocked THEN hook callback SHALL return non-zero (consume event)
3. WHEN a key passes through THEN hook callback SHALL return zero (pass event)
4. WHEN injecting THEN KEYBDINPUT structure SHALL be properly configured
5. WHEN injecting THEN KEYEVENTF_EXTENDEDKEY SHALL be set for extended keys
6. WHEN injecting THEN both KEYEVENTF_KEYDOWN and KEYEVENTF_KEYUP SHALL be used correctly
7. IF SendInput fails THEN error SHALL be logged with GetLastError details

### REQ-5: Event Loop Integration

**User Story:** As a developer, I want drivers to integrate seamlessly with the async engine, so that the architecture remains clean.

#### Acceptance Criteria

1. WHEN driver implements InputSource THEN it SHALL work with existing Engine<I,S,St>
2. WHEN poll_events is called THEN it SHALL return events without blocking indefinitely
3. WHEN send_output is called THEN it SHALL inject keys asynchronously
4. WHEN driver is running THEN it SHALL use channels for cross-thread communication
5. WHEN engine stops THEN driver threads SHALL terminate within 100ms
6. IF driver thread panics THEN engine SHALL detect and report error

### REQ-6: Latency Requirements

**User Story:** As a user, I want key remapping to feel instantaneous, so that my typing is not affected.

#### Acceptance Criteria

1. WHEN processing a key event THEN total latency SHALL be < 1ms (capture + process + inject)
2. WHEN benchmarking THEN p99 latency SHALL be < 2ms
3. WHEN running THEN latency stats SHALL be available via `keyrx state --latency`
4. WHEN latency exceeds threshold THEN warning SHALL be logged
5. IF script execution is slow THEN it SHALL be measured separately from driver latency
6. WHEN profiling THEN driver overhead vs script overhead SHALL be distinguishable

### REQ-7: Graceful Degradation

**User Story:** As a user, I want KeyRx to handle errors gracefully, so that my keyboard always works.

#### Acceptance Criteria

1. IF driver initialization fails THEN keyboard SHALL continue working normally
2. IF driver crashes during operation THEN keyboard grab SHALL be released
3. WHEN Ctrl+C is pressed THEN graceful shutdown SHALL occur within 500ms
4. WHEN SIGTERM is received (Linux) THEN graceful shutdown SHALL occur
5. IF panic occurs in driver thread THEN keyboard SHALL be restored to normal
6. WHEN recovering from error THEN no key events SHALL be lost or duplicated

### REQ-8: Device Selection

**User Story:** As a user with multiple keyboards, I want to select which keyboard to remap, so that I can use different configs for different devices.

#### Acceptance Criteria

1. WHEN running `keyrx devices` THEN all connected keyboards SHALL be listed
2. WHEN listing devices THEN device name, vendor ID, product ID SHALL be shown
3. WHEN starting with `--device <id>` THEN only that device SHALL be grabbed
4. WHEN no device specified THEN first keyboard device SHALL be used
5. IF specified device is not found THEN clear error with device list SHALL be shown
6. WHEN device is hot-plugged THEN it SHALL be detectable via `keyrx devices`

### REQ-9: Event Metadata Capture

**User Story:** As a script developer, I want access to comprehensive keyboard event metadata, so that I can implement advanced features like tap-hold, custom modifiers, and device-specific configs.

#### Acceptance Criteria

1. WHEN an event is captured THEN timestamp_us SHALL be included (microseconds since driver start)
2. WHEN an event is captured THEN device_id SHALL identify the source keyboard
3. WHEN an event is captured THEN is_repeat SHALL distinguish auto-repeat from initial press
4. WHEN an event is captured THEN is_synthetic SHALL be true for software-injected events
5. WHEN an event is captured THEN scan_code SHALL contain the raw hardware scan code
6. WHEN processing events THEN is_synthetic=true events SHALL be skippable to prevent infinite loops
7. WHEN metadata is unavailable on a platform THEN sensible defaults SHALL be used
8. ALL metadata fields SHALL be consistent in meaning between Linux and Windows

#### Platform Mapping

| Field | Linux Source | Windows Source |
|-------|-------------|----------------|
| timestamp_us | `evdev::InputEvent.time` | `KBDLLHOOKSTRUCT.time` |
| device_id | `/dev/input/eventX` path | Device instance ID |
| is_repeat | `evdev::InputEvent.value == 2` | Track previous state |
| is_synthetic | Compare to uinput device | `LLKHF_INJECTED` flag |
| scan_code | `evdev::InputEvent.code` | `KBDLLHOOKSTRUCT.scanCode` |

## Non-Functional Requirements

### Performance
- Keyboard capture latency: < 100μs
- Key injection latency: < 100μs
- Total end-to-end latency: < 1ms (p50), < 2ms (p99)
- Memory overhead: < 10MB for driver threads
- CPU usage: < 1% idle, < 5% during typing

### Reliability
- Zero lost key events during normal operation
- Keyboard always recoverable after crash
- No stuck keys after driver restart
- Signal handlers registered for cleanup

### Security
- No elevated privileges required on properly configured systems
- udev rules for Linux permission management
- No network access from driver code

### Compatibility
- Linux: Kernel 5.0+, X11 and Wayland (via evdev)
- Windows: Windows 10 build 1903+
- Support for standard HID keyboards
- USB and Bluetooth keyboards supported
