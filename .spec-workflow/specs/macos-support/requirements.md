# Requirements Document

## Introduction

This specification defines requirements for adding macOS platform support to keyrx, enabling keyboard remapping on macOS with the same sub-millisecond latency and feature parity as existing Linux and Windows implementations. The implementation will leverage Rust-first libraries to minimize Objective-C FFI complexity and maximize memory safety through RAII patterns.

**Value Proposition**: Expands keyrx's cross-platform story to include macOS (15% desktop market share), enabling macOS/Linux dual-boot developers and professional users to maintain consistent keyboard configurations across platforms.

## Alignment with Product Vision

This feature aligns with keyrx's core product vision:

1. **Cross-Platform OS Integration** (product.md): Extends existing Windows/Linux support to macOS, completing coverage of major desktop platforms
2. **Sub-Millisecond Latency Processing** (product.md): Achieves <1ms latency on macOS using CGEventTap API (verified in research)
3. **AI Coding Agent First** (product.md): Maintains deterministic behavior and structured logging on macOS platform
4. **Zero-Cost Abstractions** (product.md): Platform trait abstraction enables macOS support with zero changes to core logic
5. **Future Vision - Extended Platform Support** (product.md line 195-198): Explicitly listed as potential enhancement

**Technical Alignment**:
- Maintains **four-crate architecture** (tech.md): No changes to keyrx_core, keyrx_compiler, or keyrx_ui
- Follows **trait-based platform abstraction** (tech.md line 318-334): Implements existing Platform trait
- Preserves **no_std core design** (tech.md line 601-610): macOS platform code isolated in keyrx_daemon
- Uses **Rust-first dependencies** (tech.md): Leverages battle-tested crates (rdev, enigo, tray-icon)

## Requirements

### Requirement 1: macOS Keyboard Input Capture

**User Story:** As a macOS user, I want keyrx to intercept my keyboard events in real-time, so that I can remap keys with firmware-level latency on macOS

#### Acceptance Criteria

1. WHEN the daemon starts on macOS THEN it SHALL register a CGEventTap callback for keyboard events
2. WHEN the daemon lacks Accessibility permission THEN it SHALL return an actionable error message with setup instructions
3. WHEN a key press event occurs on macOS THEN the system SHALL capture the event within <1ms (95th percentile)
4. WHEN a CGEventTap callback receives a keyboard event THEN it SHALL convert CGKeyCode to keyrx KeyCode using a bidirectional mapping table
5. IF the event matches a remap rule THEN the system SHALL suppress the original event by returning NULL from the callback
6. WHEN multiple keyboards are connected THEN the system SHALL enumerate devices via IOKit and identify them by serial number

### Requirement 2: macOS Keyboard Output Injection

**User Story:** As a macOS user, I want keyrx to inject remapped keyboard events seamlessly, so that applications receive the modified input as if it came from a real keyboard

#### Acceptance Criteria

1. WHEN a remapped event is ready for injection THEN the system SHALL use CGEventPost to inject it at HID level
2. WHEN injecting a key press THEN the system SHALL create both keyDown and keyUp CGEvents with correct modifiers
3. WHEN injection completes THEN the total latency from capture to injection SHALL be <1ms (95th percentile)
4. WHEN the daemon shuts down unexpectedly THEN no keys SHALL remain stuck in pressed state
5. IF injection fails THEN the system SHALL log the error with structured JSON including keycode and error details

### Requirement 3: macOS Device Enumeration and Identification

**User Story:** As a power user with multiple keyboards, I want keyrx to identify each keyboard by serial number on macOS, so that I can configure device-specific remapping rules

#### Acceptance Criteria

1. WHEN the daemon starts THEN it SHALL enumerate USB keyboard devices using IOKit
2. WHEN enumerating devices THEN it SHALL extract Vendor ID (VID), Product ID (PID), and serial number for each device
3. WHEN a device lacks a serial number THEN the system SHALL generate a stable identifier based on VID/PID and USB port
4. WHEN listing devices via API THEN the response SHALL include name, VID, PID, serial, and connection status
5. IF IOKit enumeration fails THEN the system SHALL fall back to input-only mode (no device filtering)

### Requirement 4: macOS System Integration

**User Story:** As a macOS user, I want keyrx to integrate seamlessly with macOS UI conventions, so that I can control the daemon via the menu bar

#### Acceptance Criteria

1. WHEN the daemon starts with web feature enabled THEN it SHALL display a menu bar icon using tray-icon crate
2. WHEN I click the menu bar icon THEN it SHALL show a context menu with options: "Open Web UI", "Reload Config", "Exit"
3. WHEN I select "Reload Config" THEN the daemon SHALL reload the .krx file and update active configuration within 500ms
4. WHEN I select "Exit" THEN the daemon SHALL gracefully shut down, releasing all event taps and injecting key release events for held keys
5. WHEN the daemon crashes THEN the macOS Input Monitoring permission SHALL remain granted for next launch

### Requirement 5: macOS Permission Handling

**User Story:** As a macOS user, I want clear guidance on granting Accessibility permission, so that I can successfully set up keyrx without confusion

#### Acceptance Criteria

1. WHEN the daemon starts without Accessibility permission THEN it SHALL detect this using AXIsProcessTrusted() API
2. WHEN permission is denied THEN the error message SHALL include step-by-step instructions to grant permission
3. WHEN permission is denied THEN the system SHALL suggest running an AppleScript (if available) to auto-open System Preferences
4. IF permission is granted mid-session THEN the daemon SHALL detect it on next config reload (no restart required)
5. WHEN permission status changes THEN the system SHALL log the change with structured JSON including timestamp and new status

### Requirement 6: Cross-Platform Feature Parity

**User Story:** As a user switching between Linux, Windows, and macOS, I want identical keyrx behavior on all platforms, so that my configuration works consistently

#### Acceptance Criteria

1. WHEN using the same .krx config on macOS THEN the remapping behavior SHALL match Linux and Windows byte-for-byte
2. WHEN measuring end-to-end latency THEN macOS SHALL achieve <1ms (matching Linux/Windows requirement)
3. WHEN testing tap-hold behavior THEN DFA state transitions SHALL be deterministic and platform-independent
4. WHEN using multi-device configs THEN macOS SHALL support cross-device modifier sharing (global ExtendedState)
5. IF a feature is unavailable on macOS (e.g., exclusive device grab) THEN the documentation SHALL clearly list the limitation

### Requirement 7: macOS Build and Distribution

**User Story:** As a macOS developer, I want to build keyrx from source with standard Rust tools, so that I can contribute to the project or customize it

#### Acceptance Criteria

1. WHEN running `cargo build --target x86_64-apple-darwin` THEN the build SHALL succeed with no errors
2. WHEN running `cargo build --target aarch64-apple-darwin` THEN the build SHALL succeed for Apple Silicon Macs
3. WHEN building for release THEN the binary SHALL be code-signed with a valid Developer ID Application certificate
4. WHEN distributing via GitHub Releases THEN the binary SHALL be notarized by Apple to avoid Gatekeeper warnings
5. WHEN the CI/CD pipeline runs THEN it SHALL include a macOS runner executing all tests (unit, integration, E2E)

### Requirement 8: macOS Testing and Validation

**User Story:** As a developer, I want automated tests covering macOS-specific code, so that I can verify correctness and prevent regressions

#### Acceptance Criteria

1. WHEN running unit tests THEN all macOS platform code SHALL achieve ≥80% test coverage
2. WHEN running integration tests THEN the system SHALL simulate CGEventTap callbacks with mock events
3. WHEN running E2E tests on macOS hardware THEN tests SHALL verify full capture → remap → injection flow
4. WHEN fuzzing the keycode mapping THEN all 100+ macOS virtual key codes SHALL map correctly to keyrx KeyCode enum
5. IF Accessibility permission is unavailable in CI THEN tests SHALL gracefully skip permission-dependent tests with clear logging

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each macOS module (input_capture.rs, output_injection.rs, device_discovery.rs, etc.) SHALL handle one specific concern
- **Modular Design**: macOS platform code SHALL be isolated in `keyrx_daemon/src/platform/macos/` with no changes to keyrx_core
- **Dependency Management**: macOS-specific crates SHALL be feature-gated with `#[cfg(target_os = "macos")]`
- **Clear Interfaces**: macOS platform SHALL implement the existing Platform trait without modifications to the trait definition

### Performance

- **Latency**: End-to-end latency (OS hook → processing → injection) SHALL be <1ms for 95th percentile
- **Lookup**: Key lookup SHALL remain O(1) using existing MPHF implementation (platform-independent)
- **Memory**: Daemon resident set size SHALL be <50MB on macOS (matching Linux/Windows)
- **CPU**: Idle CPU usage SHALL be <1%, sustained input (1000 keys/sec) SHALL be <5%

### Security

- **Memory Safety**: macOS platform code SHALL minimize unsafe blocks (target <5% of codebase, isolated in IOKit FFI)
- **RAII Patterns**: All Objective-C object lifetimes SHALL be managed via `Retained<T>` wrappers (no manual retain/release)
- **No Secret Logging**: Structured JSON logs SHALL NOT include PII, credentials, or full USB device descriptors
- **Sandbox Compliance**: Code SHALL comply with macOS App Sandbox requirements (if future Mac App Store distribution)

### Reliability

- **Graceful Degradation**: If daemon crashes, input SHALL pass through unmodified (no stuck keys)
- **Permission Errors**: Accessibility permission denial SHALL produce actionable error messages (not silent failures)
- **Resource Cleanup**: Event taps and IOKit iterators SHALL be released in all error paths (no resource leaks)
- **Crash Recovery**: Daemon SHALL support restart within <100ms with no manual intervention

### Usability

- **Setup Clarity**: Accessibility permission setup SHALL be documented with screenshots in `docs/setup/macos.md`
- **Error Messages**: All macOS-specific errors SHALL include next steps (e.g., "Open System Preferences → Security & Privacy")
- **Menu Bar Integration**: Menu bar icon SHALL follow macOS HIG (Human Interface Guidelines) for menu bar items
- **Native Experience**: macOS users SHALL feel the daemon is a native macOS application (not a cross-platform port)
