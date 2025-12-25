# Windows Quality Improvements - Requirements

## Overview

Systematic bug hunt and quality improvements for Windows-specific implementation of keyrx daemon, mirroring the comprehensive Linux bug hunt that found and fixed 5 critical bugs.

## Context

The Linux implementation underwent systematic bug hunting that found:
- 1 CRITICAL bug (signal handling)
- 2 HIGH bugs (device hot-unplug, grab rollback)
- 1 MEDIUM bug (poll busy loop)
- 2 LOW bugs (error flag handling, logging)

Windows implementation has **unique architecture** that requires separate analysis:
- Uses Raw Input API (not low-level hooks)
- Message pump-based event loop (not poll-based)
- Implicit device "grab" (not exclusive access)
- Raw pointer management for window context
- RwLock-based synchronization

## Goals

1. **Find Windows-Specific Bugs**: Identify issues unique to Windows implementation
2. **Improve Resource Safety**: Fix memory leaks, pointer issues, lock poisoning
3. **Enhance Error Recovery**: Add graceful error handling for Windows API failures
4. **Comprehensive Testing**: Write tests for all bugs before fixing
5. **Documentation**: Document Windows-specific architectural decisions

## Critical Areas to Investigate

### 1. Memory Safety & Raw Pointer Management
**Priority**: CRITICAL
**Locations**:
- `keyrx_daemon/src/platform/windows/rawinput.rs:52-60` - Box::into_raw()
- `keyrx_daemon/src/platform/windows/rawinput.rs:177-187` - Drop with pointer reconstruction

**Issues to Find**:
- [ ] Use-after-free if RawInputManager dropped during wnd_proc execution
- [ ] Double-free if window destroyed externally
- [ ] Thread safety of raw pointer access
- [ ] Lifetime mismatch between window handle and Rust ownership

### 2. RwLock Panic & Poisoning
**Priority**: HIGH
**Locations**:
- `keyrx_daemon/src/platform/windows/device_map.rs:94` - `.write().unwrap()`
- `keyrx_daemon/src/platform/windows/rawinput.rs:74` - `.write().unwrap()`
- `keyrx_daemon/src/platform/windows/rawinput.rs:298` - `.read().unwrap()`

**Issues to Find**:
- [ ] Lock poisoning cascades if thread panics while holding lock
- [ ] No fallback error handling for poisoned locks
- [ ] Panic in wnd_proc crashes daemon

### 3. Message Queue & Event Processing
**Priority**: HIGH
**Locations**:
- `keyrx_daemon/src/main.rs:185-220` - Message pump loop
- `keyrx_daemon/src/platform/windows/rawinput.rs:214-230` - GetRawInputData

**Issues to Find**:
- [ ] Message queue overflow under high event rate
- [ ] Unbounded memory allocation in GetRawInputData
- [ ] No rate limiting for keyboard events
- [ ] Lost events if queue fills up
- [ ] No timeout or watchdog for stuck messages

### 4. Error Recovery & Crash Resilience
**Priority**: HIGH
**Locations**:
- `keyrx_daemon/src/main.rs:185-220` - Main message loop
- `keyrx_daemon/src/platform/windows/mod.rs:43-52` - init() error paths

**Issues to Find**:
- [ ] Unhandled panics in DispatchMessageW
- [ ] No restart mechanism on crash
- [ ] Partial cleanup on init() failure
- [ ] No recovery from Windows API errors

### 5. Device Management & Hotplug
**Priority**: MEDIUM
**Locations**:
- `keyrx_daemon/src/platform/windows/rawinput.rs:243-260` - WM_INPUT_DEVICE_CHANGE
- `keyrx_daemon/src/platform/windows/device_map.rs` - Device tracking

**Issues to Find**:
- [ ] Silent failure on device arrival: `let _ = add_device()`
- [ ] No config reload when device added
- [ ] Race condition: device removed during event processing
- [ ] Stale device references after removal

### 6. Windows API Error Handling
**Priority**: MEDIUM
**Locations**:
- `keyrx_daemon/src/platform/windows/rawinput.rs:109-131` - RegisterClassExW
- `keyrx_daemon/src/platform/windows/rawinput.rs:142-153` - CreateWindowExW
- `keyrx_daemon/src/platform/windows/inject.rs` - SendInput

**Issues to Find**:
- [ ] Swallowed errors in window class registration
- [ ] No validation of Windows API return values
- [ ] Silent failures in SendInput injection
- [ ] Missing GetLastError() diagnostics

### 7. Scancode & KeyCode Mapping
**Priority**: LOW
**Locations**:
- `keyrx_daemon/src/platform/windows/keycode.rs:161-176` - scancode_to_keycode
- `keyrx_daemon/src/platform/windows/inject.rs:49-65` - is_extended_key

**Issues to Find**:
- [ ] Layout-dependent MapVirtualKeyW (placeholder implementation)
- [ ] Lost information in scancode → VK → KeyCode conversion
- [ ] Incomplete extended key list (hardcoded)
- [ ] IME/special keys not handled correctly

### 8. Resource Cleanup & Leaks
**Priority**: LOW
**Locations**:
- `keyrx_daemon/src/platform/windows/rawinput.rs:177-187` - Drop impl
- `keyrx_daemon/src/platform/windows/mod.rs` - WindowsPlatform lifecycle

**Issues to Find**:
- [ ] Window handle leaks on error paths
- [ ] Raw Input registration not unregistered
- [ ] Channel leaks if receiver dropped
- [ ] Memory allocated in GetRawInputData not freed

## Comparison: Windows vs Linux Issues

### Issues Fixed in Linux (Also Benefit Windows)
✅ **BUG #36 (CRITICAL)**: Signal handling - Windows uses different mechanism (WM_QUIT)
✅ **BUG #34 (MEDIUM)**: Poll error backoff - Windows uses message pump
✅ **BUG #35 (LOW)**: POLLERR handling - Windows uses different event model
✅ **BUG #38 (LOW)**: Config reload logging - **BENEFITS WINDOWS** (same code)
✅ **BUG #37 (HIGH)**: Unwrap on device removal - **BENEFITS WINDOWS** (same code)

### Windows-Specific Issues (Need Investigation)
❌ **Raw pointer safety**: Use-after-free, double-free
❌ **RwLock poisoning**: Cascade failures
❌ **Message queue overflow**: Lost events, memory exhaustion
❌ **Error recovery**: No panic handling in message loop
❌ **Device hotplug**: Silent failures, race conditions
❌ **Windows API errors**: Swallowed errors, no diagnostics

## Success Criteria

### Minimum
- [ ] Find at least 5 Windows-specific bugs
- [ ] All bugs categorized by severity (CRITICAL/HIGH/MEDIUM/LOW)
- [ ] Write tests for all bugs before fixing
- [ ] Document root causes and user impact

### Target
- [ ] Find 8+ Windows-specific bugs
- [ ] Fix all CRITICAL and HIGH bugs
- [ ] Achieve same quality bar as Linux implementation
- [ ] Create Windows-specific E2E tests

### Stretch
- [ ] Improve Windows architecture (reduce raw pointers, use safer abstractions)
- [ ] Add Windows-specific documentation
- [ ] Create Windows CI/CD testing pipeline

## Non-Goals

- Changing Windows to use hooks instead of Raw Input (architectural decision)
- Making Windows behave identically to Linux (different OS capabilities)
- Supporting Windows XP or older (target Windows 10+)

## Out of Scope

- GUI/tray icon improvements (separate feature)
- Performance optimization (focus on correctness first)
- Windows-specific features (focus on parity with Linux)

## Constraints

### Technical
- Must maintain Raw Input API approach (not switching to hooks)
- Must keep message pump architecture
- Must work on Windows 10+ (64-bit)
- Must handle both admin and non-admin modes

### Testing
- Requires Windows PC for testing
- Need admin privileges for some tests
- Virtual device creation more complex than Linux
- No equivalent to evdev for testing

### Resources
- User will test on Windows PC (not CI yet)
- Manual testing required (no automated Windows E2E yet)

## Dependencies

### Code
- `windows-sys` crate for Windows API bindings
- `crossbeam-channel` for event passing
- Existing test infrastructure from Linux bug hunt

### Tools
- Windows 10+ PC
- Admin privileges for testing
- Multiple keyboards for device testing (optional)

### Knowledge
- Linux bug hunt results (for comparison)
- Windows Raw Input API documentation
- Windows message pump internals

## Timeline

### Phase 1: Investigation (Est. 2-3 hours)
- Systematic code review of all Windows-specific code
- Pattern matching for common bug types
- Document all findings

### Phase 2: Testing (Est. 3-4 hours)
- Write tests for each bug found
- Verify bugs can be reproduced
- Test on Windows PC

### Phase 3: Fixing (Est. 2-3 hours)
- Fix bugs in priority order
- Verify fixes with tests
- Update documentation

### Total: 8-10 hours (similar to Linux bug hunt)

## Stakeholders

- **Developer**: Needs robust Windows daemon
- **Users**: Windows users need same quality as Linux
- **QA**: Need comprehensive test coverage
- **Documentation**: Windows-specific docs needed

## Risks

### Technical Risks
- **Raw pointer issues may be architectural**: Might require significant refactoring
- **Message pump testing complex**: Hard to simulate Windows message queue behavior
- **RwLock poisoning hard to test**: Need to deliberately panic threads

### Mitigation
- Start with less invasive fixes (error handling, logging)
- Create isolated tests for unsafe code
- Document architectural limitations

### Schedule Risks
- Testing on Windows only (no CI yet)
- Manual testing slower than automated

### Mitigation
- Focus on high-impact bugs first
- Create test plan before implementing

## Open Questions

1. Should we refactor away from raw pointers entirely?
2. Is RwLock the right synchronization primitive for Windows?
3. Should daemon continue without tray icon (headless mode)?
4. How to test message queue overflow in controlled manner?
5. Should we add Windows-specific CI runners?

## References

- Linux Bug Hunt Results: `/tmp/bug-hunt-findings.md`
- Windows Implementation Analysis: Agent a70e62b
- Windows Raw Input API: https://docs.microsoft.com/en-us/windows/win32/inputdev/raw-input
- Rust Raw Pointers: https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
