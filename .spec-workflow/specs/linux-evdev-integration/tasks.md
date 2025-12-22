# Tasks Document

## Phase 1: evdev Input Implementation

- [x] 1. Add Linux platform dependencies
  - File: `keyrx_daemon/Cargo.toml`
  - Add `evdev` crate for input device access
  - Add `nix` crate for Unix utilities (ioctl, signals)
  - Add `signal-hook` crate for signal handling
  - Add platform-specific cfg attributes
  - Purpose: Enable Linux-specific functionality
  - _Leverage: Existing Cargo.toml structure_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Role: Rust Developer with expertise in cross-platform development and Cargo configuration | Task: Add Linux-specific dependencies (evdev, nix, signal-hook) to keyrx_daemon/Cargo.toml with appropriate cfg attributes for platform-specific compilation, following requirements 1.1 and 2.1 | Restrictions: Must use target.'cfg(target_os = "linux")'.dependencies for platform-specific deps, keep existing dependencies unchanged, use latest stable versions of crates, add comments explaining each dependency's purpose | Success: evdev, nix, and signal-hook added with Linux-only cfg, cargo build compiles on Linux, cargo build compiles on non-Linux (skipping platform deps), no version conflicts_

- [x] 2. Create KeyCode mapping functions
  - File: `keyrx_daemon/src/platform/linux.rs` (NEW)
  - Implement `evdev_to_keycode(code: u16) -> Option<KeyCode>` for all keys
  - Implement `keycode_to_evdev(keycode: KeyCode) -> u16` for output
  - Cover all KeyCode variants (A-Z, F1-F24, modifiers, special keys)
  - Purpose: Bridge between evdev codes and keyrx KeyCode enum
  - _Leverage: keyrx_core::config::KeyCode enum (existing)_
  - _Requirements: 1.4_
  - _Prompt: Role: Rust Developer with expertise in keyboard input systems and Linux evdev | Task: Create KeyCode mapping functions between evdev event codes and keyrx KeyCode enum, covering all keyboard keys following requirement 1.4 | Restrictions: Must handle all KeyCode variants exhaustively with match, return None for unknown evdev codes (for passthrough handling), use evdev::Key constants (KEY_A, KEY_LEFTSHIFT, etc.), add comprehensive doc comments with mapping table, organize by key category (letters, modifiers, function keys, special) | Success: evdev_to_keycode maps all evdev key codes to KeyCode, keycode_to_evdev maps all KeyCode variants to evdev codes, round-trip conversion is identity (evdev→KeyCode→evdev), unknown codes return None, compiles without warnings_

- [x] 3. Implement EvdevInput struct and constructor
  - File: `keyrx_daemon/src/platform/linux.rs` (continue)
  - Define `EvdevInput` struct with `evdev::Device` and `grabbed: bool`
  - Implement `open(path: &Path) -> Result<Self, DeviceError>`
  - Implement `from_device(device: evdev::Device) -> Self`
  - Add accessors: `name()`, `serial()`, `path()`
  - Purpose: Wrap evdev device with keyrx interface
  - _Leverage: evdev crate Device type_
  - _Requirements: 1.1, 1.3_
  - _Prompt: Role: Rust Systems Programmer with expertise in Linux device access | Task: Implement EvdevInput struct wrapping evdev::Device with constructor and accessor methods for device metadata, following requirements 1.1 and 1.3 | Restrictions: Must use evdev::Device::open for path-based opening, handle io::Error and convert to DeviceError::NotFound or DeviceError::PermissionDenied based on error kind, serial() should return Option<&str> (not all devices have serial), store grabbed flag for tracking state, add doc comments with usage examples | Success: EvdevInput::open opens device by path, returns PermissionDenied for EACCES, returns NotFound for ENOENT, from_device wraps existing device, name/serial/path accessors work correctly, compiles and can open /dev/input/event* on Linux_

- [x] 4. Implement InputDevice trait for EvdevInput
  - File: `keyrx_daemon/src/platform/linux.rs` (continue)
  - Implement `next_event(&mut self) -> Result<KeyEvent, DeviceError>`
  - Implement `grab(&mut self) -> Result<(), DeviceError>` using EVIOCGRAB
  - Implement `release(&mut self) -> Result<(), DeviceError>`
  - Handle key press (value=1), release (value=0), ignore repeat (value=2)
  - Purpose: Enable keyboard event capture from real devices
  - _Leverage: keyrx_daemon::platform::InputDevice trait (existing)_
  - _Requirements: 1.2, 1.4_
  - _Prompt: Role: Rust Systems Programmer with expertise in Linux evdev subsystem | Task: Implement InputDevice trait for EvdevInput with event reading, exclusive grab, and release functionality following requirements 1.2 and 1.4 | Restrictions: Must use evdev::Device::fetch_events or next_event for reading, filter for EV_KEY events only, map value 1 to KeyEvent::Press and value 0 to KeyEvent::Release, ignore value 2 (repeat), use EVIOCGRAB ioctl (via evdev or nix) for grab/release, update grabbed flag on success, return DeviceError::Io for read errors, return DeviceError::EndOfStream when appropriate | Success: next_event returns KeyEvent::Press/Release for key events, grab acquires exclusive access (other programs don't receive events), release restores normal access, repeat events are filtered out, compiles and works with real keyboard on Linux_

- [x] 5. Write EvdevInput unit tests
  - File: `keyrx_daemon/src/platform/linux.rs` (add tests module)
  - Test KeyCode mapping functions (evdev_to_keycode, keycode_to_evdev)
  - Test round-trip conversion for all keys
  - Test error handling (PermissionDenied, NotFound)
  - Purpose: Verify evdev input implementation correctness
  - _Leverage: None (unit tests)_
  - _Requirements: 1.1-1.4_
  - _Prompt: Role: QA Engineer with Linux testing expertise | Task: Create unit tests for EvdevInput covering KeyCode mappings and error handling following requirements 1.1-1.4 | Restrictions: KeyCode mapping tests should work without real devices (pure function tests), test all key categories (letters, numbers, modifiers, function keys), verify round-trip identity, test error construction with correct messages, use #[cfg(target_os = "linux")] for Linux-only tests, add integration test that requires real device (marked #[ignore] for CI) | Success: KeyCode mapping tests pass for all variants, round-trip tests verify identity, error tests verify correct DeviceError variants, tests compile on Linux, ignored tests document real-device requirements_

## Phase 2: uinput Output Implementation

- [x] 6. Implement UinputOutput struct and constructor
  - File: `keyrx_daemon/src/platform/linux.rs` (continue)
  - Define `UinputOutput` struct wrapping uinput device
  - Implement `create(name: &str) -> Result<Self, DeviceError>`
  - Configure device with full keyboard capabilities (all KEY_* events)
  - Set device name for identification
  - Purpose: Create virtual keyboard for event injection
  - _Leverage: uinput crate or direct ioctl via nix_
  - _Requirements: 2.1, 2.5_
  - _Prompt: Role: Rust Systems Programmer with expertise in Linux uinput subsystem | Task: Implement UinputOutput struct that creates virtual keyboard device via uinput, configuring all keyboard capabilities following requirements 2.1 and 2.5 | Restrictions: Must open /dev/uinput with write access, configure EV_KEY capability for all KEY_* codes, set device name via UI_SET_* ioctls, handle permission errors with DeviceError::PermissionDenied, include helpful error message about udev rules, add doc comments explaining uinput setup | Success: UinputOutput::create opens /dev/uinput successfully (as root or with udev rules), creates virtual device visible in /dev/input/, device has correct name, returns PermissionDenied with helpful message if access denied_

- [x] 7. Implement OutputDevice trait for UinputOutput
  - File: `keyrx_daemon/src/platform/linux.rs` (continue)
  - Implement `inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError>`
  - Convert KeyEvent to uinput event struct
  - Write EV_KEY event followed by EV_SYN/SYN_REPORT
  - Purpose: Enable remapped key injection to system
  - _Leverage: keyrx_daemon::platform::OutputDevice trait (existing)_
  - _Requirements: 2.2, 2.3_
  - _Prompt: Role: Rust Systems Programmer with expertise in Linux input injection | Task: Implement OutputDevice trait for UinputOutput with event injection and sync following requirements 2.2 and 2.3 | Restrictions: Must convert KeyEvent::Press to EV_KEY with value 1, KeyEvent::Release to EV_KEY with value 0, write EV_SYN/SYN_REPORT after each key event to flush, use keycode_to_evdev for code conversion, return DeviceError::InjectionFailed on write errors, add doc comments explaining sync requirement | Success: inject_event writes correct uinput events, EV_SYN ensures immediate delivery, Press events result in key down in applications, Release events result in key up, compiles and injects keys visible in xev or evtest_

- [x] 8. Implement UinputOutput cleanup
  - File: `keyrx_daemon/src/platform/linux.rs` (continue)
  - Add `destroy(&mut self) -> Result<(), DeviceError>` method
  - Release any held keys before destroying
  - Properly close uinput device
  - Implement Drop trait for automatic cleanup
  - Purpose: Ensure clean shutdown without orphaned virtual devices
  - _Leverage: uinput device destruction API_
  - _Requirements: 2.4_
  - _Prompt: Role: Rust Developer with expertise in resource cleanup and RAII | Task: Implement proper cleanup for UinputOutput including held key release and device destruction, with Drop trait implementation following requirement 2.4 | Restrictions: Must track held keys (HashSet<KeyCode>) if needed, inject release events for any held keys before destroy, use UI_DEV_DESTROY ioctl to remove device, close file descriptor, implement Drop to ensure cleanup even on panic, log cleanup at DEBUG level | Success: destroy() releases held keys and removes virtual device, Drop calls destroy automatically, no orphaned devices after daemon exit (verify with ls /dev/input/), clean shutdown even on panic_

- [ ] 9. Write UinputOutput unit tests
  - File: `keyrx_daemon/src/platform/linux.rs` (add to tests module)
  - Test device creation with correct capabilities
  - Test event injection (requires root or udev rules)
  - Test cleanup removes virtual device
  - Purpose: Verify uinput output implementation correctness
  - _Leverage: None (unit tests)_
  - _Requirements: 2.1-2.5_
  - _Prompt: Role: QA Engineer with Linux testing expertise | Task: Create unit tests for UinputOutput covering device creation, event injection, and cleanup following requirements 2.1-2.5 | Restrictions: Device creation tests may require elevated permissions (mark #[ignore] for CI), test inject_event formats events correctly, test cleanup removes device from /dev/input/, use temporary device names for isolation, add integration test with real injection (marked #[ignore]) | Success: Creation tests verify device appears in /dev/input/, injection tests verify events are formatted correctly, cleanup tests verify device removal, ignored tests document permission requirements_

## Phase 3: Device Discovery and Matching

- [ ] 10. Implement device enumeration
  - File: `keyrx_daemon/src/device_manager.rs` (NEW)
  - Implement `enumerate_keyboards() -> Result<Vec<evdev::Device>, DeviceError>`
  - Scan /dev/input/event* devices
  - Filter for keyboard devices (has EV_KEY with alphanumeric keys)
  - Exclude non-keyboard devices (mice, touchpads)
  - Purpose: Discover available keyboard devices
  - _Leverage: evdev crate device enumeration_
  - _Requirements: 1.1_
  - _Prompt: Role: Rust Developer with expertise in Linux device discovery | Task: Implement keyboard device enumeration by scanning /dev/input/ and filtering for keyboard capabilities following requirement 1.1 | Restrictions: Must iterate /dev/input/event* files, use evdev::Device::open for each, check supported_events for EV_KEY, filter for devices with alphabetic keys (KEY_A through KEY_Z), skip devices without keyboard keys (mice, etc.), return Vec of keyboard devices, handle permission errors gracefully (log and skip), add doc comments | Success: enumerate_keyboards finds all connected keyboards, filters out mice and other non-keyboard devices, handles permission errors by skipping (with log), returns empty Vec if no keyboards found, works with multiple keyboards_

- [ ] 11. Implement pattern matching for devices
  - File: `keyrx_daemon/src/device_manager.rs` (continue)
  - Implement `match_device(device: &evdev::Device, pattern: &str) -> bool`
  - Support wildcard pattern `"*"` (matches all)
  - Support prefix patterns (e.g., `"USB\\VID_04D9*"`)
  - Match against device name and serial number
  - Purpose: Select devices based on configuration patterns
  - _Leverage: keyrx_core::config::DeviceIdentifier pattern field_
  - _Requirements: 3.1, 3.3_
  - _Prompt: Role: Rust Developer with expertise in pattern matching and string handling | Task: Implement device pattern matching supporting wildcards and prefix patterns, matching against device name and serial following requirements 3.1 and 3.3 | Restrictions: Wildcard "*" must match all devices, prefix patterns (ending with *) match device name or serial starting with prefix, exact patterns require exact match, case-insensitive matching for robustness, use glob crate if needed for complex patterns, add doc comments with pattern examples | Success: match_device("*") returns true for all devices, match_device("USB*") matches devices with USB in name or serial, exact match works for specific devices, case-insensitive matching works, returns false for non-matching devices_

- [ ] 12. Implement DeviceManager struct
  - File: `keyrx_daemon/src/device_manager.rs` (continue)
  - Define `DeviceManager` struct managing multiple devices
  - Define `ManagedDevice` struct bundling device with config
  - Implement `discover(configs: &[DeviceConfig]) -> Result<Self, DiscoveryError>`
  - Match devices to configs using pattern matching
  - Purpose: Orchestrate multi-device management
  - _Leverage: EvdevInput, pattern matching, DeviceConfig_
  - _Requirements: 3.1, 3.2, 3.4_
  - _Prompt: Role: Rust Software Architect with expertise in resource management | Task: Implement DeviceManager that discovers devices and matches them to configurations, creating ManagedDevice instances following requirements 3.1, 3.2, and 3.4 | Restrictions: Must enumerate keyboards, match each against config patterns in priority order (specific before wildcard), create ManagedDevice with EvdevInput, DeviceState, and KeyLookup, handle no-match case (skip device, don't grab), log matched devices at INFO level, log unmatched devices at DEBUG level, return error if no devices match any config | Success: DeviceManager::discover creates ManagedDevice for each matched device, unmatched devices are not grabbed, multiple devices can match same pattern, priority order respected (first matching config wins), empty device list returns error with available devices listed_

- [ ] 13. Add device hot-plug support
  - File: `keyrx_daemon/src/device_manager.rs` (continue)
  - Implement `refresh(&mut self) -> Result<(), DiscoveryError>`
  - Detect newly connected devices
  - Detect disconnected devices and clean up
  - Purpose: Handle USB keyboard connect/disconnect during operation
  - _Leverage: enumerate_keyboards, pattern matching_
  - _Requirements: 1.5, 7.3_
  - _Prompt: Role: Rust Developer with expertise in dynamic device management | Task: Implement device hot-plug support in DeviceManager with refresh method to detect new and removed devices following requirements 1.5 and 7.3 | Restrictions: Must compare current device list with previous, add new matching devices (grab and create ManagedDevice), remove disconnected devices (release and cleanup), log device changes at INFO level, handle errors gracefully (continue with remaining devices), call refresh periodically or on inotify event | Success: refresh() detects new keyboard when plugged in, refresh() removes device when unplugged, state is preserved for remaining devices, new devices are grabbed correctly, compiles and handles USB keyboard plug/unplug_

- [ ] 14. Write DeviceManager unit tests
  - File: `keyrx_daemon/src/device_manager.rs` (add tests module)
  - Test pattern matching with various patterns
  - Test device enumeration filtering
  - Test discovery with mock devices (if possible)
  - Purpose: Verify device management logic
  - _Leverage: Mock data for pattern testing_
  - _Requirements: 3.1-3.4_
  - _Prompt: Role: QA Engineer with focus on device management testing | Task: Create unit tests for DeviceManager covering pattern matching, enumeration, and discovery following requirements 3.1-3.4 | Restrictions: Pattern matching tests are pure functions (no real devices needed), test wildcard, prefix, and exact patterns, test case insensitivity, device enumeration tests may need real devices (mark #[ignore]), test discovery logic with mock device list if possible, verify priority ordering | Success: Pattern matching tests pass for all pattern types, enumeration tests verify keyboard filtering, discovery tests verify config matching, ignored tests document real-device requirements_

## Phase 4: Daemon Lifecycle Management

- [ ] 15. Implement signal handling
  - File: `keyrx_daemon/src/daemon.rs` (NEW)
  - Implement `install_signal_handlers(running: Arc<AtomicBool>) -> Result<(), io::Error>`
  - Handle SIGTERM and SIGINT for graceful shutdown
  - Handle SIGHUP for configuration reload
  - Purpose: Enable clean daemon termination and reload
  - _Leverage: signal-hook crate_
  - _Requirements: 4.2, 4.3_
  - _Prompt: Role: Rust Developer with expertise in Unix signal handling | Task: Implement signal handlers for SIGTERM, SIGINT, and SIGHUP using signal-hook crate, coordinating with daemon running flag following requirements 4.2 and 4.3 | Restrictions: Must use signal_hook::flag::register for SIGTERM and SIGINT to set running=false, use signal_hook::iterator::Signals for SIGHUP to trigger reload, running must be Arc<AtomicBool> for thread safety, add doc comments explaining signal behavior, handle registration errors gracefully | Success: SIGTERM sets running flag to false, SIGINT sets running flag to false, SIGHUP can be detected for reload trigger, signal handlers don't interfere with normal operation, compiles and handles Ctrl+C correctly_

- [ ] 16. Implement Daemon struct and constructor
  - File: `keyrx_daemon/src/daemon.rs` (continue)
  - Define `Daemon` struct with config_path, device_manager, output, running flag
  - Implement `new(config_path: &Path) -> Result<Self, DaemonError>`
  - Load configuration, discover devices, create uinput output
  - Install signal handlers
  - Purpose: Initialize daemon with all components
  - _Leverage: load_config, DeviceManager, UinputOutput, signal handlers_
  - _Requirements: 4.1_
  - _Prompt: Role: Rust Software Architect with expertise in daemon design | Task: Implement Daemon struct that initializes all components (config, device manager, uinput output, signals) in correct order following requirement 4.1 | Restrictions: Must load config first, then discover devices, then create uinput output, install signal handlers early, log startup progress at INFO level, handle errors with DaemonError wrapping, store all components in struct for lifecycle management, add doc comments | Success: Daemon::new initializes all components, logs startup progress, returns DaemonError on any failure, stores all components for later use, signal handlers installed and functional_

- [ ] 17. Implement Daemon event loop
  - File: `keyrx_daemon/src/daemon.rs` (continue)
  - Implement `run(&mut self) -> Result<(), DaemonError>`
  - Poll all input devices for events
  - Process events through respective DeviceState/KeyLookup
  - Inject output events via shared UinputOutput
  - Check shutdown flag between iterations
  - Purpose: Main event processing loop
  - _Leverage: EventProcessor pattern from Phase 2, but adapted for multi-device_
  - _Requirements: 4.1, 4.9, 7.1_
  - _Prompt: Role: Rust Developer with expertise in event-driven systems and polling | Task: Implement Daemon run method with multi-device event loop, processing events from all devices and outputting via shared uinput following requirements 4.1, 4.9, and 7.1 | Restrictions: Must iterate all managed devices, use non-blocking or select/poll for fair handling, process each event through process_event, inject output via shared UinputOutput, check running flag between iterations, handle device errors gracefully (log and continue or remove device), log at DEBUG level for each event, measure and log latency periodically | Success: run() processes events from all devices, events are remapped correctly, output appears in applications, Ctrl+C stops the loop gracefully, device errors don't crash daemon, latency stays under 1ms_

- [ ] 18. Implement configuration reload
  - File: `keyrx_daemon/src/daemon.rs` (continue)
  - Implement `reload(&mut self) -> Result<(), DaemonError>`
  - Reload .krx file
  - Rebuild lookup tables for all devices
  - Keep device grabs active (no re-grab)
  - Purpose: Apply new configuration without restart
  - _Leverage: load_config, KeyLookup::from_device_config_
  - _Requirements: 4.3_
  - _Prompt: Role: Rust Developer with expertise in hot-reload systems | Task: Implement Daemon reload method that reloads configuration and rebuilds lookup tables without interrupting event processing following requirement 4.3 | Restrictions: Must reload .krx file from disk, rebuild KeyLookup for each device, preserve DeviceState (don't reset modifier/lock state), don't release/re-grab devices, log reload at INFO level, handle errors gracefully (keep old config on failure), add doc comments | Success: reload() loads new config, lookup tables are updated, modifier state preserved, devices stay grabbed, reload errors don't crash daemon, old config retained on failure_

- [ ] 19. Implement graceful shutdown
  - File: `keyrx_daemon/src/daemon.rs` (continue)
  - Implement `shutdown(&mut self)`
  - Release all grabbed devices
  - Destroy uinput output device
  - Log shutdown completion
  - Purpose: Clean daemon termination
  - _Leverage: EvdevInput::release, UinputOutput::destroy_
  - _Requirements: 4.2_
  - _Prompt: Role: Rust Developer with expertise in resource cleanup | Task: Implement Daemon shutdown method that releases all resources in correct order following requirement 4.2 | Restrictions: Must release all grabbed devices first (restore normal input), destroy uinput device second, log each step at INFO level, handle errors gracefully (log warning but continue cleanup), ensure no orphaned resources, call from Drop trait for automatic cleanup | Success: shutdown() releases all devices, destroys uinput device, logs completion, no orphaned devices/resources, handles errors gracefully, Drop ensures cleanup on panic_

- [ ] 20. Write Daemon integration tests
  - File: `keyrx_daemon/tests/daemon_tests.rs` (NEW)
  - Test daemon initialization and shutdown
  - Test signal handling (SIGTERM, SIGINT)
  - Test configuration reload
  - Purpose: Verify daemon lifecycle management
  - _Leverage: Mock devices where possible_
  - _Requirements: 4.1-4.4_
  - _Prompt: Role: Integration Test Engineer with daemon testing expertise | Task: Create integration tests for Daemon lifecycle including initialization, shutdown, signals, and reload following requirements 4.1-4.4 | Restrictions: Use mock devices for most tests (avoid real hardware dependency), test signal handling with signal::kill, test reload by modifying config file, verify cleanup on shutdown (no orphaned resources), mark real-device tests #[ignore], use timeout for daemon operations | Success: Initialization test verifies all components created, shutdown test verifies all resources released, signal test verifies graceful stop on SIGTERM, reload test verifies new config applied, tests pass in CI without real devices_

## Phase 5: CLI and systemd Integration

- [ ] 21. Implement CLI with clap
  - File: `keyrx_daemon/src/main.rs` (MODIFY)
  - Define CLI with subcommands: run, list-devices, validate
  - `run --config <path> [--debug]`: Start daemon
  - `list-devices`: List available input devices
  - `validate --config <path>`: Validate config and device matching
  - Purpose: User-friendly daemon interface
  - _Leverage: clap derive API, matching keyrx_compiler pattern_
  - _Requirements: 4.1, 4.5, Usability_
  - _Prompt: Role: Rust Developer with expertise in CLI design and clap | Task: Implement CLI for keyrx_daemon with run, list-devices, and validate subcommands using clap derive API following requirements 4.1, 4.5, and Usability section | Restrictions: Must use #[derive(Parser)] and #[derive(Subcommand)], run subcommand takes --config path and optional --debug flag, list-devices takes no arguments, validate takes --config path, add help text for all commands, set appropriate default values, follow keyrx_compiler CLI pattern for consistency | Success: CLI parses all subcommands correctly, help text is informative, run starts daemon, list-devices shows available keyboards, validate checks config without grabbing, exit codes are appropriate (0=success, 1=config error, 2=permission error)_

- [ ] 22. Implement list-devices subcommand
  - File: `keyrx_daemon/src/main.rs` (continue)
  - Enumerate all input devices
  - Print device name, path, serial number
  - Identify keyboard vs non-keyboard
  - Purpose: Help users configure device patterns
  - _Leverage: enumerate_keyboards from device_manager_
  - _Requirements: Usability: Device Listing_
  - _Prompt: Role: Rust Developer with expertise in user-facing output | Task: Implement list-devices subcommand that enumerates and displays input devices with useful information following Usability: Device Listing requirement | Restrictions: Must list all /dev/input/event* devices, indicate which are keyboards, show name, path, and serial (if available), format output clearly with columns or indentation, handle permission errors gracefully (show which devices couldn't be read), exit code 0 on success | Success: list-devices shows all input devices, keyboards are clearly marked, serial numbers displayed when available, permission denied devices noted, output is clear and helpful for configuration_

- [ ] 23. Implement validate subcommand
  - File: `keyrx_daemon/src/main.rs` (continue)
  - Load and validate .krx configuration
  - Enumerate devices and show matches
  - Don't actually grab devices
  - Purpose: Dry-run for configuration verification
  - _Leverage: load_config, DeviceManager pattern matching_
  - _Requirements: Usability: Dry-Run Mode_
  - _Prompt: Role: Rust Developer with expertise in validation and diagnostics | Task: Implement validate subcommand that loads config, enumerates devices, shows matches without grabbing following Usability: Dry-Run Mode requirement | Restrictions: Must load and validate .krx file, enumerate keyboards, run pattern matching, print which devices match which configs, don't call grab() (dry-run only), show warnings for unmatched devices, exit code 0 if config valid and devices match, exit code 1 if config invalid or no matches | Success: validate loads config successfully, shows matched devices, shows unmatched devices as warnings, doesn't grab devices, exit codes are appropriate, helpful for debugging configuration_

- [ ] 24. Create udev rules file
  - File: `keyrx_daemon/udev/99-keyrx.rules` (NEW)
  - Grant input group access to /dev/input/event*
  - Grant uinput group access to /dev/uinput
  - Include installation instructions in comments
  - Purpose: Enable non-root daemon operation
  - _Leverage: Standard udev rule format_
  - _Requirements: 5.1, 5.2, 5.4_
  - _Prompt: Role: Linux System Administrator with udev expertise | Task: Create udev rules file granting necessary permissions for keyrx daemon to operate without root following requirements 5.1, 5.2, and 5.4 | Restrictions: Must grant input group read access to /dev/input/event*, grant uinput group read/write access to /dev/uinput, create uinput device node if not exists, include header comments with installation instructions (copy to /etc/udev/rules.d/, reload with udevadm), test on common distributions | Success: Rules file has correct syntax, grants appropriate permissions, includes installation instructions, works on Ubuntu/Fedora/Arch, non-root user in correct groups can run daemon_

- [ ] 25. Create systemd service file
  - File: `keyrx_daemon/systemd/keyrx.service` (NEW)
  - Type=simple for foreground daemon
  - ExecStart with config path
  - Restart=on-failure for auto-recovery
  - Documentation and description
  - Purpose: Enable systemd service management
  - _Leverage: Standard systemd service format_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  - _Prompt: Role: Linux System Administrator with systemd expertise | Task: Create systemd service file for keyrx daemon with proper configuration for reliable service operation following requirements 6.1-6.5 | Restrictions: Must set Type=simple (daemon runs in foreground), set User/Group for non-root operation, set ExecStart with --config path (use Environment or EnvironmentFile for config path), set Restart=on-failure with RestartSec=1, set ExecReload for SIGHUP, add Documentation and Description, include installation instructions in comments | Success: Service file has correct syntax, starts daemon with config, restarts on failure, reloads on systemctl reload, logs to journald, can be enabled/started/stopped via systemctl_

- [ ] 26. Write CLI and service tests
  - File: `keyrx_daemon/tests/cli_tests.rs` (NEW)
  - Test CLI argument parsing
  - Test list-devices output format
  - Test validate subcommand
  - Test help text
  - Purpose: Verify CLI functionality
  - _Leverage: assert_cmd crate for CLI testing_
  - _Requirements: 4.1, 4.5, Usability_
  - _Prompt: Role: QA Engineer with CLI testing expertise | Task: Create CLI tests for keyrx_daemon covering argument parsing, subcommands, and help text following requirements 4.1, 4.5, and Usability section | Restrictions: Must use assert_cmd for CLI testing, test run --help, list-devices, validate with valid and invalid configs, verify exit codes, verify output format contains expected strings, mark real-device tests #[ignore], test error messages are helpful | Success: CLI parsing tests pass, help text tests pass, list-devices outputs correctly, validate tests pass, exit codes are correct, tests run in CI without real devices_

## Phase 6: Integration Testing and Documentation

- [ ] 27. Create end-to-end integration tests
  - File: `keyrx_daemon/tests/e2e_tests.rs` (NEW)
  - Test CapsLock→Escape remapping with real device
  - Test Vim navigation layer with real device
  - Test multi-device configuration
  - Purpose: Validate complete system behavior
  - _Leverage: Real devices, evtest for verification_
  - _Requirements: Testing Strategy: End-to-End Testing_
  - _Prompt: Role: E2E Test Engineer with Linux input testing expertise | Task: Create end-to-end integration tests using real input devices (marked #[ignore] for CI) covering common use cases following Testing Strategy: End-to-End Testing | Restrictions: All tests require real devices (mark #[ignore] and document requirements), test basic remapping (A→B), test CapsLock→Escape, test Vim navigation layer (CapsLock+HJKL), test multi-device (if multiple keyboards available), use subprocess to run daemon and inject test events, verify output with evtest or xdotool, add detailed test documentation | Success: E2E tests verify real remapping works, tests document hardware requirements, tests can be run manually on development machine, test results are reliable and repeatable_

- [ ] 28. Update CHANGELOG and documentation
  - Files: `CHANGELOG.md`, `README.md`, `docs/LINUX_SETUP.md` (NEW)
  - Document Linux platform support
  - Write installation guide with udev rules
  - Write systemd service setup guide
  - Document troubleshooting steps
  - Purpose: Enable users to set up keyrx on Linux
  - _Leverage: Existing documentation structure_
  - _Requirements: Documentation_
  - _Prompt: Role: Technical Writer with Linux expertise | Task: Update documentation for Linux platform support including installation, configuration, and troubleshooting following Documentation requirements | Restrictions: Update CHANGELOG.md with Linux evdev integration features, update README.md with Linux quickstart, create docs/LINUX_SETUP.md with detailed setup instructions (udev rules, systemd service, permissions), include troubleshooting section for common issues (permission denied, no devices found), test instructions on fresh Ubuntu/Fedora install | Success: CHANGELOG documents all new features, README has Linux quickstart, LINUX_SETUP.md has complete setup guide, troubleshooting covers common issues, documentation is accurate and tested_

- [ ] 29. Final integration and verification
  - Files: All modified files
  - Run full test suite on Linux
  - Verify performance benchmarks
  - Test on multiple Linux distributions
  - Fix any remaining issues
  - Purpose: Ensure production readiness
  - _Leverage: CI/CD, manual testing_
  - _Requirements: All_
  - _Prompt: Role: Senior Rust Developer with Linux deployment expertise | Task: Complete final integration and verification of Linux platform support, running all tests and verifying on multiple distributions following all requirements | Restrictions: Must run cargo test --workspace on Linux, run cargo clippy with -D warnings, verify benchmarks meet performance targets (<1ms latency), test on Ubuntu 22.04 and Fedora 39 at minimum, fix any platform-specific issues, verify udev rules work, verify systemd service starts correctly, update any failing tests | Success: All tests pass on Linux, clippy has zero warnings, performance meets targets, works on Ubuntu and Fedora, udev rules grant correct permissions, systemd service runs correctly, end-to-end remapping works on real hardware_
