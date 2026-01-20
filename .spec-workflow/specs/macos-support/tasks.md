# Tasks Document

- [x] 1. Add macOS dependencies to Cargo.toml
  - File: keyrx_daemon/Cargo.toml
  - Add macOS target-specific dependencies: rdev, enigo, iokit-sys, objc2, accessibility-sys
  - Purpose: Enable macOS-specific crate dependencies
  - _Leverage: keyrx_daemon/Cargo.toml Windows target dependencies pattern_
  - _Requirements: REQ-7_
  - _Prompt: Role: Rust Developer | Task: Add macOS dependencies using `[target.'cfg(target_os = "macos")'.dependencies]` syntax with rdev = "0.5.3", enigo = "0.2", iokit-sys = "0.4", objc2 = "0.5", accessibility-sys = "0.1" | Restrictions: Do not modify Linux/Windows dependencies, ensure Rust 1.70+ compatibility | Success: Cargo build succeeds on macOS target without dependency conflicts_

- [x] 2. Create macOS platform module structure
  - Files: keyrx_daemon/src/platform/macos/mod.rs, input_capture.rs, output_injection.rs, device_discovery.rs, keycode_map.rs, tray.rs, permissions.rs
  - Create directory with skeleton module files and MacosPlatform struct
  - Purpose: Establish module organization for macOS platform code
  - _Leverage: keyrx_daemon/src/platform/windows/ directory structure_
  - _Requirements: REQ-1, REQ-2, REQ-3, REQ-4, REQ-5_
  - _Prompt: Role: Software Architect | Task: Create macos/ directory with 7 skeleton files (mod.rs with MacosPlatform struct, input_capture.rs, output_injection.rs, device_discovery.rs, keycode_map.rs, tray.rs, permissions.rs), each with module docs | Restrictions: Follow snake_case naming, skeleton only (no implementation), must compile | Success: All files compile, structure matches Windows pattern_

- [x] 3. Implement Platform trait for MacosPlatform
  - File: keyrx_daemon/src/platform/macos/mod.rs
  - Implement all Platform trait methods with delegation to sub-components
  - Purpose: Provide main platform abstraction implementation
  - _Leverage: keyrx_daemon/src/platform/mod.rs Platform trait, keyrx_daemon/src/platform/windows/mod.rs pattern_
  - _Requirements: REQ-1, REQ-2, REQ-3, REQ-4_
  - _Prompt: Role: Platform Engineer | Task: Implement Platform trait methods (initialize with permission check, capture_input, inject_output, list_devices, shutdown) delegating to MacosInputCapture, MacosOutputInjector, device_discovery | Restrictions: Do not modify trait definition, use PlatformError enum | Success: Platform trait fully implemented, compiles without errors_

- [x] 4. Implement Accessibility permission checker
  - File: keyrx_daemon/src/platform/macos/permissions.rs
  - Add check_accessibility_permission() and get_permission_error_message()
  - Purpose: Detect and report Accessibility permission status
  - _Leverage: accessibility-sys::accessibility::AXIsProcessTrusted_
  - _Requirements: REQ-5_
  - _Prompt: Role: macOS Developer | Task: Create check_accessibility_permission() using unsafe AXIsProcessTrusted() and get_permission_error_message() with step-by-step setup instructions | Restrictions: Minimal unsafe code, user-friendly error message | Success: Permission check works, error message actionable_

- [x] 5. Implement CGKeyCode mapping
  - File: keyrx_daemon/src/platform/macos/keycode_map.rs
  - Create bidirectional CGKeyCode ↔ KeyCode mapping and rdev/enigo conversions
  - Purpose: Enable keycode translation for input/output
  - _Leverage: keyrx_daemon/src/platform/windows/keycode.rs pattern_
  - _Requirements: REQ-1, REQ-2_
  - _Prompt: Role: Systems Developer | Task: Implement cgkeycode_to_keyrx(), keyrx_to_cgkeycode(), rdev_key_to_keyrx(), keyrx_to_enigo_key() covering 100+ keys with comprehensive tests | Restrictions: Maintain bidirectional consistency, handle unmapped keys gracefully | Success: All keycodes mapped, bidirectional property verified, tests pass_

- [x] 6. Implement input capture
  - File: keyrx_daemon/src/platform/macos/input_capture.rs
  - Create MacosInputCapture using rdev::listen with channel architecture
  - Purpose: Capture keyboard events using safe Rust wrapper
  - _Leverage: rdev::listen, keycode_map::rdev_key_to_keyrx_
  - _Requirements: REQ-1_
  - _Prompt: Role: Concurrent Systems Developer | Task: Implement MacosInputCapture with new() spawning rdev::listen thread and next_event() blocking on channel recv | Restrictions: Safe Rust only, thread-safe channels, <1ms latency | Success: Events captured correctly, thread-safe, latency <1ms_

- [x] 7. Implement output injection
  - File: keyrx_daemon/src/platform/macos/output_injection.rs
  - Create MacosOutputInjector using enigo::Enigo
  - Purpose: Inject remapped keyboard events
  - _Leverage: enigo::Enigo, keycode_map::keyrx_to_enigo_key_
  - _Requirements: REQ-2_
  - _Prompt: Role: Systems Developer | Task: Implement MacosOutputInjector with new() creating Enigo and inject() converting KeyEvent to enigo::Key | Restrictions: Safe Rust, proper error propagation, <1ms latency | Success: Injection works, error handling robust, latency <1ms_

- [x] 8. Implement IOKit device enumeration
  - File: keyrx_daemon/src/platform/macos/device_discovery.rs
  - Implement list_keyboard_devices() using iokit-sys with RAII wrappers
  - Purpose: Enumerate USB keyboards for device-specific configs
  - _Leverage: iokit-sys::*, keyrx_daemon/src/platform/common::DeviceInfo_
  - _Requirements: REQ-3_
  - _Prompt: Role: Low-level Systems Programmer | Task: Implement list_keyboard_devices() with IOKit APIs and RAII IOObjectGuard wrapper for automatic resource cleanup | Restrictions: Minimal unsafe code (<5%), document safety invariants, test for leaks | Success: Enumeration works, no leaks, VID/PID/serial extracted_

- [x] 9. Implement system tray
  - File: keyrx_daemon/src/platform/macos/tray.rs
  - Create MacosSystemTray using tray-icon crate
  - Purpose: Provide menu bar integration
  - _Leverage: tray-icon crate, keyrx_daemon/src/platform/windows/tray.rs_
  - _Requirements: REQ-4_
  - _Prompt: Role: macOS UI Developer | Task: Implement MacosSystemTray with menu items (Open Web UI, Reload Config, Exit) following Windows tray.rs pattern | Restrictions: Use existing tray-icon crate, follow macOS HIG | Success: Menu bar icon works, menu items functional_

- [x] 10. Update platform factory
  - File: keyrx_daemon/src/platform/mod.rs
  - Add macOS arm to create_platform() function
  - Purpose: Enable macOS platform selection
  - _Leverage: keyrx_daemon/src/platform/mod.rs lines 320-337_
  - _Requirements: REQ-1, REQ-2, REQ-3, REQ-4_
  - _Prompt: Role: Rust Developer | Task: Add `#[cfg(target_os = "macos")]` arm returning MacosPlatform::new() and add module declaration | Restrictions: Follow Linux/Windows pattern exactly | Success: macOS platform selected on macOS target, compiles_

- [x] 11. Add macOS CI/CD
  - File: .github/workflows/ci.yml
  - Add macos-latest to test matrix
  - Purpose: Enable automated testing
  - _Leverage: .github/workflows/ci.yml Ubuntu/Windows jobs_
  - _Requirements: REQ-7, REQ-8_
  - _Prompt: Role: DevOps Engineer | Task: Add macos-latest to os matrix with build and test steps, skip E2E tests requiring Accessibility | Restrictions: Follow existing pattern, use matrix strategy | Success: macOS CI runs, unit/integration tests pass_

- [x] 12. Create keycode mapping tests
  - File: keyrx_daemon/src/platform/macos/keycode_map.rs
  - Add comprehensive unit tests and property-based tests
  - Purpose: Ensure mapping correctness
  - _Leverage: proptest crate_
  - _Requirements: REQ-8_
  - _Prompt: Role: QA Engineer | Task: Add tests module with test_all_cgkeycodes_mapped, test_bidirectional_mapping (proptest), test_special_keys | Restrictions: ≥80% coverage, test success and failure cases | Success: All tests pass, bidirectional property verified_

- [x] 13. Create integration tests
  - File: keyrx_daemon/tests/macos_integration.rs
  - Test capture → inject flow with mocked components
  - Purpose: Ensure components work together
  - _Leverage: keyrx_daemon/tests/integration pattern_
  - _Requirements: REQ-8_
  - _Prompt: Role: Integration Test Engineer | Task: Create integration tests with mocked rdev/enigo/IOKit to avoid Accessibility requirement | Restrictions: No Accessibility dependency, test integration not implementation | Success: Tests pass in CI, component interactions verified_

- [x] 14. Create setup documentation
  - File: docs/setup/macos.md
  - Write setup guide with Accessibility permission instructions
  - Purpose: Help users set up keyrx on macOS
  - _Leverage: permissions.rs error message, docs/ structure_
  - _Requirements: REQ-5, REQ-7_
  - _Prompt: Role: Technical Writer | Task: Document installation, Accessibility permission grant (with screenshots), troubleshooting, building from source | Restrictions: Beginner-friendly, include screenshots or placeholders | Success: Clear setup instructions, comprehensive troubleshooting_

- [x] 15. Update README
  - File: README.md
  - Add macOS to supported platforms
  - Purpose: Inform users of macOS support
  - _Leverage: README.md existing structure_
  - _Requirements: REQ-7_
  - _Prompt: Role: Technical Writer | Task: Add macOS to supported platforms section and feature comparison table with links to setup docs | Restrictions: Maintain existing style, concise (link to details) | Success: macOS listed prominently, feature table updated_

- [x] 16. Add release workflow
  - File: .github/workflows/release.yml
  - Add macOS binary builds (x86_64 and ARM64)
  - Purpose: Enable automated releases
  - _Leverage: .github/workflows/release.yml existing jobs_
  - _Requirements: REQ-7_
  - _Prompt: Role: Release Engineer | Task: Add macOS build job for both Intel and ARM64 with optional code signing/notarization | Restrictions: Build both architectures, signing optional | Success: macOS binaries built and uploaded to releases_

- [x] 17. Create E2E test checklist
  - File: docs/testing/macos-e2e-checklist.md
  - Document manual testing procedures
  - Purpose: Ensure thorough pre-release testing
  - _Leverage: Design document E2E section_
  - _Requirements: REQ-8_
  - _Prompt: Role: QA Engineer | Task: Create checklist covering latency, memory, CPU, device enumeration, cross-device modifiers, permission flow | Restrictions: Include measurement methods and tools | Success: Comprehensive checklist, clear pass/fail criteria_

- [x] 18. Performance benchmarking
  - File: keyrx_daemon/benches/macos_latency.rs
  - Create latency benchmarks using criterion
  - Purpose: Verify <1ms latency requirement
  - _Leverage: criterion crate, existing benchmarks_
  - _Requirements: REQ-1, REQ-2, REQ-6_
  - _Prompt: Role: Performance Engineer | Task: Benchmark input capture, output injection, full pipeline latency with criterion | Restrictions: Run on real hardware, measure actual system latency | Success: p95 latency <1ms verified_

- [x] 19. Cross-platform verification
  - Files: All macOS platform files
  - Test identical .krx on Linux, Windows, macOS
  - Purpose: Ensure cross-platform compatibility
  - _Leverage: Existing test configs_
  - _Requirements: REQ-6_
  - _Prompt: Role: Senior QA Engineer | Task: Verify same .krx produces identical behavior on all platforms using deterministic testing | Restrictions: Test on real hardware, document any differences | Success: Identical behavior confirmed, limitations documented_

- [x] 20. Documentation review
  - Files: All documentation files
  - Review all docs for accuracy and consistency
  - Purpose: Provide high-quality documentation
  - _Leverage: docs/ structure_
  - _Requirements: All_
  - _Prompt: Role: Senior Technical Writer | Task: Review all docs for accuracy, completeness, consistency, clarity, working links | Restrictions: Verify all procedures, beginner-friendly language | Success: All docs reviewed, accurate, consistent style_
