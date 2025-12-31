# Tasks: Windows Platform Support

**Spec Name**: windows-platform-support
**Created**: 2024-12-24
**Status**: Completed
**Version**: 1.0.0

---

- [x] 1. Create Virtual Key Mapping Module
  - File: keyrx_daemon/src/platform/windows/keycode.rs (NEW)
  - File: keyrx_daemon/src/platform/windows/mod.rs (MODIFIED)
  - Create bidirectional mapping between Windows Virtual Key codes and KeyRx KeyCode enum
  - Implement compile-time constant arrays for O(1) lookup performance
  - Support all standard keys (letters, numbers, modifiers, function keys, arrows, special keys)
  - _Leverage: keyrx_core::config::KeyCode enum, windows::Win32::UI::Input::KeyboardAndMouse::* constants, phf crate_
  - _Requirements: US-2.1, US-2.2_
  - _Prompt: Role: Rust systems programmer with Win32 API expertise | Task: Create bidirectional mapping between Windows Virtual Key codes (VK_*) and KeyRx KeyCode enum following requirements US-2.1 and US-2.2, implementing compile-time constant arrays for O(1) lookup performance, leveraging keyrx_core::config::KeyCode enum and windows API constants | Restrictions: Use const arrays (no HashMap or runtime initialization), no heap allocation, handle unmapped VK codes gracefully (return None, don't panic), follow existing codebase naming conventions (snake_case for functions) | Success: All standard VK codes mapped to KeyCode, vk_to_keycode(0x41) returns Some(KeyCode::A), keycode_to_vk(KeyCode::A) returns Some(0x41), unmapped codes return None, roundtrip verification passes for all valid keys, unit tests written and passing_

- [x] 2. Implement Windows Keyboard Hook
  - File: keyrx_daemon/src/platform/windows/hook.rs (NEW)
  - File: keyrx_daemon/src/platform/windows/mod.rs (MODIFIED)
  - Install Windows low-level keyboard hook (WH_KEYBOARD_LL) using SetWindowsHookEx API
  - Route events to processing thread via lock-free channel
  - Ensure hook callback returns <50μs to avoid input lag
  - Implement RAII cleanup with Drop trait
  - _Leverage: windows::Win32::UI::WindowsAndMessaging::*, crossbeam_channel::Sender, keyrx_core::runtime::event::EventType_
  - _Requirements: US-1.1, US-1.2_
  - _Prompt: Role: Rust systems programmer with Win32 API and unsafe code expertise | Task: Implement Windows low-level keyboard hook (WH_KEYBOARD_LL) using SetWindowsHookEx API following requirements US-1.1 and US-1.2, installing hook on main thread and routing events to processing thread via lock-free channel, leveraging windows API and crossbeam_channel | Restrictions: No heap allocation in callback (use try_send, not send), callback must be thread-safe (uses thread-local storage), must clean up hook even on panic (Drop trait), follow RAII pattern (hook lifetime tied to struct), ensure callback returns <50μs | Success: SetWindowsHookEx returns valid handle, keyboard events routed to callback, injected events ignored (no infinite loop), hook uninstalled on drop, no crashes, no resource leaks, tests verify hook lifecycle_

- [x] 3. Implement Event Injection
  - File: keyrx_daemon/src/platform/windows/inject.rs (NEW)
  - Implement keyboard event injection using SendInput API
  - Convert KeyRx KeyCode to Windows Virtual Key codes
  - Handle modifiers (Shift, Ctrl, Alt, Win) with proper ordering
  - _Leverage: windows::Win32::UI::Input::KeyboardAndMouse::SendInput, Task 1 keycode_to_vk() function, keyrx_core::runtime::event::EventType_
  - _Requirements: US-3.1, US-3.2_
  - _Prompt: Role: Rust systems programmer with Win32 input API expertise | Task: Implement keyboard event injection using SendInput API following requirements US-3.1 and US-3.2, converting KeyRx KeyCode to Windows Virtual Key codes and handling modifiers with proper ordering, leveraging SendInput and keycode_to_vk() from Task 1 | Restrictions: Do not inject events marked as KEYEVENTF_UNICODE (scan codes only), mark injected events appropriately to avoid hook re-processing, handle extended keys (arrows, etc.) with KEYEVENTF_EXTENDEDKEY flag, inject modifiers before main key in correct order | Success: inject_key_event(KeyCode::A, Press, ...) sends VK_A press, modifiers injected before main key, release events inject KEYEVENTF_KEYUP flag, SendInput returns non-zero (success), modifier ordering test passes (Shift+A injects Shift down, A down, A up, Shift up)_

- [x] 4. Implement InputDevice Trait for Windows
  - File: keyrx_daemon/src/platform/windows/input.rs (NEW)
  - File: keyrx_daemon/src/platform/mod.rs (MODIFIED)
  - Implement InputDevice trait integrating keyboard hook with platform-agnostic event interface
  - Convert RawKeyEvent → KeyEvent (platform-neutral)
  - _Leverage: Task 1 vk_to_keycode() function, Task 2 WindowsKeyboardHook, keyrx_daemon/src/platform/mod.rs InputDevice trait_
  - _Requirements: US-5.1_
  - _Prompt: Role: Rust platform abstraction developer | Task: Implement InputDevice trait for Windows following requirement US-5.1, integrating keyboard hook with platform-agnostic event interface and converting RawKeyEvent to KeyEvent, leveraging vk_to_keycode() from Task 1 and WindowsKeyboardHook from Task 2 | Restrictions: Do not create new hook inside next_event() (reuse existing), handle EndOfStream when channel closed, map unmapped VK codes to DeviceError::UnmappedKey, no-op for grab() and release() (Windows hook is implicitly exclusive) | Success: WindowsKeyboardInput implements InputDevice trait, next_event() returns KeyEvent when key pressed, unmapped VK codes return error (not panic), channel close returns EndOfStream, integration test passes_

- [x] 5. Implement OutputDevice Trait for Windows
  - File: keyrx_daemon/src/platform/windows/output.rs (NEW)
  - File: keyrx_daemon/src/platform/mod.rs (MODIFIED)
  - Implement OutputDevice trait wrapping EventInjector in platform-agnostic interface
  - _Leverage: Task 3 EventInjector, keyrx_daemon/src/platform/mod.rs OutputDevice trait_
  - _Requirements: US-5.1_
  - _Prompt: Role: Rust platform abstraction developer | Task: Implement OutputDevice trait for Windows following requirement US-5.1, wrapping EventInjector in platform-agnostic interface, leveraging EventInjector from Task 3 and OutputDevice trait from platform/mod.rs | Restrictions: Error conversion from InjectionError to DeviceError, no state modification (stateless output), simple wrapper with proper error mapping | Success: WindowsKeyboardOutput implements OutputDevice, send_event() successfully injects events, errors mapped to DeviceError variants, test verifies SendInput called correctly_

- [x] 6. Implement System Tray Icon
  - File: keyrx_daemon/src/windows/tray.rs (NEW)
  - File: Cargo.toml (MODIFIED - add tray-icon dependency)
  - Implement Windows system tray icon using tray-icon crate
  - Provide menu for daemon control (Reload Config, Exit)
  - _Leverage: tray-icon crate documentation, crossbeam_channel for event delivery_
  - _Requirements: US-4.1, US-4.2_
  - _Prompt: Role: Rust GUI developer with tray icon experience | Task: Implement Windows system tray icon using tray-icon crate following requirements US-4.1 and US-4.2, providing menu for daemon control (Reload Config, Exit), leveraging tray-icon crate and crossbeam_channel | Restrictions: Use default icon if custom icon not available, handle menu creation errors gracefully, non-blocking event poll (use try_recv), add dependency to Cargo.toml under [target.'cfg(windows)'.dependencies] | Success: Tray icon appears in Windows notification area, right-click shows menu with Reload Config and Exit, clicking menu items triggers TrayMenuEvent, tooltip shows KeyRx Daemon, manual test confirms visibility and functionality_

- [x] 7. Integrate Components in Main Function
  - File: keyrx_daemon/src/main.rs (MODIFIED)
  - File: keyrx_daemon/src/platform/mod.rs (MODIFIED)
  - Integrate all Windows components in main() function
  - Setup event loop with hook, processor thread, tray icon, and graceful shutdown
  - Implement Windows message loop on main thread
  - _Leverage: Tasks 2, 4, 5, 6 (all components), windows::Win32::UI::WindowsAndMessaging::*_
  - _Requirements: US-1.1, US-6.1_
  - _Prompt: Role: Rust systems integration developer | Task: Integrate all Windows components in main() function following requirements US-1.1 and US-6.1, setting up event loop with hook, processor thread, tray icon, and graceful shutdown, implementing Windows message loop on main thread, leveraging WindowsKeyboardHook, WindowsKeyboardInput, WindowsKeyboardOutput, TrayIconController from previous tasks | Restrictions: Message loop must run on main thread (Windows requirement), event processing must run on separate thread (avoid blocking hook), clean shutdown on Ctrl+C (install signal handler), load config from .krx file path | Success: Running keyrx_daemon.exe run --config test.krx starts daemon, hook installed and events processed, remapping works end-to-end, tray icon appears and responds to clicks, Ctrl+C triggers clean shutdown, no resource leaks on exit_

- [x] 8. Write Comprehensive Tests
  - File: keyrx_daemon/src/platform/windows/tests.rs (NEW)
  - File: keyrx_daemon/tests/windows_integration.rs (NEW)
  - Write comprehensive unit and integration tests for Windows platform
  - Achieve ≥95% code coverage for platform/windows/*
  - Use property-based testing for VK mapping invariants
  - _Leverage: proptest crate for property-based tests, existing Linux tests as reference (keyrx_daemon/tests/)_
  - _Requirements: US-7.1_
  - _Prompt: Role: Rust test engineer with property-based testing expertise | Task: Write comprehensive unit and integration tests for Windows platform following requirement US-7.1, achieving ≥95% code coverage for platform/windows/* and using property-based testing for VK mapping invariants, leveraging proptest crate and existing Linux tests as reference | Restrictions: Tests must run in CI (Windows runner required), avoid tests requiring user interaction (automated only), mock what cannot be tested directly (SendInput result), test all VK mappings with roundtrip verification, test hook lifecycle, test event injection | Success: cargo test --target x86_64-pc-windows-msvc passes all tests, code coverage ≥95% for platform/windows/*, all VK codes tested, all edge cases covered, property tests verify roundtrip invariants_

- [x] 9. Update Documentation
  - File: README.md (MODIFIED)
  - File: docs/user-guide/windows-setup.md (NEW)
  - File: CHANGELOG.md (MODIFIED)
  - Update documentation to reflect Windows platform support
  - Create Windows setup guide with installation and troubleshooting
  - _Leverage: docs/user-guide/linux-setup.md as template, docs/user-guide/dsl-manual.md for config examples_
  - _Requirements: NFR-2_
  - _Prompt: Role: Technical documentation writer | Task: Update documentation to reflect Windows platform support following requirement NFR-2, creating Windows setup guide with installation and troubleshooting, updating README and CHANGELOG, leveraging linux-setup.md as template and dsl-manual.md for examples | Restrictions: Follow existing documentation style, include code examples for common tasks, add FAQ section for Windows-specific issues, change Windows badge from Planned to Supported | Success: README accurately reflects Windows support, Windows setup guide is complete and tested, CHANGELOG lists all v0.2.0 changes, documentation reviewed for accuracy_

- [x] 10. Verify CI/CD for Windows Builds
  - File: .github/workflows/ci.yml (VERIFY)
  - File: .github/workflows/release.yml (VERIFY)
  - Verify GitHub Actions workflows build Windows binaries correctly
  - Ensure CI runs tests on Windows runner
  - _Leverage: Existing CI/CD workflows (already set up for Linux)_
  - _Requirements: US-7.2_
  - _Prompt: Role: DevOps engineer with GitHub Actions expertise | Task: Verify GitHub Actions workflows build Windows binaries correctly following requirement US-7.2, ensuring CI runs tests on Windows runner, leveraging existing CI/CD workflows | Restrictions: Do not modify workflow if already correct, only update if Windows support missing, verify matrix includes windows-latest, verify build command includes --features windows, verify tests run on Windows | Success: CI runs on Windows runner, tests pass on Windows, release creates keyrx_daemon-windows.exe artifact, artifact downloadable from GitHub releases, CI build triggered by test commit succeeds_
