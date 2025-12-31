# Requirements: Windows Platform Support

**Spec Name**: windows-platform-support
**Created**: 2024-12-24
**Status**: Draft
**Version**: 0.1.0

---

## Introduction

This document specifies the requirements for adding Windows platform support to keyrx, enabling cross-platform keyboard remapping with feature parity between Linux and Windows.

## Alignment with Product Vision

This feature directly supports multiple strategic goals outlined in product.md:

1. **Cross-Platform OS Integration**: Implements Windows support via Low-Level Hooks (WH_KEYBOARD_LL), complementing existing Linux evdev/uinput implementation
2. **Target User Expansion**: Unlocks access to competitive gamers (primarily Windows users) and professional power users on Windows (73% of desktop market)
3. **Sub-Millisecond Latency**: Maintains <1ms processing target on Windows, consistent with firmware-class performance goal
4. **AI-First Verification**: Enables automated testing via deterministic Windows API mocking and property-based tests
5. **Zero-Cost Abstractions**: Windows-specific code isolated behind Platform trait, preserving core logic's platform-agnostic nature

## Problem Statement

KeyRx currently supports Linux only (via evdev/uinput), limiting adoption to ~3% of desktop users. Windows represents ~73% of desktop market share and includes the majority of competitive gamers and professional power users—the primary target audience for sub-millisecond keyboard remapping.

**Current Gap**: Windows users cannot use KeyRx despite having the same need for low-latency keyboard customization.

### Business Value

- **Market Expansion**: Access to 73% of desktop users (Windows) vs current 3% (Linux)
- **Target User Alignment**: Competitive gamers primarily use Windows
- **Feature Parity**: Achieve cross-platform consistency for v0.2.0 release
- **Competitive Positioning**: Match competitors (AutoHotkey, PowerToys) while delivering superior latency

### Success Criteria

**Technical**:
- ✅ <1ms end-to-end remapping latency on Windows (same as Linux target)
- ✅ Feature parity: All core features work identically on Windows and Linux
- ✅ 95% test coverage for Windows-specific code
- ✅ CI/CD builds for Windows x86_64 (windows-msvc target)

**User Experience**:
- ✅ Installation: Single .exe file (portable) or MSI installer
- ✅ Configuration: Same .krx files work on both platforms
- ✅ Control: System tray icon for quick start/stop/reload

**Performance**:
- ✅ Latency: <1ms processing time (keyboard input → remapped output)
- ✅ Resource usage: <50MB RAM, <1% CPU idle

---

## User Stories

### Epic 1: Windows Keyboard Interception

#### US-1.1: As a Windows user, I want keyboard events intercepted before applications see them

**Acceptance Criteria**:
- WHEN I run `keyrx_daemon.exe run --config my-config.krx`
- THEN the daemon installs a low-level keyboard hook (WH_KEYBOARD_LL)
- AND all keyboard events are intercepted before reaching applications
- AND hook installation succeeds within 100ms

**EARS Criteria**:
- **Event**: User starts daemon with valid config file
- **Action**: Install Windows keyboard hook (SetWindowsHookEx)
- **Response**: Hook handle returned, all keyboard events routed to callback
- **State**: N/A (stateless installation)

**Priority**: P0 (Critical)
**Estimated Effort**: 3 days

---

#### US-1.2: As a Windows user, I want the daemon to release keyboard hooks cleanly on exit

**Acceptance Criteria**:
- WHEN I close the daemon (tray icon "Exit" or Ctrl+C)
- THEN the keyboard hook is uninstalled (UnhookWindowsHookEx)
- AND keyboard events return to normal OS routing
- AND cleanup completes within 50ms

**EARS Criteria**:
- **Event**: User triggers daemon shutdown (exit command, SIGTERM, or window close)
- **Action**: Call UnhookWindowsHookEx with hook handle
- **Response**: Hook removed, events pass through unmodified
- **State**: Hook must be installed (cannot unhook if not hooked)

**Priority**: P0 (Critical)
**Estimated Effort**: 1 day

---

### Epic 2: Virtual Key Code Mapping

#### US-2.1: As a Windows user, I want Windows Virtual Key codes mapped to KeyRx KeyCode enum

**Acceptance Criteria**:
- WHEN a keyboard event arrives with a Virtual Key code (e.g., VK_A = 0x41)
- THEN it is mapped to the corresponding KeyRx KeyCode (e.g., KeyCode::A)
- AND all standard keys are supported (letters, numbers, modifiers, function keys)
- AND unsupported keys are logged with warning (not crash)

**EARS Criteria**:
- **Event**: Windows keyboard hook receives KBDLLHOOKSTRUCT with vkCode
- **Action**: Look up vkCode in bidirectional mapping table
- **Response**: Return corresponding KeyCode variant
- **State**: N/A (stateless lookup)

**Priority**: P0 (Critical)
**Estimated Effort**: 2 days

---

#### US-2.2: As a developer, I want comprehensive tests for all Virtual Key mappings

**Acceptance Criteria**:
- WHEN I run the test suite
- THEN all standard VK codes (letters A-Z, numbers 0-9, modifiers, function keys) have tests
- AND edge cases (unmapped codes, extended keys) are tested
- AND bidirectional mapping is verified (VK→KeyCode→VK roundtrip)

**EARS Criteria**:
- **Event**: Test execution
- **Action**: Test all VK_* constants against mapping table
- **Response**: All mappings verified correct, edge cases handled
- **State**: N/A

**Priority**: P1 (High)
**Estimated Effort**: 1 day

---

### Epic 3: Event Injection

#### US-3.1: As a Windows user, I want remapped keys injected into the input stream

**Acceptance Criteria**:
- WHEN a key is remapped (e.g., CapsLock → Escape)
- THEN the output key event is injected via SendInput API
- AND the injected event appears to applications as a real keyboard press
- AND injection latency is <100μs

**EARS Criteria**:
- **Event**: Remapping logic outputs KeyCode (e.g., KeyCode::Escape)
- **Action**: Convert KeyCode to VK code, create INPUT structure, call SendInput
- **Response**: Event injected, applications receive remapped key
- **State**: Original event must be blocked (hook returns 1)

**Priority**: P0 (Critical)
**Estimated Effort**: 2 days

---

#### US-3.2: As a Windows user, I want modifier states (Shift, Ctrl, Alt, Win) preserved during remapping

**Acceptance Criteria**:
- WHEN I hold Shift and press a remapped key
- THEN the Shift modifier is preserved in the output
- AND modified output keys include correct modifier flags
- AND modifier-only keys (Shift down/up) are handled correctly

**EARS Criteria**:
- **Event**: Keyboard event with modifier flags set (e.g., Shift held)
- **Action**: Extract modifier state, apply to remapped output
- **Response**: Output event includes correct modifier flags
- **State**: Modifier state tracked across events

**Priority**: P0 (Critical)
**Estimated Effort**: 2 days

---

### Epic 4: System Tray Integration

#### US-4.1: As a Windows user, I want a system tray icon to control the daemon

**Acceptance Criteria**:
- WHEN the daemon starts
- THEN a tray icon appears in the Windows system tray
- AND the icon shows daemon status (active/inactive)
- AND hovering shows tooltip: "KeyRx Daemon - Active"

**EARS Criteria**:
- **Event**: Daemon startup completes
- **Action**: Create tray icon with Shell_NotifyIcon API
- **Response**: Icon appears in system tray notification area
- **State**: N/A

**Priority**: P1 (High)
**Estimated Effort**: 2 days

---

#### US-4.2: As a Windows user, I want a tray context menu for daemon control

**Acceptance Criteria**:
- WHEN I right-click the tray icon
- THEN a context menu appears with options:
  - "Reload Config" (re-reads .krx file without restart)
  - "Exit" (cleanly shuts down daemon)
- AND selecting "Reload Config" reloads the configuration
- AND selecting "Exit" terminates the daemon

**EARS Criteria**:
- **Event**: User right-clicks tray icon
- **Action**: Display TrackPopupMenu with menu items
- **Response**: Menu appears, user selection triggers corresponding action
- **State**: N/A

**Priority**: P1 (High)
**Estimated Effort**: 2 days

---

### Epic 5: Platform Abstraction

#### US-5.1: As a developer, I want Windows-specific code isolated behind the Platform trait

**Acceptance Criteria**:
- WHEN I examine the codebase
- THEN Windows-specific code is in `keyrx_daemon/src/platform/windows.rs`
- AND it implements the `Platform` trait (input/output device interfaces)
- AND no Windows-specific code leaks into cross-platform modules

**EARS Criteria**:
- **Event**: Code inspection
- **Action**: Verify module boundaries and trait implementation
- **Response**: Clean separation of concerns confirmed
- **State**: N/A

**Priority**: P0 (Critical)
**Estimated Effort**: 1 day (refactoring existing stub)

---

### Epic 6: Configuration Compatibility

#### US-6.1: As a Windows user, I want to use the same .krx config files as Linux

**Acceptance Criteria**:
- WHEN I copy a .krx file from Linux to Windows
- THEN the daemon loads it successfully
- AND all mappings work identically
- AND device-specific configs match by device name (not Linux-specific paths)

**EARS Criteria**:
- **Event**: User provides .krx file path via --config argument
- **Action**: Load file via rkyv deserialization (platform-agnostic)
- **Response**: Configuration loaded, mappings active
- **State**: N/A

**Priority**: P0 (Critical)
**Estimated Effort**: 1 day (verification only, already implemented in core)

---

### Epic 7: Testing & Quality

#### US-7.1: As a developer, I want automated tests for Windows-specific code

**Acceptance Criteria**:
- WHEN I run `cargo test --target x86_64-pc-windows-msvc`
- THEN all Windows-specific tests pass
- AND test coverage is ≥95% for `platform/windows.rs`
- AND tests include:
  - Hook installation/cleanup
  - VK code mapping (all standard keys)
  - Event injection
  - Tray icon creation/destruction

**EARS Criteria**:
- **Event**: CI/CD pipeline executes tests
- **Action**: Run test suite on Windows target
- **Response**: All tests pass, coverage meets threshold
- **State**: N/A

**Priority**: P1 (High)
**Estimated Effort**: 3 days

---

#### US-7.2: As a developer, I want CI/CD builds for Windows binaries

**Acceptance Criteria**:
- WHEN code is pushed to main branch
- THEN GitHub Actions builds Windows x86_64 binary
- AND binary is uploaded as release artifact
- AND release workflow creates GitHub release on version tags

**EARS Criteria**:
- **Event**: Git push to main or tag creation
- **Action**: GitHub Actions workflow builds with `--target x86_64-pc-windows-msvc`
- **Response**: Binary artifact created, release published (on tags)
- **State**: N/A

**Priority**: P2 (Medium)
**Estimated Effort**: 1 day (workflow already exists, needs verification)

---

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each Windows platform module has one clear purpose (keycode.rs = VK mapping only, hook.rs = hook management only, inject.rs = event injection only)
- **Modular Design**: Windows components isolated in `platform/windows/*` with trait-based abstraction (InputDevice/OutputDevice traits)
- **Dependency Management**: Windows-specific dependencies feature-gated (#[cfg(windows)]), no Windows code leaks into keyrx_core
- **Clear Interfaces**: Platform trait provides clean contract between Windows-specific code and cross-platform logic

### NFR-1: Performance

**Latency Budget**:
- Hook processing: <50μs
- KeyCode mapping: <10μs
- Event injection: <50μs
- **Total end-to-end**: <1ms (matching Linux target)

**Resource Usage**:
- Memory: <50MB RAM (excluding config file size)
- CPU: <1% idle, <5% under 1000 events/sec load

---

### NFR-2: Compatibility

**Windows Versions**:
- Windows 10 (version 1909+)
- Windows 11 (all versions)

**Architecture**:
- x86_64 only (no ARM/x86 initially)

---

### NFR-3: Reliability

**Error Handling**:
- Hook installation failures logged with actionable error messages
- Config file errors prevent daemon start (fail-fast)
- Graceful degradation: If tray icon fails, daemon continues via CLI

**Crash Recovery**:
- Hooks must be cleaned up even on panic (Drop trait implementation)

---

### NFR-4: Security

**Privileges**:
- No admin rights required for basic operation
- Hook installation works in user context
- Config files validated before loading (prevent code injection via malformed .krx)

**Anti-Cheat Compatibility**:
- Document limitations: Some games with kernel-level anti-cheat may detect hooks
- Provide workaround: Suspend remapping (future feature)

---

### Usability

**Installation**:
- Single-file executable (keyrx_daemon.exe) - no installer required for v0.2.0
- No admin privileges needed for basic operation
- Clear error messages if hook installation fails (with actionable suggestions)

**Configuration**:
- Same .krx files work on both Windows and Linux (no platform-specific configuration)
- Configuration errors detected at compile-time (keyrx_compiler validation)
- Reload config without restarting daemon (via tray menu)

**User Interface**:
- System tray icon provides visual status indicator
- Right-click menu for common operations (Reload Config, Exit)
- Tooltip shows daemon status at a glance
- Graceful degradation: Daemon continues via CLI if tray icon fails

**Documentation**:
- Windows setup guide with installation steps
- Troubleshooting section for common issues (hook conflicts, anti-cheat)
- Clear guidance on secure desktop limitations (UAC prompts, lock screen)

---

## Out of Scope (v0.2.0)

### Deferred to Later Versions

- **Kernel-mode driver**: Low-level hooks are sufficient for v0.2.0
- **Web UI**: Focus on tray icon, defer web interface to v0.3.0
- **MSI installer**: Portable .exe first, installer in v0.3.0
- **Gaming mode**: Suspend/resume feature deferred to v0.3.0
- **Per-application configs**: Window title-based rules deferred to v0.3.0
- **macOS support**: Not planned (Windows + Linux sufficient for target users)

---

## Dependencies

### External Dependencies

- **Windows SDK**: For Win32 API headers (provided by windows-sys crate)
- **Rust toolchain**: MSVC target (x86_64-pc-windows-msvc)

### Internal Dependencies

- **keyrx_core**: Platform-agnostic logic (already implemented)
- **keyrx_compiler**: .krx generation (already implemented)

---

## Assumptions & Constraints

### Assumptions

1. Users have Windows 10 1909+ or Windows 11
2. Users have basic CLI familiarity (`cmd.exe` or PowerShell)
3. Standard keyboard layouts (QWERTY, AZERTY, etc.) are used
4. No conflicting keyboard hooks from other software (e.g., AutoHotkey)

### Constraints

1. Low-level hooks have limitations:
   - Cannot intercept secure desktop (UAC prompts, lock screen)
   - Some anti-cheat software blocks hooks
   - Requires message loop (event loop integration)

2. Development constraints:
   - 3-week timeline for v0.2.0
   - No budget for driver code signing ($300-500/year)

---

## Risks & Mitigations

### Risk 1: Hook Latency Higher Than Expected

**Likelihood**: Low
**Impact**: High (defeats core value proposition)

**Mitigation**:
- Benchmark early (Week 1)
- If >1ms, escalate immediately
- Fallback: Document as "Windows beta" with known latency

---

### Risk 2: Anti-Cheat Software Blocks Hooks

**Likelihood**: Medium
**Impact**: Medium (limits gaming use case)

**Mitigation**:
- Document known incompatibilities
- Provide toggle to disable hooks (future feature)
- Target professional users first, gamers second

---

### Risk 3: Tray Icon Library Lacks Windows Support

**Likelihood**: Low
**Impact**: Low (can fallback to CLI-only)

**Mitigation**:
- Verify tray-icon crate on Windows during Week 1
- Fallback: CLI-only control if tray fails

---

## Acceptance Criteria Summary

**Definition of Done**:
- [ ] All P0 user stories implemented and tested
- [ ] Windows binary builds in CI/CD
- [ ] Test coverage ≥95% for platform/windows.rs
- [ ] Latency <1ms end-to-end verified via benchmarks
- [ ] Documentation updated (README, Windows setup guide)
- [ ] Tray icon functional with reload/exit menu
- [ ] Same .krx files work on Windows and Linux

---

## References

- [Windows Implementation Options RFC](../../../docs/rfcs/windows-implementation-options.md)
- [Product Steering Doc](../../steering/product.md)
- [Tech Steering Doc](../../steering/tech.md)
- Windows API Documentation: https://learn.microsoft.com/en-us/windows/win32/
- tray-icon crate: https://docs.rs/tray-icon/

---

**Document Status**: Ready for Review
