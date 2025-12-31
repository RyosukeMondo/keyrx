# Requirements Document

## Introduction

This specification refactors the Windows keyboard input system from Low-Level Hooks (`WH_KEYBOARD_LL`) to Raw Input API (`RAWINPUT`) to enable per-device event discrimination. This unlocks the "keyboard-aware remapping" killer feature on Windows, matching Linux's multi-device capabilities.

**Current Limitation:** Windows Low-Level Hooks do not provide device information—all keyboards appear as a single unified input stream. Users cannot configure their USB numpad differently from their main keyboard.

**Solution:** Raw Input API (`RegisterRawInputDevices`, `WM_INPUT`) provides device handles (`HANDLE hDevice`) that map to hardware serial numbers via `GetRawInputDeviceInfo`. This enables device-specific configuration identical to Linux.

**Business Value:** Enables Windows users to repurpose spare keyboards/numpads as macro pads (Stream Deck alternative), a $150+ value proposition using $12 hardware.

## Alignment with Product Vision

This feature directly supports the product vision outlined in `.spec-workflow/steering/product.md`:

### Multi-Device Support (Product Goal #7)
> **"Serial number-based identification"**: True per-device configs (not USB port-dependent)
> **"Industry-first serial number support"**: Fills gap left by Karabiner-Elements

This spec achieves serial number discrimination on Windows, completing the cross-platform story:
- **vs AutoHotkey**: AHK cannot reliably discriminate devices (WH_KEYBOARD_LL limitation)
- **vs kmonad**: No Windows device discrimination support
- **vs keyrx (Linux)**: This brings Windows to parity with Linux's evdev serial tracking

### Performance Requirements
> **"<1ms end-to-end latency"**, **"Sub-millisecond latency processing"**

Raw Input has **lower latency** than LL Hooks (kernel mode vs user mode callbacks), so this refactor *improves* performance while adding functionality.

## Requirements

### Requirement 1: Replace Low-Level Hooks with Raw Input API

**User Story:** As a Windows developer, I want to replace `WH_KEYBOARD_LL` with `RegisterRawInputDevices` so that the system can identify which keyboard generated each keystroke.

#### Acceptance Criteria

1. WHEN daemon starts on Windows THEN it SHALL call `RegisterRawInputDevices` with `RIDEV_INPUTSINK` and `RIDEV_NOLEGACY` flags
2. WHEN keyboard event occurs THEN daemon SHALL receive `WM_INPUT` message in window procedure
3. WHEN processing `WM_INPUT` THEN daemon SHALL call `GetRawInputData` to extract `RAWINPUT` structure
4. IF hook installation succeeds THEN daemon SHALL NOT fall back to LL Hooks (complete replacement)
5. WHEN daemon exits THEN it SHALL unregister raw input devices via `RegisterRawInputDevices` with `RIDEV_REMOVE` flag

### Requirement 2: Extract Device Handle from WM_INPUT

**User Story:** As a Windows developer, I want to extract the device handle (`HANDLE hDevice`) from each `WM_INPUT` message so that I can map it to a device serial number.

#### Acceptance Criteria

1. WHEN `WM_INPUT` message is received THEN daemon SHALL extract `RAWINPUT.header.hDevice` handle
2. WHEN device handle is extracted THEN daemon SHALL store it with the keyboard event
3. IF `hDevice` is null or invalid THEN daemon SHALL log error and skip the event (do not process)
4. WHEN event is processed THEN the `hDevice` SHALL be associated with the resulting `KeyEvent`

### Requirement 3: Map Device Handle to Serial Number

**User Story:** As a Windows user, I want each keyboard identified by its USB serial number so that I can write device-specific remapping rules that persist across reboots (not dependent on USB port).

#### Acceptance Criteria

1. WHEN device handle is obtained THEN daemon SHALL call `GetRawInputDeviceInfo` with `RIDI_DEVICENAME` to get device path
2. WHEN device path is retrieved THEN it SHALL be in format `\\?\HID#VID_XXXX&PID_YYYY#SerialNumber#{GUID}`
3. WHEN parsing device path THEN daemon SHALL extract the serial number substring
4. IF serial number is unavailable THEN daemon SHALL use full device path as fallback identifier (stable across sessions)
5. WHEN device_id is determined THEN it SHALL be added to `KeyEvent` via `with_device_id()` method

### Requirement 4: Maintain System Tray Functionality

**User Story:** As a Windows user, I want the system tray icon to continue working exactly as before so that I can still Reload and Exit via the GUI.

#### Acceptance Criteria

1. WHEN daemon refactor is complete THEN system tray SHALL function identically to current behavior
2. WHEN user clicks "Reload Config" THEN daemon SHALL reload .krx file (no behavior change)
3. WHEN user clicks "Exit" THEN daemon SHALL unregister raw input, cleanup, and terminate gracefully
4. IF tray implementation uses SystemTray trait (from Spec 1) THEN this refactor SHALL not break that abstraction

### Requirement 5: Performance Equivalence or Improvement

**User Story:** As a competitive gamer, I need <1ms latency so that raw input refactor does not degrade performance compared to LL Hooks.

#### Acceptance Criteria

1. WHEN measuring end-to-end latency THEN raw input SHALL achieve ≤1ms (same or better than LL Hooks)
2. WHEN benchmarking event processing THEN overhead SHALL be <100μs (target: <50μs)
3. IF raw input introduces overhead THEN optimizations SHALL be applied (e.g., device handle caching, pre-allocated buffers)
4. WHEN running performance tests THEN latency percentiles (p50, p95, p99) SHALL not regress compared to baseline

### Requirement 6: Backward Compatibility with Rhai Configs

**User Story:** As a keyrx user, I want existing Rhai configurations to work without modification so that upgrading to device discrimination doesn't break my setup.

#### Acceptance Criteria

1. WHEN existing Rhai config is loaded THEN it SHALL compile and execute without errors (device_id is optional)
2. WHEN config does not check `device_id` THEN remapping SHALL apply to all devices (current behavior preserved)
3. WHEN config uses `event.device_id()` conditionals THEN only matching devices SHALL trigger remapping
4. IF user has single keyboard THEN behavior SHALL be identical to pre-refactor version

### Requirement 7: Device Enumeration at Startup

**User Story:** As a Windows user, I want the daemon to list all connected keyboards at startup so that I can verify which devices are being monitored.

#### Acceptance Criteria

1. WHEN daemon starts THEN it SHALL enumerate all HID keyboard devices via `GetRawInputDeviceList`
2. WHEN device is enumerated THEN daemon SHALL log: device name, serial number (or fallback ID), device handle
3. IF no keyboards are detected THEN daemon SHALL log error and exit with code 1 (cannot operate without input)
4. WHEN device is enumerated THEN its info SHALL be available via `/api/devices` endpoint (matching Linux behavior)

### Requirement 8: Graceful Handling of Device Hot-Plug

**User Story:** As a Windows user, I want the daemon to detect when I plug in a new USB keyboard so that it starts processing events from that device automatically.

#### Acceptance Criteria

1. WHEN USB keyboard is connected THEN daemon SHALL receive `WM_INPUT_DEVICE_CHANGE` notification
2. WHEN new device is detected THEN daemon SHALL query its info via `GetRawInputDeviceInfo` and add to device list
3. WHEN device is disconnected THEN daemon SHALL remove it from device list and continue with remaining devices
4. IF only one keyboard is unplugged THEN daemon SHALL continue operating with other keyboards (no crash)
5. WHEN device is hot-plugged THEN web UI `/api/devices` SHALL update to reflect new device list

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**:
  - `platform/windows/rawinput.rs`: Raw input registration and message handling only
  - `platform/windows/device_map.rs`: Device handle → serial number mapping only
  - `platform/windows/hook.rs`: Delete this file (replaced by rawinput.rs)
- **Modular Design**:
  - Device mapping logic isolated from event processing (testable independently)
  - Window procedure separated from input parsing
- **Dependency Management**:
  - Only `windows-sys` crate (already in workspace, no new deps)
  - Use `Win32::UI::Input::KeyboardAndMouse` and `Win32::Devices::HumanInterfaceDevice` modules
- **Clear Interfaces**:
  - `InputDevice` trait implementation unchanged (still returns `KeyEvent`)
  - Device ID propagation via existing `KeyEvent::with_device_id()` (no API changes)

### Performance
- **Latency**: Raw input SHALL achieve <1ms end-to-end (target: <500μs, better than LL Hooks)
- **Memory**: Device map SHALL use <100KB (cache device handles → serial numbers, max 10 devices)
- **CPU**: Raw input processing SHALL use <1% CPU on idle, <5% under sustained input (1000 events/sec)

### Reliability
- **Fallback**: No fallback to LL Hooks (raw input is superior, if it fails daemon should exit with clear error)
- **Hot-Plug**: Device connect/disconnect SHALL NOT crash daemon (handled via `WM_INPUT_DEVICE_CHANGE`)
- **Device Errors**: Invalid `hDevice` or `GetRawInputDeviceInfo` failure SHALL log warning and skip event (do not crash)

### Usability
- **Device Naming**: Log human-readable device names (from `RIDI_DEVICENAME`) at startup
- **Error Messages**: If raw input registration fails, log: "Failed to register raw input (error code: XXX). Try running as Administrator."
- **Migration**: Document breaking change (LL Hooks removed) in release notes, explain why (device discrimination)

### Security
- **Injected Event Filtering**: Raw input SHALL ignore `RIM_INPUT_SINK` flag to avoid processing injected events (prevents infinite loops)
- **Administrator Privileges**: Raw input with `RIDEV_INPUTSINK` requires admin on some Windows versions—document this requirement
