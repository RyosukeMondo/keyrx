# Requirements Document

## Introduction

This specification adds two critical features to the Linux platform to achieve parity with Windows and unlock the "keyboard-aware remapping" market positioning:

1. **System Tray Integration**: Implement system tray icon with Reload/Exit controls (matching Windows functionality)
2. **Multi-Device Event Discrimination**: Expose device serial numbers in KeyEvent to enable per-device configuration (e.g., "turn your $12 numpad into a $150 Stream Deck")

These features enable users to:
- Control the daemon via GUI (reload configs, exit gracefully) without CLI
- Configure different remapping rules per physical keyboard (main keyboard vs USB numpad vs macro pad)
- Identify which keyboard sent which keystroke for advanced workflows

## Alignment with Product Vision

This feature directly supports the product vision outlined in `.spec-workflow/steering/product.md`:

### Multi-Device Support (Product Goal #7)
> **"N:M device-to-configuration mapping"**: Multiple keyboards, modular configs with **"Serial number-based identification"**

This spec implements the Linux side of true per-device configuration, filling the gap left by competitors:
- **vs Karabiner-Elements**: Karabiner lacks serial number support (open issue #2007)
- **vs kmonad**: No per-device support (open issue #318 since 2021)
- **vs xremap**: Linux-only and limited to YAML static rules

### Platform Parity
Windows already has system tray (via `tray.rs`). This spec brings Linux to feature parity, enabling:
- Consistent UX across platforms
- Code reuse through platform trait abstraction
- Professional deployment (no CLI-only limitation)

## Requirements

### Requirement 1: Cross-Platform System Tray Abstraction

**User Story:** As a developer, I want a platform-agnostic SystemTray trait so that Linux and Windows can share tray icon logic without duplication.

#### Acceptance Criteria

1. WHEN implementing system tray THEN code SHALL define a `SystemTray` trait in `keyrx_daemon/src/platform/mod.rs`
2. WHEN trait is defined THEN it SHALL include methods: `new()`, `poll_event() -> Option<TrayControlEvent>`, and shutdown cleanup
3. WHEN Windows tray is refactored THEN it SHALL implement the `SystemTray` trait with no behavior changes
4. IF either platform's tray emits Reload or Exit events THEN the daemon's main loop SHALL handle them identically

### Requirement 2: Linux System Tray Implementation

**User Story:** As a Linux user, I want a system tray icon for keyrx_daemon so that I can reload configuration and exit the daemon without using the terminal.

#### Acceptance Criteria

1. WHEN daemon starts on Linux THEN a tray icon SHALL appear in the system tray with label "KeyRx Daemon"
2. WHEN user right-clicks the tray icon THEN a context menu SHALL display with options: "Reload Config", separator, "Exit"
3. WHEN user clicks "Reload Config" THEN daemon SHALL reload the .krx configuration file and apply changes without restart
4. WHEN user clicks "Exit" THEN daemon SHALL release grabbed devices, cleanup resources, and terminate gracefully within 500ms
5. IF system tray is unavailable (headless server) THEN daemon SHALL log warning and continue without tray (degraded mode)
6. WHEN daemon exits THEN tray icon SHALL be removed from system tray automatically

### Requirement 3: Device ID in KeyEvent Structure

**User Story:** As a developer, I want KeyEvent to include a device identifier so that the remapping engine can apply different rules per physical keyboard.

#### Acceptance Criteria

1. WHEN KeyEvent is created THEN it SHALL include an optional `device_id: Option<String>` field
2. WHEN device_id is None THEN remapping engine SHALL use default configuration (backward compatible)
3. WHEN device_id is Some(id) THEN Rhai scripts SHALL access it via `event.device_id()` function
4. IF KeyEvent is created without device info THEN device_id SHALL default to None (no breakage of existing code)

### Requirement 4: Linux Device Enumeration and ID Tracking

**User Story:** As a Linux user, I want keyrx to distinguish between my laptop keyboard and my USB numpad so that I can configure them independently (e.g., numpad as macro pad).

#### Acceptance Criteria

1. WHEN daemon starts on Linux THEN it SHALL enumerate all keyboard devices in `/dev/input/by-id/` or `/dev/input/event*`
2. WHEN opening each device THEN daemon SHALL extract serial number via `EvdevInput::serial()` (or generate fallback ID if serial unavailable)
3. WHEN device sends key event THEN the event SHALL be tagged with the device's serial number as `device_id`
4. IF device has no serial number THEN daemon SHALL use device path (e.g., `/dev/input/event3`) as stable identifier
5. WHEN user connects new USB keyboard THEN daemon SHALL detect it and add to device list (hot-plug support is future, not required for v1)

### Requirement 5: Device Information in Web UI

**User Story:** As a user, I want to see which keyboards are currently active in the web dashboard so that I can verify my device-specific configuration is applied to the correct keyboard.

#### Acceptance Criteria

1. WHEN user opens web UI THEN a "Connected Devices" section SHALL display list of all input devices
2. WHEN device is active THEN its entry SHALL show: name (e.g., "Logitech USB Keyboard"), serial number (if available), device path
3. WHEN device sends event THEN UI SHALL highlight the device briefly (visual feedback)
4. IF user has configured device-specific rules THEN UI SHALL display which config applies to each device

### Requirement 6: Rhai API for Device-Specific Configuration

**User Story:** As a power user, I want to write Rhai scripts that check which keyboard sent a keystroke so that I can remap my USB numpad to OBS hotkeys while keeping my main keyboard normal.

#### Acceptance Criteria

1. WHEN Rhai script is evaluated THEN it SHALL have access to `event.device_id()` function returning `Option<String>`
2. WHEN device_id is available THEN script SHALL use conditional logic: `if event.device_id() == "USB\\SERIAL_NUMPAD" { /* ... */ }`
3. WHEN device_id is None THEN script SHALL treat event as coming from default device
4. IF user configures per-device mapping THEN only that device's events SHALL match the rule (other devices unaffected)

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**:
  - `platform/tray.rs` (Linux): System tray implementation only
  - `platform/mod.rs`: SystemTray trait definition, no platform-specific code
  - `platform/linux/mod.rs`: Device enumeration and ID management
- **Modular Design**:
  - System tray code abstracted via trait (Windows and Linux implement same interface)
  - Device ID tracking isolated in platform layer (core logic unchanged)
- **Dependency Management**:
  - Linux tray uses `ksni` crate (KDE StatusNotifierItem) or `libayatana-appindicator` (GTK)
  - Minimize new dependencies (reuse existing `tray-icon` crate if cross-platform)
- **Clear Interfaces**:
  - `SystemTray` trait defines contract for all platforms
  - `InputDevice::next_event()` returns KeyEvent with device_id populated

### Performance
- **Latency Impact**: Device ID lookup SHALL add <10μs overhead (string comparison optimized via interning or hash map)
- **Memory**: Device enumeration SHALL use <1MB additional memory (device list stored once at startup)
- **Tray Polling**: `poll_event()` SHALL use non-blocking check (<1μs overhead per main loop iteration)

### Reliability
- **Tray Fallback**: If system tray init fails, daemon SHALL continue without tray (degraded mode, CLI still works)
- **Device Hot-Plug**: V1 SHALL detect devices at startup only; hot-plug detection is future enhancement
- **Graceful Shutdown**: Tray "Exit" SHALL trigger same cleanup path as SIGTERM (release grabs, flush logs)

### Usability
- **Tray Icon Design**: Use existing `assets/icon.png` (same as Windows) for visual consistency
- **Error Messages**: If tray fails on headless server, log: "System tray unavailable (headless environment), use CLI to control daemon"
- **Device Naming**: Show human-readable device names (from evdev) + serial in UI, not just paths
