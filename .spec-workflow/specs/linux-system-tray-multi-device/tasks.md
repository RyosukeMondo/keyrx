# Tasks Document

- [x] 1. Add device_id field to KeyEvent structure
  - File: keyrx_core/src/runtime/event.rs
  - Add `device_id: Option<String>` field to KeyEvent struct
  - Implement `with_device_id()` builder method
  - Implement `device_id()` accessor method
  - Ensure backward compatibility (default to None)
  - Purpose: Enable device discrimination in event pipeline
  - _Leverage: Existing `with_timestamp()` pattern_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Prompt: Role: Rust Developer specializing in data structures and backward compatibility | Task: Extend KeyEvent structure with optional device_id field following requirement 3.1-3.4, using the existing with_timestamp() builder pattern from keyrx_core/src/runtime/event.rs | Restrictions: Must maintain backward compatibility, do not change existing constructor signatures, ensure None default for device_id | Success: KeyEvent compiles with new field, existing code unaffected, with_device_id() and device_id() methods work correctly, all existing tests pass_

- [x] 2. Create cross-platform SystemTray trait
  - File: keyrx_daemon/src/platform/mod.rs
  - Define `TrayControlEvent` enum (Reload, Exit)
  - Define `SystemTray` trait with methods: new(), poll_event(), shutdown()
  - Move TrayControlEvent from windows/tray.rs to platform/mod.rs
  - Purpose: Abstract system tray interface for code reuse
  - _Leverage: Existing TrayControlEvent enum from windows/tray.rs_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Role: Software Architect specializing in trait design and cross-platform abstractions | Task: Design and implement SystemTray trait following requirements 1.1-1.4, moving TrayControlEvent from windows/tray.rs to platform/mod.rs for cross-platform use | Restrictions: Must not break existing Windows tray code, keep trait minimal and focused, ensure trait methods are implementable on both Linux and Windows | Success: Trait compiles, TrayControlEvent accessible from both platforms, trait contract is clear and documented_

- [x] 3. Refactor Windows tray to implement SystemTray trait
  - File: keyrx_daemon/src/platform/windows/tray.rs
  - Implement `SystemTray` trait for `TrayIconController`
  - Ensure no behavior changes (trait is pure refactor)
  - Update imports to use platform::TrayControlEvent
  - Purpose: Ensure Windows tray conforms to cross-platform interface
  - _Leverage: Existing TrayIconController implementation_
  - _Requirements: 1.3_
  - _Prompt: Role: Rust Developer with expertise in trait implementation and refactoring | Task: Refactor existing TrayIconController to implement the SystemTray trait following requirement 1.3, ensuring zero behavior changes | Restrictions: Do not change menu structure or event handling logic, only add trait implementation, maintain existing functionality exactly | Success: TrayIconController implements SystemTray trait, all Windows tray tests pass, no behavior regressions_

- [x] 4. Implement Linux system tray using ksni
  - File: keyrx_daemon/src/platform/linux/tray.rs (new file)
  - Add ksni dependency to Cargo.toml
  - Create LinuxSystemTray struct implementing SystemTray trait
  - Implement ksni::Tray for internal TrayService
  - Create menu with Reload and Exit items
  - Load icon from assets/icon.png
  - Purpose: Provide system tray UI for Linux users
  - _Leverage: Icon loading pattern from windows/tray.rs, TrayControlEvent enum_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  - _Prompt: Role: Linux Desktop Developer with expertise in system tray protocols and ksni crate | Task: Implement Linux system tray following requirements 2.1-2.6 using ksni crate, mirroring Windows tray structure with Reload/Exit menu | Restrictions: Must implement SystemTray trait exactly, handle tray unavailable gracefully (return error, don't panic), use crossbeam_channel for event passing | Success: Tray icon appears in KDE/GNOME, menu items trigger correct events, icon loads from assets/icon.png, graceful fallback if tray unavailable_

- [x] 5. Create DeviceManager for Linux
  - File: keyrx_daemon/src/platform/linux/device_manager.rs (new file)
  - Define DeviceInfo struct with id, name, path, serial fields
  - Implement DeviceManager::enumerate() to scan /dev/input/event*
  - Extract device ID via EvdevInput::serial() or fallback to path
  - Implement next_event() to poll all devices and return (KeyEvent, device_id)
  - Purpose: Manage multiple input devices with unique identifiers
  - _Leverage: EvdevInput::open(), EvdevInput::serial(), EvdevInput::name()_
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Prompt: Role: Linux Systems Developer with expertise in evdev and device management | Task: Implement DeviceManager to enumerate and manage multiple input devices following requirements 4.1-4.4, leveraging EvdevInput methods from linux/mod.rs | Restrictions: Must filter for keyboard devices only (ignore mice), use serial() when available or fallback to stable path-based ID, handle permission errors gracefully | Success: Enumerates all keyboards, assigns unique IDs, next_event() returns device_id with each event, handles devices without serial numbers_

- [x] 6. Integrate DeviceManager into Linux platform
  - File: keyrx_daemon/src/platform/linux/mod.rs
  - Update LinuxPlatform to use DeviceManager instead of single device
  - Modify init() to enumerate devices via DeviceManager
  - Update process_events() to call DeviceManager::next_event()
  - Tag KeyEvent with device_id using with_device_id()
  - Purpose: Enable multi-device support in Linux daemon
  - _Leverage: DeviceManager from task 5, KeyEvent::with_device_id() from task 1_
  - _Requirements: 4.3_
  - _Prompt: Role: Backend Developer with expertise in Rust async I/O and event loop design | Task: Integrate DeviceManager into LinuxPlatform following requirement 4.3, updating init() and process_events() to handle multiple devices | Restrictions: Must maintain existing event processing logic, preserve <1ms latency, handle device enumeration errors at startup, ensure graceful shutdown releases all devices | Success: Daemon opens multiple devices, events tagged with correct device_id, main loop processes events from all devices, no latency regression_

- [ ] 7. Add tray support to Linux platform
  - File: keyrx_daemon/src/platform/linux/mod.rs
  - Add LinuxSystemTray field to LinuxPlatform struct (Option type for graceful fallback)
  - Initialize tray in init() with error handling (log warning if fails)
  - Poll tray events in process_events() loop
  - Handle TrayControlEvent::Reload and TrayControlEvent::Exit
  - Purpose: Provide GUI control for Linux daemon
  - _Leverage: LinuxSystemTray from task 4, SystemTray trait from task 2_
  - _Requirements: 2.5_
  - _Prompt: Role: System Integration Engineer with expertise in daemon lifecycle management | Task: Integrate LinuxSystemTray into LinuxPlatform following requirement 2.5, handling tray unavailable gracefully with degraded mode | Restrictions: Must not crash if tray init fails, log clear warning for headless environments, poll_event() must be non-blocking (<1μs overhead), ensure tray cleanup on shutdown | Success: Tray appears on GUI systems, daemon continues without tray on headless servers, Reload triggers config reload, Exit performs clean shutdown_

- [ ] 8. Add /api/devices endpoint to web server
  - File: keyrx_daemon/src/web/api.rs (or create if doesn't exist)
  - Create GET /api/devices endpoint returning JSON device list
  - Endpoint calls platform layer to get device info
  - Return array of {id, name, path, serial, active} objects
  - Purpose: Display connected devices in web UI
  - _Leverage: DeviceManager::device_ids() and device_info() from task 5_
  - _Requirements: 5.1, 5.2_
  - _Prompt: Role: Full-stack Developer with expertise in REST API design and axum framework | Task: Implement /api/devices endpoint following requirements 5.1-5.2, returning JSON list of connected input devices | Restrictions: Must use existing axum router configuration, ensure CORS headers if needed, handle empty device list gracefully, return 200 OK with empty array if no devices | Success: GET /api/devices returns valid JSON, includes all enumerated devices, response matches schema in design.md, endpoint accessible from React frontend_

- [ ] 9. Create React component for device list in UI
  - File: keyrx_ui/src/components/DeviceList.tsx (new file)
  - Fetch device list from /api/devices on mount
  - Display table with columns: Name, Serial, Path, Status
  - Highlight active device when it sends event (via WebSocket)
  - Purpose: Visualize connected devices for user verification
  - _Leverage: Existing React hooks, axum WebSocket integration_
  - _Requirements: 5.1, 5.2, 5.3_
  - _Prompt: Role: Frontend Developer specializing in React and real-time UI updates | Task: Create DeviceList component following requirements 5.1-5.3, fetching from /api/devices and highlighting active devices via WebSocket | Restrictions: Must use existing fetch patterns from other components, handle loading/error states, update UI smoothly without flickering, use existing theme/styling | Success: Component renders device list, data fetches on mount, devices highlighted on activity, no console errors, responsive design_

- [ ] 10. Add Rhai bindings for device_id access
  - File: keyrx_compiler/src/rhai_bindings.rs (or equivalent)
  - Expose event.device_id() method to Rhai scripts
  - Return Option<String> (None if device_id not set)
  - Add example Rhai script to documentation
  - Purpose: Enable per-device configuration in Rhai
  - _Leverage: Existing Rhai event bindings, KeyEvent::device_id() from task 1_
  - _Requirements: 6.1, 6.2, 6.3, 6.4_
  - _Prompt: Role: Compiler Engineer with expertise in Rhai FFI and scripting language bindings | Task: Expose device_id() method to Rhai scripts following requirements 6.1-6.4, allowing conditional logic based on device ID | Restrictions: Must follow existing Rhai binding patterns, handle None case gracefully in Rhai (return null or empty string), ensure compile-time evaluation only (no runtime overhead), document with example | Success: Rhai scripts can call event.device_id(), conditional logic works (if device_id == "numpad"), compiles to static .krx without runtime checks, example script provided_

- [ ] 11. Write integration tests for multi-device support
  - File: keyrx_daemon/tests/multi_device_integration.rs (new file)
  - Test DeviceManager enumerates multiple mock devices
  - Test events tagged with correct device_id
  - Test Rhai per-device remapping (numpad → F13, main → passthrough)
  - Test web API returns correct device list
  - Purpose: Verify end-to-end multi-device functionality
  - _Leverage: Existing integration test framework, mock platform from platform/mock.rs_
  - _Requirements: All_
  - _Prompt: Role: QA Automation Engineer with expertise in Rust integration testing and mocking | Task: Write comprehensive integration tests covering all multi-device requirements, using mock devices to simulate multiple keyboards | Restrictions: Must use existing test utilities, tests must be deterministic (no timing dependencies), mock at platform boundary not evdev layer, ensure tests run in CI without real hardware | Success: Tests cover device enumeration, event tagging, per-device remapping, web API, all tests pass consistently, achieve 85%+ coverage of new code_

- [ ] 12. Update documentation with multi-device examples
  - File: docs/multi-device-configuration.md (new file) or update existing docs
  - Document how to identify device IDs (serial numbers vs paths)
  - Provide example Rhai config for "numpad as Stream Deck"
  - Document tray menu usage (Reload, Exit)
  - Troubleshooting: permissions, headless mode, device hot-plug
  - Purpose: Guide users through multi-device setup
  - _Leverage: Existing documentation structure_
  - _Requirements: All_
  - _Prompt: Role: Technical Writer with expertise in system administration and user guides | Task: Create comprehensive multi-device configuration guide with examples, troubleshooting, and best practices covering all requirements | Restrictions: Must use clear, beginner-friendly language, provide copy-paste examples, include permission setup commands for Linux, explain serial number vs path-based IDs | Success: Documentation covers device identification, example configs, tray usage, troubleshooting, reviewed for clarity and completeness_
